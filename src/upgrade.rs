//! Build the proposal that upgrades a relay chain and its system parachains.

use crate::*;

/// What to upgrade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeInput {
	/// The relay chain whose governance will enact the upgrade: Polkadot or Kusama.
	pub relay: Network,
	/// Each chain to upgrade and the hash of its new runtime, in the order they should appear in
	/// the batch.
	pub runtimes: Vec<(Network, [u8; 32])>,
	/// An additional, encoded Asset Hub call to execute in the same batch.
	pub additional: Option<Vec<u8>>,
}

/// Build the proposal: a `system.authorize_upgrade` for each chain, sent over XCM from Asset Hub
/// where needed, all in a `utility.force_batch`. The result is an Asset Hub call.
pub fn build_upgrade(input: UpgradeInput) -> Result<CallInfo, Error> {
	let asset_hub = asset_hub_of(&input.relay)?;
	let mut authorizations = Vec::with_capacity(input.runtimes.len());
	for (network, code_hash) in &input.runtimes {
		if network.is_kusama() != input.relay.is_kusama() {
			return Err(Error::NotInRelayFamily {
				relay: input.relay.clone(),
				network: network.clone(),
			});
		}
		authorizations.push(authorize_upgrade_call(network, *code_hash));
	}
	let additional = input.additional.map(|bytes| CallInfo::from_bytes(&bytes, asset_hub));
	batch_calls(&input.relay, authorizations, additional)
}

// The Asset Hub on which a relay chain's governance runs.
fn asset_hub_of(relay: &Network) -> Result<Network, Error> {
	match relay {
		Network::Polkadot => Ok(Network::PolkadotAssetHub),
		Network::Kusama => Ok(Network::KusamaAssetHub),
		_ => Err(Error::NotARelayChain),
	}
}

/// Convert a semver version (e.g. "1.2.3") to an integer runtime version (e.g. 1002003).
pub fn semver_to_intver(semver: &str) -> Result<String, Error> {
	// M.m.p => M_mmm_ppp
	let points =
		semver.bytes().enumerate().filter(|(_, b)| *b == b'.').map(|(i, _)| i).collect::<Vec<_>>();

	if points.len() != 2 {
		return Err(Error::NotSemver);
	}

	let major = &semver[..points[0]];
	let minor = &semver[points[0] + 1..points[1]];
	let patch = &semver[points[1] + 1..];

	Ok(format!("{major}{minor:0>3}{patch:0>3}"))
}

/// The file name of a chain's runtime blob in a Fellowship release, e.g.
/// `asset-hub-polkadot_runtime-v2005002.compact.compressed.wasm`. The `version` is the semver of
/// the release, with or without a leading `v`.
pub fn runtime_file_name(network: &Network, version: &str) -> Result<String, Error> {
	let name = network.runtime_name();
	let runtime_version = semver_to_intver(version.trim_start_matches('v'))?;
	Ok(format!("{name}_runtime-v{runtime_version}.compact.compressed.wasm"))
}

/// Check that `blob` looks like a compressed Substrate runtime (unless `skip_checks`), and return
/// its `blake2_256` hash, which is the code hash to authorize.
pub fn check_and_hash_runtime(blob: &[u8], skip_checks: bool) -> Result<[u8; 32], Error> {
	if !skip_checks {
		// Substrate zstd-compressed blob magic (`sp_maybe_compressed_blob::ZSTD_PREFIX`).
		const ZSTD_PREFIX: [u8; 8] = [82, 188, 83, 118, 70, 219, 142, 5];
		if !blob.starts_with(&ZSTD_PREFIX) {
			return Err(Error::RuntimeNotCompressed);
		}
		let size = blob.len();
		if !(100 * 1024..=10 * 1024 * 1024).contains(&size) {
			return Err(Error::RuntimeSize(size));
		}
	}
	Ok(blake2_256(blob))
}

/// Construct the `system.authorize_upgrade` call that will execute on a given chain.
pub fn authorize_upgrade_call(network: &Network, code_hash: [u8; 32]) -> CallInfo {
	macro_rules! authorize_upgrade {
		($module:ident, $call_ty:ident, $network_variant:ident) => {{
			use $module::runtime_types::frame_system::pallet::Call as SystemCall;
			CallInfo::from_runtime_call(NetworkRuntimeCall::$network_variant($call_ty::System(
				SystemCall::authorize_upgrade { code_hash: H256(code_hash) },
			)))
		}};
	}

	match network {
		Network::Kusama => authorize_upgrade!(kusama_relay, KusamaRuntimeCall, Kusama),
		Network::KusamaAssetHub =>
			authorize_upgrade!(kusama_asset_hub, KusamaAssetHubRuntimeCall, KusamaAssetHub),
		Network::KusamaBridgeHub =>
			authorize_upgrade!(kusama_bridge_hub, KusamaBridgeHubRuntimeCall, KusamaBridgeHub),
		Network::KusamaPeople =>
			authorize_upgrade!(kusama_people, KusamaPeopleRuntimeCall, KusamaPeople),
		Network::KusamaCoretime =>
			authorize_upgrade!(kusama_coretime, KusamaCoretimeRuntimeCall, KusamaCoretime),
		Network::KusamaEncointer =>
			authorize_upgrade!(kusama_encointer, KusamaEncointerRuntimeCall, KusamaEncointer),
		Network::Polkadot => authorize_upgrade!(polkadot_relay, PolkadotRuntimeCall, Polkadot),
		Network::PolkadotAssetHub =>
			authorize_upgrade!(polkadot_asset_hub, PolkadotAssetHubRuntimeCall, PolkadotAssetHub),
		Network::PolkadotCollectives =>
			authorize_upgrade!(polkadot_collectives, CollectivesRuntimeCall, PolkadotCollectives),
		Network::PolkadotBridgeHub =>
			authorize_upgrade!(polkadot_bridge_hub, PolkadotBridgeHubRuntimeCall, PolkadotBridgeHub),
		Network::PolkadotPeople =>
			authorize_upgrade!(polkadot_people, PolkadotPeopleRuntimeCall, PolkadotPeople),
		Network::PolkadotCoretime =>
			authorize_upgrade!(polkadot_coretime, PolkadotCoretimeRuntimeCall, PolkadotCoretime),
		Network::PolkadotBulletin =>
			authorize_upgrade!(polkadot_bulletin, PolkadotBulletinRuntimeCall, PolkadotBulletin),
	}
}

// Take the authorization calls for each chain, and batch them into one call that can be executed
// on the relay chain's Asset Hub. The call returned here is the proposal to put to referendum.
pub(crate) fn batch_calls(
	relay: &Network,
	calls: Vec<CallInfo>,
	additional: Option<CallInfo>,
) -> Result<CallInfo, Error> {
	match relay {
		Network::Kusama => construct_kusama_batch(calls, additional),
		Network::Polkadot => construct_polkadot_batch(calls, additional),
		_ => Err(Error::NotARelayChain),
	}
}

// Construct the batch needed on Kusama.
fn construct_kusama_batch(
	calls: Vec<CallInfo>,
	additional: Option<CallInfo>,
) -> Result<CallInfo, Error> {
	use kusama_asset_hub::runtime_types::{
		pallet_utility::pallet::Call as UtilityCall, xcm::v3::OriginKind,
	};

	let mut batch_calls = Vec::new();
	for auth in calls {
		if matches!(auth.network, Network::KusamaAssetHub) {
			batch_calls.push(auth.get_kusama_asset_hub_call()?);
		} else {
			let send_auth = send_from_kusama_asset_hub(&auth, OriginKind::Superuser);
			batch_calls.push(send_auth);
		}
	}
	if let Some(a) = additional {
		batch_calls.push(a.get_kusama_asset_hub_call()?)
	}
	match batch_calls.len() {
		0 => Err(Error::NoCalls),
		1 => Ok(CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
			batch_calls.remove(0),
		))),
		_ => Ok(CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
			KusamaAssetHubRuntimeCall::Utility(UtilityCall::force_batch { calls: batch_calls }),
		))),
	}
}

// Construct the batch needed on Polkadot.
fn construct_polkadot_batch(
	calls: Vec<CallInfo>,
	additional: Option<CallInfo>,
) -> Result<CallInfo, Error> {
	use polkadot_asset_hub::runtime_types::{
		pallet_utility::pallet::Call as UtilityCall, xcm::v3::OriginKind,
	};

	let mut batch_calls = Vec::new();
	for auth in calls {
		if matches!(auth.network, Network::PolkadotAssetHub) {
			batch_calls.push(auth.get_polkadot_asset_hub_call()?);
		} else {
			let send_auth = send_from_polkadot_asset_hub(&auth, OriginKind::Superuser);
			batch_calls.push(send_auth);
		}
	}
	if let Some(a) = additional {
		batch_calls.push(a.get_polkadot_asset_hub_call()?)
	}
	match batch_calls.len() {
		0 => Err(Error::NoCalls),
		1 => Ok(CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
			batch_calls.remove(0),
		))),
		_ => Ok(CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
			PolkadotAssetHubRuntimeCall::Utility(UtilityCall::force_batch { calls: batch_calls }),
		))),
	}
}
