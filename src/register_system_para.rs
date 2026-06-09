use crate::*;
use clap::Parser as ClapParser;
use std::fs;

/// Generate the calls needed to register a system parachain via Asset Hub governance.
///
/// Produces:
///   - A `force_register_call.hex` file with raw preimage bytes for the relay chain
///   - A proposal file with a batched Asset Hub call: `utility.batch_all([xcm_whitelist, xcm_dispatch])`
///
/// The proposal uses the whitelist+preimage pattern to work around the 128 KB UMP limit.
/// Asset Hub has Root on the relay chain via LocationAsSuperuser, so the XCM Transacts
/// execute whitelist.whitelist_call and whitelist.dispatch_whitelisted_call with Root origin.
#[derive(Debug, ClapParser)]
pub(crate) struct RegisterSystemParaArgs {
	/// Path to the WASM validation code file.
	#[clap(long = "wasm")]
	wasm: String,

	/// Path to the genesis head hex file (from: polkadot-omni-node export-genesis-head).
	#[clap(long = "genesis-head")]
	genesis_head: String,

	/// Parachain ID to register.
	#[clap(long = "para-id")]
	para_id: u32,

	/// Network. Currently only `polkadot` is supported.
	#[clap(long = "network", short)]
	network: String,

	/// Manager account SS58 address.
	#[clap(long = "manager")]
	manager: String,

	/// Deposit in plancks.
	#[clap(long = "deposit", default_value = "0")]
	deposit: u128,

	/// Weight ref_time witness for dispatch_whitelisted_call.
	/// Default based on Polkadot benchmarks for force_register (~7.7B) with headroom.
	#[clap(long = "ref-time", default_value = "10000000000")]
	ref_time: u64,

	/// Weight proof_size witness for dispatch_whitelisted_call.
	/// Default based on Polkadot benchmarks for force_register (~3697) with headroom.
	#[clap(long = "proof-size", default_value = "5000")]
	proof_size: u64,

	/// Reserve a dedicated core for this parachain on the Coretime chain via broker.force_reserve.
	/// Provide the core index. Sent as XCM from Asset Hub to the Coretime chain.
	#[clap(long = "assign-core")]
	assign_core: Option<u16>,

	/// Delay dispatch_whitelisted_call using Scheduler.schedule_after so that the preimage
	/// can be noted for free after whitelist_call requests it. Value is the delay in relay
	/// chain blocks (e.g. 100 = ~10 minutes on Polkadot).
	#[clap(long = "delay-whitelist-dispatch-relay")]
	delay_whitelist_dispatch_relay: Option<u32>,

	/// Output file name for the Asset Hub proposal.
	#[clap(long = "filename")]
	filename: Option<String>,
}

/// Parameters for building registration calls (decoupled from CLI/file I/O).
pub(crate) struct RegisterSystemParaParams {
	pub wasm_bytes: Vec<u8>,
	pub genesis_head_bytes: Vec<u8>,
	pub para_id: u32,
	pub manager_bytes: [u8; 32],
	pub deposit: u128,
	pub ref_time: u64,
	pub proof_size: u64,
	/// If set, reserve this core index for the para via broker.force_reserve on the Coretime chain.
	pub assign_core: Option<u16>,
	/// If set, wrap dispatch_whitelisted_call in Scheduler.schedule_after with this delay (blocks).
	/// This allows the preimage to be noted for free after whitelist_call requests it.
	pub delay_whitelist_dispatch_relay: Option<u32>,
}

/// Output of building registration calls.
pub(crate) struct RegisterSystemParaOutput {
	/// The encoded force_register call (relay chain). Used as preimage content.
	pub force_register_info: CallInfo,
	/// The batched Asset Hub proposal (utility.batch_all with two or three XCM sends).
	pub proposal: CallInfo,
	/// The encoded force_reserve call (coretime chain), if --assign-core was used.
	pub force_reserve_info: Option<CallInfo>,
}

/// Build the registration calls from parameters (pure logic, no file I/O).
///
/// Composes primitives from the `primitives` module:
/// 1. `force_register` (relay)
/// 2. `whitelist_call(hash)` wrapped in XCM from AH → Relay
/// 3. `dispatch_whitelisted_call(hash, len, weight)` optionally wrapped in Scheduler, then XCM
/// 4. Optional `force_reserve(para_id, core)` wrapped in XCM from AH → Coretime
/// 5. All AH calls batched via `Utility.batch_all`
pub(crate) fn build_polkadot_register_calls(params: RegisterSystemParaParams) -> RegisterSystemParaOutput {
	use crate::primitives::{
		batch_all_on_ah, build_dispatch_whitelisted_call, build_force_register_call,
		build_force_reserve_call, build_whitelist_call, wrap_in_relay_scheduler,
		wrap_in_xcm_send_from_ah, ForceRegisterParams, XcmDest,
	};

	// 1. Relay force_register (preimage content)
	let force_register_info = build_force_register_call(ForceRegisterParams {
		wasm_bytes: params.wasm_bytes,
		genesis_head_bytes: params.genesis_head_bytes,
		para_id: params.para_id,
		manager_bytes: params.manager_bytes,
		deposit: params.deposit,
	});

	// 2. whitelist_call(hash) + XCM wrap
	let whitelist_info = build_whitelist_call(force_register_info.hash);
	let xcm_whitelist = wrap_in_xcm_send_from_ah(XcmDest::Relay, whitelist_info.encoded);

	// 3. dispatch_whitelisted_call(hash, len, weight), optional Scheduler wrap, then XCM wrap
	let dispatch_call = build_dispatch_whitelisted_call(
		force_register_info.hash,
		force_register_info.length,
		params.ref_time,
		params.proof_size,
	);
	let dispatch_call = match params.delay_whitelist_dispatch_relay {
		Some(delay) => wrap_in_relay_scheduler(dispatch_call, delay),
		None => dispatch_call,
	};
	let dispatch_info = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(dispatch_call));
	let xcm_dispatch = wrap_in_xcm_send_from_ah(XcmDest::Relay, dispatch_info.encoded);

	// 4. Optional force_reserve on Coretime via XCM
	let mut batch_calls = vec![xcm_whitelist, xcm_dispatch];
	let force_reserve_info = params.assign_core.map(|core| {
		let fr_info = build_force_reserve_call(params.para_id, core);
		let xcm_force_reserve =
			wrap_in_xcm_send_from_ah(XcmDest::Sibling(1005), fr_info.encoded.clone());
		batch_calls.push(xcm_force_reserve);
		fr_info
	});

	// 5. Batch into single Asset Hub proposal
	let proposal = batch_all_on_ah(batch_calls);

	RegisterSystemParaOutput { force_register_info, proposal, force_reserve_info }
}

pub(crate) async fn register_system_para(args: RegisterSystemParaArgs) {
	match args.network.to_ascii_lowercase().as_str() {
		"polkadot" => register_polkadot_system_para(args).await,
		"kusama" => {
			panic!("Kusama support not yet implemented. Use --network polkadot.");
		},
		_ => panic!("`network` must be `polkadot` or `kusama`"),
	}
}

async fn register_polkadot_system_para(args: RegisterSystemParaArgs) {
	// Read inputs
	let wasm_bytes = fs::read(&args.wasm).expect("Should read WASM file");
	println!("WASM size: {} bytes", wasm_bytes.len());

	let genesis_head_hex = fs::read_to_string(&args.genesis_head)
		.expect("Should read genesis head file")
		.trim()
		.to_string();
	let genesis_head_bytes =
		hex::decode(genesis_head_hex.trim_start_matches("0x")).expect("Valid hex for genesis head");
	println!("Genesis head size: {} bytes", genesis_head_bytes.len());

	use sp_core::crypto::Ss58Codec;
	let manager_account =
		sp_core::crypto::AccountId32::from_ss58check(&args.manager).expect("Valid SS58 address");
	let manager_bytes: [u8; 32] = manager_account.into();

	println!("ParaId: {}", args.para_id);
	println!("Manager: {}", args.manager);
	println!("Deposit: {}", args.deposit);

	if let Some(core) = args.assign_core {
		println!("Assign core: {} (via broker.force_reserve on Coretime chain)", core);
	}
	if let Some(delay) = args.delay_whitelist_dispatch_relay {
		println!("Free preimage: dispatch delayed by {} blocks (~{} minutes)", delay, delay * 6 / 60);
	}

	// Build calls
	let output = build_polkadot_register_calls(RegisterSystemParaParams {
		wasm_bytes,
		genesis_head_bytes,
		para_id: args.para_id,
		manager_bytes,
		deposit: args.deposit,
		ref_time: args.ref_time,
		proof_size: args.proof_size,
		assign_core: args.assign_core,
		delay_whitelist_dispatch_relay: args.delay_whitelist_dispatch_relay,
	});

	println!("\nforce_register call:");
	println!("  Encoded size: {} bytes", output.force_register_info.length);
	println!("  Hash: 0x{}", hex::encode(output.force_register_info.hash));

	// Write raw preimage bytes
	let mut preimage_hex = "0x".to_owned();
	preimage_hex.push_str(&hex::encode(&output.force_register_info.encoded));
	fs::write("force_register_call.hex", &preimage_hex).expect("write force_register_call.hex");
	println!("  Written to: force_register_call.hex");

	if let Some(ref fr) = output.force_reserve_info {
		let mut fr_hex = "0x".to_owned();
		fr_hex.push_str(&hex::encode(&fr.encoded));
		fs::write("force_reserve_call.hex", &fr_hex).expect("write force_reserve_call.hex");
		println!("  force_reserve call written to: force_reserve_call.hex ({} bytes)", fr.length);
	}

	println!("\nWhitelist+dispatch wrapped in XCM batch:");
	println!("  Proposal size: {} bytes", output.proposal.length);

	let fname = args.filename.unwrap_or_else(|| {
		format!("register-system-para-{}.call", args.para_id)
	});

	let mut proposal_hex = "0x".to_owned();
	proposal_hex.push_str(&hex::encode(&output.proposal.encoded));
	fs::write(&fname, &proposal_hex).expect("write proposal file");

	// Output summary
	println!("\n{}", "=".repeat(60));
	println!("  REGISTER SYSTEM PARACHAIN {}", args.para_id);
	println!("{}", "=".repeat(60));
	println!();
	println!("  STEP 1 — Submit on Relay Chain (permissionless):");
	println!(
		"    Use force_register_call.hex ({} bytes) as the bytes for",
		output.force_register_info.length
	);
	println!("    Preimage.note_preimage in any wallet (e.g. Polkadot JS Apps).");
	println!();
	println!("  STEP 2 — Submit as referendum:");
	println!("    opengov-cli submit-referendum \\");
	println!("      --proposal \"{}\" \\", fname);
	println!("      --network \"polkadot\" --track whitelistedcaller");
	println!("{}", "=".repeat(60));
}
