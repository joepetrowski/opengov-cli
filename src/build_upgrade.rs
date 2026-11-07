use crate::*;
use clap::Parser as ClapParser;
use std::fs;
use std::path::Path;

/// Generate a single call that will upgrade all system chains in a given network.
#[derive(Debug, ClapParser)]
pub(crate) struct UpgradeArgs {
	/// Network on which to submit the referendum. `polkadot` or `kusama`.
	#[clap(long = "network", short)]
	pub(crate) network: String,

	/// Only include the runtimes explicitly specified.
	#[clap(long = "only")]
	pub(crate) only: bool,

	/// Use local WASM files instead of downloading from GitHub. Files are assumed to already be in
	/// the upgrade directory.
	#[clap(long = "local")]
	pub(crate) local: bool,

	/// Skip sanity checks on the downloaded runtime blobs (e.g. their file size).
	#[clap(long = "no-runtime-checks")]
	pub(crate) no_runtime_checks: bool,

	/// The Fellowship release version. Should be semver and correspond to the release published.
	#[clap(long = "relay-version")]
	pub(crate) relay_version: Option<String>,

	/// Optional. The runtime version of Asset Hub to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "asset-hub")]
	pub(crate) asset_hub: Option<String>,

	/// Optional. The runtime version of Bridge Hub to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "bridge-hub")]
	pub(crate) bridge_hub: Option<String>,

	/// Optional. The runtime version of Collectives to which to upgrade. If not provided, it will
	/// use the Relay Chain's version.
	#[clap(long = "collectives")]
	pub(crate) collectives: Option<String>,

	/// Optional. The runtime version of Encointer to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "encointer")]
	pub(crate) encointer: Option<String>,

	/// Optional. The runtime version of People to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "people")]
	pub(crate) people: Option<String>,

	/// Optional. The runtime version of Coretime to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "coretime")]
	pub(crate) coretime: Option<String>,

	/// Optional. The runtime version of Bulletin to which to upgrade. If not provided, it will use
	/// the Relay Chain's version. Only applicable to Polkadot.
	#[clap(long = "bulletin")]
	pub(crate) bulletin: Option<String>,

	/// Name of the file to which to write the output. If not provided, a default will be
	/// constructed.
	#[clap(long = "filename")]
	pub(crate) filename: Option<String>,

	/// Some additional call that you want executed on the Relay Chain along with the upgrade.
	#[clap(long = "additional")]
	pub(crate) additional: Option<String>,
}

// The sub-command's "main" function.
pub(crate) async fn build_upgrade(prefs: UpgradeArgs) {
	// 0. Find out what to do.
	let use_local = prefs.local;
	let upgrade_details = parse_inputs(prefs);

	// 1. Download all the Wasm files needed from the release pages (unless using local files).
	if use_local {
		println!("\nUsing local WASM files from {}\n", upgrade_details.directory);
	} else {
		download_runtimes(&upgrade_details).await;
	}

	// 2. Construct the `authorize_upgrade` call on each chain.
	let authorization_calls = generate_authorize_upgrade_calls(&upgrade_details);

	// 3. Construct a `force_batch` call with everything.
	let batch = construct_batch(&upgrade_details, authorization_calls).await;

	// 4. Write this call as a file that can then be passed to `submit_referendum`.
	write_batch(&upgrade_details, batch);
}

fn chain_version(chain: Option<String>, default: Option<String>, only: bool) -> Option<String> {
	// if the user specified a version for this particular chain, use it
	if let Some(v) = chain {
		Some(String::from(v.trim_start_matches('v')))
	} else {
		// if the user only wants to upgrade specific chains, and have not specified this one, then
		// return None so that it will not be added to the batch of upgrades
		if only {
			None
		// otherwise, use the default version
		} else {
			assert!(default.is_some(), "no version specified");
			default
		}
	}
}

// Parse the CLI inputs and return a typed struct with all the details needed.
pub(crate) fn parse_inputs(prefs: UpgradeArgs) -> UpgradeDetails {
	let mut networks = Vec::new();
	let only = prefs.only;

	if !only {
		assert!(
			prefs.relay_version.is_some(),
			"relay-version must be specified unless using --only"
		);
	}
	let relay_version = chain_version(prefs.relay_version, None, only);
	let asset_hub_version = chain_version(prefs.asset_hub, relay_version.clone(), only);
	let bridge_hub_version = chain_version(prefs.bridge_hub, relay_version.clone(), only);
	let people_version = chain_version(prefs.people, relay_version.clone(), only);
	let coretime_version = chain_version(prefs.coretime, relay_version.clone(), only);
	let encointer_version = chain_version(prefs.encointer, relay_version.clone(), only);
	let collectives_version = chain_version(prefs.collectives, relay_version.clone(), only);
	let bulletin_version = chain_version(prefs.bulletin, relay_version.clone(), only);

	let relay = match prefs.network.to_ascii_lowercase().as_str() {
		"polkadot" => {
			if let Some(v) = relay_version.clone() {
				networks.push(VersionedNetwork { network: Network::Polkadot, version: v });
			}
			if let Some(v) = asset_hub_version.clone() {
				networks.push(VersionedNetwork { network: Network::PolkadotAssetHub, version: v });
			}
			if let Some(v) = collectives_version.clone() {
				networks
					.push(VersionedNetwork { network: Network::PolkadotCollectives, version: v });
			}
			if let Some(v) = bridge_hub_version.clone() {
				networks.push(VersionedNetwork { network: Network::PolkadotBridgeHub, version: v });
			}
			if let Some(v) = people_version.clone() {
				networks.push(VersionedNetwork { network: Network::PolkadotPeople, version: v });
			}
			if let Some(v) = coretime_version.clone() {
				networks.push(VersionedNetwork { network: Network::PolkadotCoretime, version: v });
			}
			if let Some(v) = bulletin_version.clone() {
				networks.push(VersionedNetwork { network: Network::PolkadotBulletin, version: v });
			}
			Network::Polkadot
		},
		"kusama" => {
			if let Some(v) = relay_version.clone() {
				networks.push(VersionedNetwork { network: Network::Kusama, version: v });
			}
			if let Some(v) = asset_hub_version.clone() {
				networks.push(VersionedNetwork { network: Network::KusamaAssetHub, version: v });
			}
			if let Some(v) = encointer_version.clone() {
				networks.push(VersionedNetwork { network: Network::KusamaEncointer, version: v });
			}
			if let Some(v) = bridge_hub_version.clone() {
				networks.push(VersionedNetwork { network: Network::KusamaBridgeHub, version: v });
			}
			if let Some(v) = people_version.clone() {
				networks.push(VersionedNetwork { network: Network::KusamaPeople, version: v });
			}
			if let Some(v) = coretime_version.clone() {
				networks.push(VersionedNetwork { network: Network::KusamaCoretime, version: v });
			}
			Network::Kusama
		},
		_ => panic!("`network` must be `polkadot` or `kusama`"),
	};

	let additional = match prefs.additional {
		Some(c) => {
			let additional_bytes = get_proposal_bytes(c.clone());
			match relay {
				// This match isn't as intuitive post-ahm, as these are AH calls.
				Network::Polkadot =>
					Some(CallInfo::from_bytes(&additional_bytes, Network::PolkadotAssetHub)),
				Network::Kusama =>
					Some(CallInfo::from_bytes(&additional_bytes, Network::KusamaAssetHub)),
				_ => panic!("`network` must be `polkadot` or `kusama`"),
			}
		},
		None => None,
	};

	let no_runtime_checks = prefs.no_runtime_checks;

	// Get a version from one of the args. (This still feels dirty.)
	let version =
		relay_version.clone().unwrap_or(asset_hub_version.unwrap_or(bridge_hub_version.unwrap_or(
			encointer_version.unwrap_or(collectives_version.unwrap_or(coretime_version.unwrap_or(
				people_version.unwrap_or(bulletin_version.unwrap_or(String::from("no-version"))),
			))),
		)));

	// Set up a directory to store information fetched/written during this program.
	let directory = format!("./upgrade-{}-{}/", prefs.network, version);
	let output_file = if let Some(user_filename) = prefs.filename {
		format!("{directory}{user_filename}")
	} else {
		let network = &prefs.network;
		format!("{directory}{network}-{version}.call")
	};

	make_version_directory(directory.as_str());

	UpgradeDetails { relay, networks, directory, output_file, additional, no_runtime_checks }
}

// Create a directory into which to place runtime blobs and the final call data.
fn make_version_directory(dir_name: &str) {
	if !Path::new(dir_name).is_dir() {
		fs::create_dir_all(dir_name).expect("it makes a dir");
	}
}

// Convert a semver version (e.g. "1.2.3") to an integer runtime version (e.g. 1002003).
pub(crate) fn semver_to_intver(semver: &str) -> String {
	// M.m.p => M_mmm_ppp
	let points =
		semver.bytes().enumerate().filter(|(_, b)| *b == b'.').map(|(i, _)| i).collect::<Vec<_>>();

	assert!(points.len() == 2, "not semver");

	let major = &semver[..points[0]];
	let minor = &semver[points[0] + 1..points[1]];
	let patch = &semver[points[1] + 1..];

	format!("{major}{minor:0>3}{patch:0>3}")
}

// Fetch all the runtime Wasm blobs from a Fellowship release.
async fn download_runtimes(upgrade_details: &UpgradeDetails) {
	// Relay Form
	// https://github.com/polkadot-fellows/runtimes/releases/download/v1.0.0/polkadot_runtime-v1000000.compact.compressed.wasm
	//
	// Parachains Form
	// https://github.com/polkadot-fellows/runtimes/releases/download/v1.0.0/asset_hub_kusama_runtime-v1000000.compact.compressed.wasm

	println!("\nDownloading runtimes.\n");
	for chain in &upgrade_details.networks {
		let fname = runtime_file_name(chain);

		let version = &chain.version;
		let download_url = format!(
			"https://github.com/polkadot-fellows/runtimes/releases/download/v{version}/{fname}"
		);

		let download_url = download_url.as_str();
		let directory = &upgrade_details.directory;
		let path_name = format!("{directory}{fname}");
		println!("Downloading... {fname}");
		let response = reqwest::get(download_url).await.expect("we need files to work");

		let status = response.status();
		assert!(status.is_success(), "Failed to download {}: HTTP {}", fname, status);

		let runtime = response.bytes().await.expect("need bytes");

		if !upgrade_details.no_runtime_checks {
			// Substrate zstd-compressed blob magic (`sp_maybe_compressed_blob::ZSTD_PREFIX`).
			const ZSTD_PREFIX: [u8; 8] = [82, 188, 83, 118, 70, 219, 142, 5];
			assert!(
				runtime.starts_with(&ZSTD_PREFIX),
				"Downloaded {} is not a zstd-compressed Substrate runtime blob. \
				 Pass --no-runtime-checks to skip.",
				fname,
			);

			let size = runtime.len();
			assert!(
				(100 * 1024..=10 * 1024 * 1024).contains(&size),
				"Downloaded {} is {} bytes; expected between 100 KiB and 10 MiB. \
				 Pass --no-runtime-checks to skip.",
				fname,
				size,
			);
		}

		// todo: we could actually just hash the file, mutate UpgradeDetails, and not write it.
		// saving it may be more convenient anyway though, since someone needs to upload it after
		// the referendum enacts.
		fs::write(path_name, runtime).expect("we can write");
	}
}

// The name a Fellowship release gives a chain's runtime, e.g. `asset-hub-polkadot`.
pub(crate) fn runtime_name(network: &Network) -> &'static str {
	match network {
		Network::Kusama => "kusama",
		Network::KusamaAssetHub => "asset-hub-kusama",
		Network::KusamaBridgeHub => "bridge-hub-kusama",
		Network::KusamaPeople => "people-kusama",
		Network::KusamaCoretime => "coretime-kusama",
		Network::KusamaEncointer => "encointer-kusama",
		Network::Polkadot => "polkadot",
		Network::PolkadotAssetHub => "asset-hub-polkadot",
		Network::PolkadotCollectives => "collectives-polkadot",
		Network::PolkadotBridgeHub => "bridge-hub-polkadot",
		Network::PolkadotPeople => "people-polkadot",
		Network::PolkadotCoretime => "coretime-polkadot",
		Network::PolkadotBulletin => "bulletin-polkadot",
	}
}

// The file name of a chain's runtime blob in a Fellowship release, e.g.
// `asset-hub-polkadot_runtime-v2005002.compact.compressed.wasm`.
pub(crate) fn runtime_file_name(chain: &VersionedNetwork) -> String {
	let name = runtime_name(&chain.network);
	let runtime_version = semver_to_intver(&chain.version);
	format!("{name}_runtime-v{runtime_version}.compact.compressed.wasm")
}

// The prefix used when logging a chain's runtime hash, padded so that the hashes line up.
fn runtime_hash_label(network: &Network) -> &'static str {
	match network {
		Network::Kusama => "Kusama Relay Chain Runtime Hash: ",
		Network::KusamaAssetHub => "Kusama Asset Hub Runtime Hash:   ",
		Network::KusamaBridgeHub => "Kusama Bridge Hub Runtime Hash:  ",
		Network::KusamaPeople => "Kusama People Runtime Hash:      ",
		Network::KusamaCoretime => "Kusama Coretime Runtime Hash:    ",
		Network::KusamaEncointer => "Kusama Encointer Runtime Hash:   ",
		Network::Polkadot => "Polkadot Relay Chain Runtime Hash: ",
		Network::PolkadotAssetHub => "Polkadot Asset Hub Runtime Hash:   ",
		Network::PolkadotCollectives => "Polkadot Collectives Runtime Hash: ",
		Network::PolkadotBridgeHub => "Polkadot Bridge Hub Runtime Hash:  ",
		Network::PolkadotPeople => "Polkadot People Runtime Hash:      ",
		Network::PolkadotCoretime => "Polkadot Coretime Runtime Hash:    ",
		Network::PolkadotBulletin => "Polkadot Bulletin Runtime Hash:    ",
	}
}

// Construct the `system.authorize_upgrade` call that will execute on a given chain.
pub(crate) fn authorize_upgrade_call(network: &Network, code_hash: [u8; 32]) -> CallInfo {
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

// Generate the `authorize_upgrade` calls that will need to execute on each chain.
fn generate_authorize_upgrade_calls(upgrade_details: &UpgradeDetails) -> Vec<CallInfo> {
	println!("\nGenerating parachain authorization calls. The runtime hashes are logged if you would like to verify them with srtool.\n");
	upgrade_details
		.networks
		.iter()
		.map(|chain| {
			let path = format!("{}{}", upgrade_details.directory, runtime_file_name(chain));
			let runtime = fs::read(path).expect("Should give a valid file path");
			let runtime_hash = blake2_256(&runtime);
			println!("{}0x{}", runtime_hash_label(&chain.network), hex::encode(runtime_hash));
			authorize_upgrade_call(&chain.network, runtime_hash)
		})
		.collect()
}

// Take the parachain authorization calls and the Relay Chain call, and batch them into one call
// that can be executed on the Relay Chain. The call returned here is the proposal to put to
// referendum.
pub(crate) async fn construct_batch(
	upgrade_details: &UpgradeDetails,
	calls: Vec<CallInfo>,
) -> CallInfo {
	println!("\nBatching calls.");
	match upgrade_details.relay {
		Network::Kusama => construct_kusama_batch(calls, upgrade_details.additional.clone()).await,
		Network::Polkadot =>
			construct_polkadot_batch(calls, upgrade_details.additional.clone()).await,
		_ => panic!("Not a Relay Chain"),
	}
}

// Construct the batch needed on Kusama.
async fn construct_kusama_batch(calls: Vec<CallInfo>, additional: Option<CallInfo>) -> CallInfo {
	use kusama_asset_hub::runtime_types::{
		pallet_utility::pallet::Call as UtilityCall, xcm::v3::OriginKind,
	};

	let mut batch_calls = Vec::new();
	for auth in calls {
		if matches!(auth.network, Network::KusamaAssetHub) {
			batch_calls.push(auth.get_kusama_asset_hub_call().expect("We just constructed this"));
		} else {
			let send_auth = send_from_kusama_asset_hub(&auth, OriginKind::Superuser);
			batch_calls.push(send_auth);
		}
	}
	if let Some(a) = additional {
		batch_calls.push(a.get_kusama_asset_hub_call().expect("kusama call"))
	}
	match &batch_calls.len() {
		0 => panic!("no calls"),
		1 =>
			CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(batch_calls[0].clone())),
		_ => CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
			KusamaAssetHubRuntimeCall::Utility(UtilityCall::force_batch { calls: batch_calls }),
		)),
	}
}

// Construct the batch needed on Polkadot.
async fn construct_polkadot_batch(calls: Vec<CallInfo>, additional: Option<CallInfo>) -> CallInfo {
	use polkadot_asset_hub::runtime_types::{
		pallet_utility::pallet::Call as UtilityCall, xcm::v3::OriginKind,
	};

	let mut batch_calls = Vec::new();
	for auth in calls {
		if matches!(auth.network, Network::PolkadotAssetHub) {
			batch_calls.push(auth.get_polkadot_asset_hub_call().expect("We just constructed this"));
		} else {
			let send_auth = send_from_polkadot_asset_hub(&auth, OriginKind::Superuser);
			batch_calls.push(send_auth);
		}
	}
	if let Some(a) = additional {
		batch_calls.push(a.get_polkadot_asset_hub_call().expect("polkadot call"))
	}
	match &batch_calls.len() {
		0 => panic!("no calls"),
		1 => CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
			batch_calls[0].clone(),
		)),
		_ => CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
			PolkadotAssetHubRuntimeCall::Utility(UtilityCall::force_batch { calls: batch_calls }),
		)),
	}
}

// Write the call needed to disk and provide instructions to the user about how to propose it.
fn write_batch(upgrade_details: &UpgradeDetails, batch: CallInfo) {
	let fname = upgrade_details.output_file.as_str();
	write_call_data(fname, &batch.encoded);

	println!("\nSuccess! The call data was written to {fname}");
	println!("To submit this as a referendum in OpenGov, run:");
	let network = match upgrade_details.relay {
		Network::Kusama => "kusama",
		Network::Polkadot => "polkadot",
		_ => panic!("not a relay network"),
	};
	println!("\nopengov-cli submit-referendum \\");
	println!("    --proposal \"{fname}\" \\");
	println!("    --network \"{network}\" --track <\"root\" or \"whitelistedcaller\">");
}
