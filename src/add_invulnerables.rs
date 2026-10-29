use crate::*;
use clap::Parser as ClapParser;
use sp_core::crypto::{AccountId32 as SpAccountId32, Ss58Codec};

/// Generate a proposal that adds invulnerable collators to a system chain.
///
/// The generated proposal is a call to be executed on the network's Asset Hub, where OpenGov
/// lives. For any target chain other than Asset Hub itself, the `collatorSelection` call is
/// wrapped in an XCM `Transact` with `OriginKind::Xcm`, so that when the referendum passes on the
/// `StakingAdmin` track, it arrives on the target chain as the StakingAdmin plurality voice, which
/// every system chain's `CollatorSelectionUpdateOrigin` accepts.
#[derive(Debug, ClapParser)]
pub(crate) struct AddInvulnerablesArgs {
	/// The chain whose collator set should be extended, e.g. `bulletin-polkadot` or
	/// `bridge-hub-kusama`.
	#[clap(long = "network", short)]
	pub(crate) network: String,

	/// Account(s) to add as invulnerable collators, as SS58 addresses or 0x-prefixed 32-byte hex.
	/// Can be passed multiple times and/or comma-separated. Each account must already have
	/// registered its session keys on the target chain (`session.setKeys`), otherwise
	/// `addInvulnerable` will fail with `ValidatorNotRegistered` on enactment.
	#[clap(long = "who", required = true, num_args = 1.., value_delimiter = ',')]
	pub(crate) who: Vec<String>,

	/// Name of the file to which to write the output. If not provided, a default will be
	/// constructed.
	#[clap(long = "filename")]
	pub(crate) filename: Option<String>,

	/// Optional: a WebSocket RPC URL of the target chain, e.g. `wss://bulletin-rpc.polkadot.io`.
	/// When provided, the tool connects to it and verifies that the proposal can actually enact:
	/// the node must serve the target chain, every account must have session keys registered
	/// (`session.nextKeys`), no account may already be invulnerable, and the resulting set must
	/// fit `MaxInvulnerables`. Nothing is written if any check fails.
	#[clap(long = "verify-via")]
	pub(crate) verify_via: Option<String>,
}

// The sub-command's "main" function.
pub(crate) async fn add_invulnerables(prefs: AddInvulnerablesArgs) {
	let network = parse_network(&prefs.network);
	let accounts = parse_accounts(&prefs.who, &network);
	if let Some(url) = &prefs.verify_via {
		verify_on_chain(url, &network, &accounts).await;
	}
	let target_call = build_add_invulnerables_call(&network, &accounts);
	let proposal = wrap_for_governance(&target_call);
	write_output(&prefs, &network, &target_call, &proposal);
}

// Map the user's chain name to a `Network`. Only chains that use `pallet-collator-selection` with
// the StakingAdmin-capable `UpdateOrigin` are supported.
pub(crate) fn parse_network(input: &str) -> Network {
	use Network::*;
	match input.to_ascii_lowercase().as_str() {
		"asset-hub-polkadot" | "polkadot-asset-hub" => PolkadotAssetHub,
		"bridge-hub-polkadot" | "polkadot-bridge-hub" => PolkadotBridgeHub,
		"collectives-polkadot" | "polkadot-collectives" => PolkadotCollectives,
		"people-polkadot" | "polkadot-people" => PolkadotPeople,
		"coretime-polkadot" | "polkadot-coretime" => PolkadotCoretime,
		"bulletin-polkadot" | "polkadot-bulletin" => PolkadotBulletin,
		"asset-hub-kusama" | "kusama-asset-hub" => KusamaAssetHub,
		"bridge-hub-kusama" | "kusama-bridge-hub" => KusamaBridgeHub,
		"people-kusama" | "kusama-people" => KusamaPeople,
		"coretime-kusama" | "kusama-coretime" => KusamaCoretime,
		"encointer-kusama" | "kusama-encointer" => panic!(
			"Encointer's collator selection `UpdateOrigin` is its own council \
			 (`MoreThanHalfCouncil`), not OpenGov's StakingAdmin, so this tool cannot produce a \
			 referendum for it."
		),
		"polkadot" | "kusama" => panic!(
			"Relay chains do not have collator selection. Pass a system parachain, e.g. \
			 `bulletin-polkadot`."
		),
		_ => panic!(
			"Unknown network. Supported: `asset-hub-`, `bridge-hub-`, `collectives-`, `people-`, \
			 `coretime-` and `bulletin-` suffixed with `polkadot`, or `asset-hub-`, \
			 `bridge-hub-`, `people-` and `coretime-` suffixed with `kusama`."
		),
	}
}

// Parse the user-provided accounts (SS58 or hex) into raw public keys. Rejects duplicates and
// warns when an address does not use the network's SS58 prefix.
pub(crate) fn parse_accounts(inputs: &[String], network: &Network) -> Vec<[u8; 32]> {
	let expected_prefix = network.ss58_prefix();
	let mut accounts = Vec::new();
	for input in inputs {
		let input = input.trim();
		let account: [u8; 32] = if let Some(h) = input.strip_prefix("0x") {
			let bytes = hex::decode(h).unwrap_or_else(|_| panic!("`{input}` is not valid hex"));
			bytes.try_into().unwrap_or_else(|_| panic!("`{input}` must be exactly 32 bytes"))
		} else {
			let (account, format) = SpAccountId32::from_ss58check_with_version(input)
				.unwrap_or_else(|e| panic!("`{input}` is not a valid SS58 address: {e:?}"));
			let prefix = u16::from(format);
			if prefix != expected_prefix {
				println!(
					"Warning: `{input}` uses SS58 prefix {prefix}, expected {expected_prefix} \
					 for this network. Proceeding with the decoded public key."
				);
			}
			*<SpAccountId32 as AsRef<[u8; 32]>>::as_ref(&account)
		};
		if accounts.contains(&account) {
			panic!("`{input}` is listed more than once");
		}
		accounts.push(account);
	}
	accounts
}

// Construct the call to execute on the target chain: one `collatorSelection.addInvulnerable` per
// account, wrapped in `utility.forceBatch` if there is more than one so that a single failing
// addition (e.g. missing session keys) does not revert the others.
macro_rules! add_invulnerables_call {
	($module:ident, $call_ty:ident, $network_variant:ident, $accounts:expr) => {{
		use $module::runtime_types::{
			pallet_collator_selection::pallet::Call as CollatorSelectionCall,
			pallet_utility::pallet::Call as UtilityCall,
		};
		let mut calls: Vec<$call_ty> = $accounts
			.iter()
			.map(|who| {
				$call_ty::CollatorSelection(CollatorSelectionCall::add_invulnerable {
					who: subxt::utils::AccountId32(*who),
				})
			})
			.collect();
		let call = if calls.len() == 1 {
			calls.remove(0)
		} else {
			$call_ty::Utility(UtilityCall::force_batch { calls })
		};
		CallInfo::from_runtime_call(NetworkRuntimeCall::$network_variant(call))
	}};
}

pub(crate) fn build_add_invulnerables_call(network: &Network, accounts: &[[u8; 32]]) -> CallInfo {
	assert!(!accounts.is_empty(), "no accounts provided");
	match network {
		Network::PolkadotAssetHub => add_invulnerables_call!(
			polkadot_asset_hub,
			PolkadotAssetHubRuntimeCall,
			PolkadotAssetHub,
			accounts
		),
		Network::PolkadotBridgeHub => add_invulnerables_call!(
			polkadot_bridge_hub,
			PolkadotBridgeHubRuntimeCall,
			PolkadotBridgeHub,
			accounts
		),
		Network::PolkadotCollectives => add_invulnerables_call!(
			polkadot_collectives,
			CollectivesRuntimeCall,
			PolkadotCollectives,
			accounts
		),
		Network::PolkadotPeople => add_invulnerables_call!(
			polkadot_people,
			PolkadotPeopleRuntimeCall,
			PolkadotPeople,
			accounts
		),
		Network::PolkadotCoretime => add_invulnerables_call!(
			polkadot_coretime,
			PolkadotCoretimeRuntimeCall,
			PolkadotCoretime,
			accounts
		),
		Network::PolkadotBulletin => add_invulnerables_call!(
			polkadot_bulletin,
			PolkadotBulletinRuntimeCall,
			PolkadotBulletin,
			accounts
		),
		Network::KusamaAssetHub => add_invulnerables_call!(
			kusama_asset_hub,
			KusamaAssetHubRuntimeCall,
			KusamaAssetHub,
			accounts
		),
		Network::KusamaBridgeHub => add_invulnerables_call!(
			kusama_bridge_hub,
			KusamaBridgeHubRuntimeCall,
			KusamaBridgeHub,
			accounts
		),
		Network::KusamaPeople =>
			add_invulnerables_call!(kusama_people, KusamaPeopleRuntimeCall, KusamaPeople, accounts),
		Network::KusamaCoretime => add_invulnerables_call!(
			kusama_coretime,
			KusamaCoretimeRuntimeCall,
			KusamaCoretime,
			accounts
		),
		Network::Polkadot | Network::Kusama | Network::KusamaEncointer =>
			panic!("not supported for collator selection referenda"),
	}
}

// Connect to the target chain over RPC and verify that the proposal can actually enact. Uses the
// dynamic subxt API so that it works against whichever supported chain the URL points to, with
// whatever runtime it is currently running. Panics (before anything is written) if any check
// fails:
// - the node must serve the target chain (checked via `parachainInfo.parachainId`),
// - every account must have session keys registered (`session.nextKeys`), which is what
//   `addInvulnerable` requires via its `ValidatorRegistration` check,
// - no account may already be in `collatorSelection.invulnerables`,
// - the resulting set must not exceed the `MaxInvulnerables` constant.
async fn verify_on_chain(url: &str, network: &Network, accounts: &[[u8; 32]]) {
	use parity_scale_codec::Decode;
	use sp_core::crypto::Ss58AddressFormat;

	println!("\nVerifying accounts on-chain via {url}");
	let client = subxt::OnlineClient::<subxt::PolkadotConfig>::from_url(url)
		.await
		.unwrap_or_else(|e| panic!("could not connect to `{url}`: {e}"));
	let storage = client.storage().at_latest().await.expect("should fetch latest block state");

	// Make sure the node actually serves the chain we are targeting.
	let expected_para_id = network.get_para_id().expect("target must be a parachain");
	let para_id_query = subxt::dynamic::storage(
		"ParachainInfo",
		"ParachainId",
		Vec::<subxt::dynamic::Value>::new(),
	);
	let para_id_bytes = storage
		.fetch(&para_id_query)
		.await
		.expect("should query ParachainInfo.ParachainId")
		.expect("every system chain sets its para id")
		.into_encoded();
	let para_id = u32::decode(&mut &para_id_bytes[..]).expect("para id should decode");
	assert!(
		para_id == expected_para_id,
		"`{url}` serves para {para_id}, but the target network has para id {expected_para_id}. \
		 Pass an RPC endpoint of the target chain."
	);

	let invulnerables_query = subxt::dynamic::storage(
		"CollatorSelection",
		"Invulnerables",
		Vec::<subxt::dynamic::Value>::new(),
	);
	let invulnerables: Vec<[u8; 32]> = match storage
		.fetch(&invulnerables_query)
		.await
		.expect("should query CollatorSelection.Invulnerables")
	{
		Some(thunk) => Vec::<subxt::utils::AccountId32>::decode(&mut &thunk.into_encoded()[..])
			.expect("invulnerables should decode")
			.into_iter()
			.map(|account| account.0)
			.collect(),
		None => Vec::new(),
	};

	// The per-account key lookups are independent reads of the same storage snapshot, so issue
	// them concurrently.
	let next_keys = futures::future::join_all(accounts.iter().map(|account| {
		let storage = &storage;
		async move {
			let next_keys_query = subxt::dynamic::storage(
				"Session",
				"NextKeys",
				vec![subxt::dynamic::Value::from_bytes(*account)],
			);
			storage.fetch(&next_keys_query).await.expect("should query Session.NextKeys")
		}
	}))
	.await;

	let prefix = Ss58AddressFormat::custom(network.ss58_prefix());
	let mut failures = 0;
	for (account, keys) in accounts.iter().zip(next_keys) {
		let display = SpAccountId32::new(*account).to_ss58check_with_version(prefix);
		match keys {
			Some(keys) =>
				println!("  ok: {display} has session keys 0x{}", hex::encode(keys.into_encoded())),
			None => {
				println!(
					"  FAILED: {display} has no session keys registered (`session.setKeys` \
					 missing)"
				);
				failures += 1;
			},
		}
		if invulnerables.contains(account) {
			println!("  FAILED: {display} is already invulnerable");
			failures += 1;
		}
	}

	let max_invulnerables = client
		.constants()
		.at(&subxt::dynamic::constant("CollatorSelection", "MaxInvulnerables"))
		.ok()
		.and_then(|value| u32::decode(&mut &value.into_encoded()[..]).ok());
	if let Some(max) = max_invulnerables {
		let resulting = invulnerables.len() + accounts.len();
		println!(
			"  invulnerables: {} on chain, {resulting} after this proposal, maximum {max}",
			invulnerables.len()
		);
		if resulting as u32 > max {
			println!("  FAILED: the resulting invulnerable set would exceed `MaxInvulnerables`");
			failures += 1;
		}
	}

	assert!(failures == 0, "on-chain verification failed with {failures} error(s), see above");
	println!("All accounts verified.");
}

// Wrap the target-chain call into the proposal to be voted on the governance chain (Asset Hub).
// Asset Hub's own `CollatorSelectionUpdateOrigin` accepts the local StakingAdmin origin, so for
// Asset Hub the proposal is the call itself; for every other chain it is an XCM `send` to the
// target with `Transact { origin_kind: Xcm }`, which the StakingAdmin track's origin turns into
// the `(AssetHub, Plurality { id: Defense, part: Voice })` location that the target's
// `CollatorSelectionUpdateOrigin` accepts.
pub(crate) fn wrap_for_governance(target: &CallInfo) -> CallInfo {
	use Network::*;
	match target.network {
		PolkadotAssetHub | KusamaAssetHub => target.clone(),
		PolkadotBridgeHub | PolkadotCollectives | PolkadotPeople | PolkadotCoretime
		| PolkadotBulletin => {
			use polkadot_asset_hub::runtime_types::xcm::v3::OriginKind;
			CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
				send_from_polkadot_asset_hub(target, OriginKind::Xcm),
			))
		},
		KusamaBridgeHub | KusamaPeople | KusamaCoretime => {
			use kusama_asset_hub::runtime_types::xcm::v3::OriginKind;
			CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
				send_from_kusama_asset_hub(target, OriginKind::Xcm),
			))
		},
		Polkadot | Kusama | KusamaEncointer => panic!("no governance wrapping for this network"),
	}
}

// Write the proposal to disk and tell the user how to put it to referendum.
fn write_output(
	prefs: &AddInvulnerablesArgs,
	network: &Network,
	target_call: &CallInfo,
	proposal: &CallInfo,
) {
	let network_name = prefs.network.to_ascii_lowercase();
	println!("\nCall to execute on {network_name}: 0x{}", hex::encode(&target_call.encoded));
	println!("Its hash: 0x{}", hex::encode(target_call.hash));

	if prefs.verify_via.is_some() {
		println!(
			"\nAll accounts were verified on-chain: session keys registered and not already \
			 invulnerable. Re-verify shortly before enactment; keys can be purged in the \
			 meantime."
		);
	} else {
		println!(
			"\nReminder: every account must already have session keys registered on \
			 {network_name} (`session.setKeys`) by the time the referendum enacts, or its \
			 `addInvulnerable` will fail with `ValidatorNotRegistered`. Pass `--verify-via \
			 <WSS_URL>` to check this on-chain."
		);
	}

	let fname = if let Some(user_filename) = &prefs.filename {
		user_filename.clone()
	} else {
		format!("./add-invulnerables-{network_name}.call")
	};
	write_call_data(&fname, &proposal.encoded);

	println!("\nSuccess! The proposal was written to {fname}");
	println!("To submit this as a referendum in OpenGov, run:");
	let relay = if network.is_kusama() { "kusama" } else { "polkadot" };
	println!("\nopengov-cli submit-referendum \\");
	println!("    --proposal \"{fname}\" \\");
	println!("    --network \"{relay}\" --track staking-admin");
}
