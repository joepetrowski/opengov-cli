use crate::*;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration describing how to launch chopsticks.
struct ChopsticksConfig {
	/// The chain config name for chopsticks (e.g. "asset-hub-kusama").
	chain: String,
	/// The WS port (default 8000).
	port: u16,
}

/// Post-AHM, all governance lives on Asset Hub. Fork Asset Hub for all tracks.
fn get_chopsticks_config(proposal_details: &ProposalDetails) -> ChopsticksConfig {
	match &proposal_details.track {
		NetworkTrack::KusamaRoot | NetworkTrack::Kusama(_) => ChopsticksConfig {
			chain: "kusama-asset-hub".to_string(),
			port: 8000,
		},
		NetworkTrack::PolkadotRoot | NetworkTrack::Polkadot(_) => ChopsticksConfig {
			chain: "polkadot-asset-hub".to_string(),
			port: 8000,
		},
	}
}

/// Boot chopsticks, generate and execute the test JS script, then clean up.
pub(crate) async fn run_chopsticks_tests(
	proposal_details: &ProposalDetails,
	calls: &PossibleCallsToSubmit,
	test_file_path: &str,
) {
	let config = get_chopsticks_config(proposal_details);

	// Start chopsticks
	let mut chopsticks_process = start_chopsticks(&config);

	// Wait for chopsticks to become ready
	println!("Waiting for chopsticks to start...");
	if !wait_for_chopsticks(config.port, 60).await {
		eprintln!("Error: chopsticks did not become ready within 60 seconds.");
		eprintln!("Make sure it is installed: npm install -g @acala-network/chopsticks");
		let _ = chopsticks_process.kill();
		let _ = chopsticks_process.wait();
		return;
	}
	println!("Chopsticks is ready.");

	let script = generate_test_script(proposal_details, calls, test_file_path, &config);

	let temp_dir = std::env::temp_dir();
	let temp_script = temp_dir.join("opengov_cli_chopsticks_test.js");
	fs::write(&temp_script, script).expect("Failed to write temp test script");

	println!("Running test script...");
	let result = execute_test_script(temp_script.to_str().unwrap()).await;

	let _ = chopsticks_process.kill();
	let _ = chopsticks_process.wait();
	let _ = fs::remove_file(&temp_script);

	match result {
		Ok(()) => println!("Chopsticks test completed successfully."),
		Err(e) => {
			eprintln!("Chopsticks test failed: {}", e);
			std::process::exit(1);
		},
	}
}

fn start_chopsticks(config: &ChopsticksConfig) -> std::process::Child {
	println!("Starting chopsticks: chain={}", config.chain);
	Command::new("chopsticks")
		.args(["-c", &config.chain, "--port", &config.port.to_string()])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect(
			"Failed to start chopsticks. Install it with: npm install -g @acala-network/chopsticks",
		)
}

/// Poll the chopsticks HTTP endpoint until it responds or we time out.
async fn wait_for_chopsticks(port: u16, timeout_secs: u64) -> bool {
	let start = std::time::Instant::now();
	let timeout = Duration::from_secs(timeout_secs);
	let url = format!("http://127.0.0.1:{}", port);

	while start.elapsed() < timeout {
		let result = Command::new("curl")
			.args([
				"-s",
				"-o",
				"/dev/null",
				"-w",
				"%{http_code}",
				"-X",
				"POST",
				"-H",
				"Content-Type: application/json",
				"-d",
				r#"{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}"#,
				&url,
			])
			.output();

		if let Ok(output) = result {
			let code = String::from_utf8_lossy(&output.stdout);
			if code.trim() == "200" {
				return true;
			}
		}
		sleep(Duration::from_secs(2)).await;
	}
	false
}

/// Get the raw proposal hex from the proposal details. This is the actual call
/// that should be executed on Asset Hub when the referendum passes.
fn get_proposal_hex(proposal_details: &ProposalDetails) -> String {
	let proposal_bytes = get_proposal_bytes(proposal_details.proposal.clone());
	format!("0x{}", hex::encode(&proposal_bytes))
}

/// Get the origin descriptor for scheduler injection on Asset Hub.
fn get_origin_for_injection(proposal_details: &ProposalDetails) -> (&'static str, &'static str) {
	match &proposal_details.track {
		NetworkTrack::KusamaRoot | NetworkTrack::PolkadotRoot => ("system", "Root"),
		NetworkTrack::Kusama(origin) => {
			use KusamaAssetHubOpenGovOrigin::*;
			match origin {
				WhitelistedCaller => ("system", "Root"),
				StakingAdmin => ("Origins", "StakingAdmin"),
				Treasurer => ("Origins", "Treasurer"),
				LeaseAdmin => ("Origins", "LeaseAdmin"),
				FellowshipAdmin => ("Origins", "FellowshipAdmin"),
				GeneralAdmin => ("Origins", "GeneralAdmin"),
				AuctionAdmin => ("Origins", "AuctionAdmin"),
				ReferendumCanceller => ("Origins", "ReferendumCanceller"),
				ReferendumKiller => ("Origins", "ReferendumKiller"),
				_ => panic!("Unsupported Kusama origin for chopsticks testing"),
			}
		},
		NetworkTrack::Polkadot(origin) => {
			use PolkadotAssetHubOpenGovOrigin::*;
			match origin {
				WhitelistedCaller => ("system", "Root"),
				StakingAdmin => ("Origins", "StakingAdmin"),
				Treasurer => ("Origins", "Treasurer"),
				LeaseAdmin => ("Origins", "LeaseAdmin"),
				FellowshipAdmin => ("Origins", "FellowshipAdmin"),
				GeneralAdmin => ("Origins", "GeneralAdmin"),
				AuctionAdmin => ("Origins", "AuctionAdmin"),
				ReferendumCanceller => ("Origins", "ReferendumCanceller"),
				ReferendumKiller => ("Origins", "ReferendumKiller"),
				_ => panic!("Unsupported Polkadot origin for chopsticks testing"),
			}
		},
	}
}

/// Generate the JS test script.
///
/// The approach: directly inject the raw proposal into the scheduler on Asset Hub
/// with the track's origin. This simulates what happens when a referendum passes
/// and the proposal is executed. For WhitelistedCaller, we skip the whitelist
/// ceremony - the scheduler dispatches with the origin directly.
fn generate_test_script(
	proposal_details: &ProposalDetails,
	_calls: &PossibleCallsToSubmit,
	user_test_file: &str,
	config: &ChopsticksConfig,
) -> String {
	let proposal_hex = get_proposal_hex(proposal_details);
	let (origin_type, origin_value) = get_origin_for_injection(proposal_details);
	let port = config.port;

	// Resolve the user test file to an absolute path for require()
	let user_test_abs = std::path::Path::new(user_test_file);
	let user_test_resolved = if user_test_abs.is_absolute() {
		user_test_file.to_string()
	} else {
		std::env::current_dir()
			.map(|d| d.join(user_test_file).to_string_lossy().to_string())
			.unwrap_or_else(|_| user_test_file.to_string())
	};

	format!(
		r#"const {{ ApiPromise, WsProvider }} = require('@polkadot/api');
const {{ blake2AsHex }} = require('@polkadot/util-crypto');

async function connectToChopsticks(port) {{
	const provider = new WsProvider(`ws://127.0.0.1:${{port}}`);
	const api = await ApiPromise.create({{ provider }});
	await api.isReady;
	const chain = await api.rpc.system.chain();
	console.log(`Connected to ${{chain}} on port ${{port}}`);
	return api;
}}

/**
 * Inject a call into the scheduler at the current relay parent block, dispatched
 * from the given origin. Post-AHM, the scheduler uses relay chain block numbers
 * as its clock, so we must schedule at the relay parent number (not parachain block).
 * We also clear scheduler.incompleteSince to avoid stale state blocking execution.
 */
async function injectSchedulerCall(api, callDataHex, originType, originValue) {{
	// Post-AHM: scheduler uses relay chain block numbers
	const validationData = await api.query.parachainSystem.validationData();
	const relayParent = validationData.toJSON()?.relayParentNumber;
	if (!relayParent) {{
		throw new Error('Could not read relay parent number from parachainSystem.validationData');
	}}
	const targetBlock = relayParent;

	const callBytes = callDataHex.startsWith('0x') ? callDataHex : '0x' + callDataHex;
	const callBytesRaw = Uint8Array.from(Buffer.from(callBytes.slice(2), 'hex'));

	// Use Inline for small calls (<=128 bytes), Lookup for larger ones
	let callEntry;
	if (callBytesRaw.length <= 128) {{
		callEntry = {{ Inline: callBytes }};
	}} else {{
		const callHash = blake2AsHex(callBytes, 256);
		const callLen = callBytesRaw.length;

		// Set preimage status as Requested (required for scheduler fetch) via raw key
		const requestStatusKey = api.query.preimage.requestStatusFor.key(callHash);
		const statusValue = api.registry.createType('PalletPreimageRequestStatus', {{
			Requested: {{ maybeTicket: null, count: 1, maybeLen: callLen }}
		}});

		// SCALE-encode the preimage as BoundedVec<u8> (compact_length + raw_bytes)
		// Note: Bytes.toU8a() includes the SCALE length prefix, .toHex() does not
		const preimageScaled = '0x' + Buffer.from(
			api.registry.createType('Bytes', callBytes).toU8a()
		).toString('hex');
		const preimageKey = api.query.preimage.preimageFor.key([callHash, callLen]);

		await api.rpc('dev_setStorage', [
			[requestStatusKey, statusValue.toHex()],
			[preimageKey, preimageScaled]
		]);
		callEntry = {{ Lookup: {{ hash: callHash, len: callLen }} }};
	}}

	// Clear incompleteSince and inject the agenda in a single setStorage call
	await api.rpc('dev_setStorage', {{
		scheduler: {{
			incompleteSince: null,
			agenda: [
				[[targetBlock], [{{
					maybeId: null,
					priority: 128,
					call: callEntry,
					maybePeriodic: null,
					origin: {{ [originType]: originValue }},
				}}]]
			]
		}}
	}});

	console.log(`  Scheduled call at relay block ${{targetBlock}} with origin ${{originType}}:${{originValue}}`);
	return targetBlock;
}}

/**
 * Create a new block via dev_newBlock and return the block hash.
 */
async function createBlock(api) {{
	const blockHash = await api.rpc('dev_newBlock', {{ count: 1 }});
	const header = await api.rpc.chain.getHeader();
	console.log(`  New block: #${{header.number.toNumber()}} (${{blockHash}})`);
	return blockHash;
}}

/**
 * Verify that the scheduler dispatched the call successfully by inspecting
 * system events at the given block hash.
 */
async function verifyDispatch(api, blockHash, targetBlock) {{
	const events = await api.query.system.events.at(blockHash);
	let dispatched = false;
	let dispatchError = null;
	let callUnavailable = false;
	const errors = [];
	const warnings = [];

	for (const record of events) {{
		const {{ event }} = record;

		if (event.section === 'scheduler' && event.method === 'Dispatched') {{
			dispatched = true;
			const result = event.data[event.data.length - 1];
			if (result.isErr) {{
				dispatchError = result.asErr.toString();
			}}
		}}

		if (event.section === 'scheduler' && event.method === 'CallUnavailable') {{
			callUnavailable = true;
		}}

		if (event.section === 'utility' && event.method === 'BatchInterrupted') {{
			errors.push('utility.BatchInterrupted: batch stopped, remaining sub-calls not executed — ' + event.data.toString());
		}}
		if (event.section === 'utility' && event.method === 'ItemFailed') {{
			warnings.push('utility.ItemFailed: ' + event.data.toString());
		}}
	}}

	if (callUnavailable) {{
		throw new Error('scheduler.CallUnavailable — the proposal call could not be resolved. ' +
			'For large calls (>128 bytes), this may indicate a preimage encoding issue.');
	}}

	if (!dispatched) {{
		throw new Error('No scheduler.Dispatched event found — the proposal was not executed. ' +
			'This may indicate a scheduler misconfiguration or weight limit issue.');
	}}

	if (dispatchError) {{
		throw new Error('Proposal dispatched but execution failed: ' + dispatchError);
	}}

	if (errors.length > 0) {{
		throw new Error('Proposal dispatched but inner calls failed:\\n  ' + errors.join('\\n  '));
	}}

	const agenda = await api.query.scheduler.agenda(targetBlock);
	const remaining = agenda.filter(item => item.isSome);
	if (remaining.length > 0) {{
		throw new Error(`Scheduler agenda at relay block ${{targetBlock}} still has ${{remaining.length}} ` +
			'unprocessed item(s) — the proposal may not have been executed.');
	}}

	for (const w of warnings) {{
		console.log('  WARNING: ' + w);
	}}

	console.log('  Dispatch verified: scheduler.Dispatched with Ok result, agenda consumed.');
}}

async function main() {{
	try {{
		// Load user test module
		let userModule;
		try {{
			userModule = require('{user_test_resolved}');
		}} catch (e) {{
			throw new Error('Failed to load test module "{user_test_resolved}": ' + e.message +
				'\\nMake sure the file exists, uses CommonJS (module.exports), and has no syntax errors.');
		}}

		// Run user pre-run setup
		if (userModule && typeof userModule.setup === 'function') {{
			console.log('Running user pre-run setup...');
			await userModule.setup(connectToChopsticks);
		}}

		// Inject the proposal into the scheduler and execute it
		console.log('Injecting proposal call...');
		const api = await connectToChopsticks({port});
		const targetBlock = await injectSchedulerCall(api, '{proposal_hex}', '{origin_type}', '{origin_value}');
		const blockHash = await createBlock(api);
		await verifyDispatch(api, blockHash, targetBlock);
		console.log('Proposal executed successfully.');

		// Run user post-run assertions
		if (userModule && typeof userModule.test === 'function') {{
			console.log('Running user post-run assertions...');
			await userModule.test(api, connectToChopsticks);
		}} else {{
			console.log('No user test() function found, skipping post-run assertions.');
		}}

		await api.disconnect();
		console.log('Test completed successfully.');
		process.exit(0);
	}} catch (error) {{
		console.error('Test failed:', error.message);
		console.error(error.stack);
		process.exit(1);
	}}
}}

main();
"#,
		user_test_resolved = user_test_resolved,
		port = port,
		proposal_hex = proposal_hex,
		origin_type = origin_type,
		origin_value = origin_value,
	)
}

/// Execute the generated JS test script with Node.
async fn execute_test_script(script_path: &str) -> Result<(), String> {
	// Resolve global npm root so require() can find @polkadot/api etc.
	let node_path = Command::new("npm")
		.args(["root", "-g"])
		.output()
		.ok()
		.and_then(|o| String::from_utf8(o.stdout).ok())
		.unwrap_or_default();

	let output = Command::new("node")
		.arg(script_path)
		.env("NODE_PATH", node_path.trim())
		.output()
		.map_err(|e| format!("Failed to execute test script: {}", e))?;

	// Always show output
	if !output.stdout.is_empty() {
		print!("{}", String::from_utf8_lossy(&output.stdout));
	}
	if !output.stderr.is_empty() {
		eprint!("{}", String::from_utf8_lossy(&output.stderr));
	}

	if output.status.success() {
		Ok(())
	} else {
		Err(format!("Process exited with code: {:?}", output.status.code()))
	}
}

/// Generate test scaffolding for a given network.
///
/// The scaffold provides a JS module that exports `setup()` and `test()` hooks
/// to be called by the chopsticks runner.
pub(crate) fn generate_test_scaffold(network: &str) -> String {
	let chain_config = match network {
		"polkadot" => "polkadot-asset-hub",
		"kusama" => "kusama-asset-hub",
		_ => "polkadot-asset-hub",
	};

	format!(
		r#"/**
 * Test file for {network} OpenGov referendum testing with Chopsticks.
 *
 * Usage:
 *   opengov-cli submit-referendum \
 *     --proposal "./your-proposal.call" \
 *     --network "{network}" \
 *     --track "whitelistedcaller" \
 *     --test "testfile.js"
 *
 * Chopsticks will fork {chain_config} (post-AHM, all governance lives on Asset Hub).
 *
 * This module should export:
 *   setup(connectToChopsticks) - called BEFORE the proposal call is injected
 *   test(api, connectToChopsticks) - called AFTER the proposal has been executed
 *
 * The `connectToChopsticks(port)` helper returns a @polkadot/api ApiPromise connected
 * to the given chopsticks port (default: 8000).
 */

/**
 * Pre-run setup. Use this to fund accounts, set storage, etc.
 * @param {{Function}} connectToChopsticks - async (port) => ApiPromise
 */
async function setup(connectToChopsticks) {{
	const api = await connectToChopsticks(8000);

	// Example: Fund Alice
	await api.rpc('dev_setStorage', {{
		system: {{
			account: [
				[['5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'], {{
					providers: 1,
					data: {{
						free: '1000000000000000000',
					}}
				}}]
			]
		}}
	}});
	console.log('Alice funded.');

	await api.disconnect();
}}

/**
 * Post-run assertions. The API is connected to the Asset Hub fork after the proposal executed.
 * Add your checks here to verify the proposal executed as expected.
 *
 * @param {{ApiPromise}} api - connected to Asset Hub after dispatch
 * @param {{Function}} connectToChopsticks - async (port) => ApiPromise
 */
async function test(api, connectToChopsticks) {{
	// Example: check runtime version after an upgrade
	const version = await api.rpc.state.getRuntimeVersion();
	console.log('Runtime version:', version.specName.toString(), version.specVersion.toNumber());

	// Example: check system.authorizedUpgrade for a runtime upgrade
	// const authorized = await api.query.system.authorizedUpgrade();
	// console.log('Authorized upgrade:', authorized.toJSON());

	// Add your assertions here. Throw an error to fail the test:
	// if (someCondition) throw new Error('Assertion failed: ...');
}}

module.exports = {{ setup, test }};
"#,
		network = network,
		chain_config = chain_config,
	)
}
