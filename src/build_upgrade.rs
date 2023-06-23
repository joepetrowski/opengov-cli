use crate::*;
use clap::Parser as ClapParser;
use std::fs;
use std::path::Path;

#[derive(Debug, ClapParser)]
pub(crate) struct UpgradeArgs {
	/// Network on which to submit the referendum. `polkadot` or `kusama`.
	#[clap(long = "network", short)]
	network: String,

	/// The runtime version of the Relay Chain to which to upgrade. E.g. "9430" or "latest".
	#[clap(long = "relay-version")]
	relay_version: String,

	/// The runtime version of the system parachains to which to upgrade. E.g. "9430" or "latest".
	#[clap(long = "parachain-version")]
	parachain_version: String,

	/// Name of the file to which to write the output. If not provided, a default will be
	/// constructed.
	#[clap(long = "filename")]
	filename: Option<String>,

	/// Override the conversion of runtime version to semver. For example the release 9430 mapping
	/// to v0.9.43. Use this if the program fails to download the Relay Chain runtime.
	#[clap(long = "relay-semver")]
	relay_semver: Option<String>,
	// todo: add input for repo to not default to parity releases / wait for fellowship
}

struct UpgradeDetails {
	networks: Vec<VersionedNetwork>,
	directory: String,
	output_file: String,
	semver_override: Option<String>,
}

struct VersionedNetwork {
	network: Network,
	version: String,
}

pub(crate) async fn build_upgrade(prefs: UpgradeArgs) {
	// 0. Find out what to do.
	let upgrade_details = parse_inputs(prefs);

	// 1. Download all the Wasm files needed from the release pages.
	download_runtimes(&upgrade_details).await;
	//
	// 2. Construct the `authorize_upgrade` call on each parachain.
	//
	// 3. Call the runtime API of each parachain and get the needed `Transact` weight.
	//
	// 4. Construct the `utility.with_weight(system.set_code(..), ..)` call on the Relay Chain.
	//
	// 5. Construct a `force_batch` call with everything.
	//
	// 6. Write this call as a file that can then be passed to `submit_referendum`.
}

fn parse_inputs(prefs: UpgradeArgs) -> UpgradeDetails {
	let mut networks = Vec::new();
	let relay_version = String::from(prefs.relay_version.trim_start_matches("v"));
	let paras_version = String::from(prefs.parachain_version.trim_start_matches("v"));

	match prefs.network.to_ascii_lowercase().as_str() {
		"polkadot" => {
			// Relay must be first!
			networks.push(VersionedNetwork {
				network: Network::Polkadot,
				version: relay_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::PolkadotAssetHub,
				version: paras_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::PolkadotCollectives,
				version: paras_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::PolkadotBridgeHub,
				version: paras_version.clone(),
			});
		},
		"kusama" => {
			// Relay must be first!
			networks.push(VersionedNetwork {
				network: Network::Kusama,
				version: relay_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::KusamaAssetHub,
				version: paras_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::KusamaBridgeHub,
				version: paras_version.clone(),
			});
		},
		_ => panic!("`network` must be `polkadot` or `kusama`"),
	}

	let directory = format!("./upgrade-{}-{}/", &prefs.network, &relay_version);
	let output_file = if let Some(user_filename) = prefs.filename {
		format!("{}{}", directory, user_filename)
	} else {
		format!("{}{}-{}.call", directory, prefs.network, relay_version)
	};

	make_version_directory(directory.as_str());

	return UpgradeDetails { networks, directory, output_file, semver_override: prefs.relay_semver }
}

fn make_version_directory(dir_name: &str) {
	if !Path::new(dir_name).is_dir() {
		fs::create_dir(dir_name).expect("it makes a dir");
	}
}

async fn download_runtimes(upgrade_details: &UpgradeDetails) {
	// Relay Form
	// https://github.com/paritytech/polkadot/releases/download/v0.9.43/polkadot_runtime-v9430.compact.compressed.wasm
	// expect runtime version to be "9430" and correspond to "0.9.43"
	//
	// Parachains Form
	// https://github.com/paritytech/cumulus/releases/download/parachains-v9430/statemint_runtime-v9430.compact.compressed.wasm
	// https://github.com/paritytech/cumulus/releases/download/parachains-v9430/collectives-polkadot_runtime-v9430.compact.compressed.wasm
	// https://github.com/paritytech/cumulus/releases/download/parachains-v9430/bridge-hub-polkadot_runtime-v9430.compact.compressed.wasm

	for chain in &upgrade_details.networks {
		let chain_name = match chain.network {
			Network::Kusama => "kusama",
			Network::Polkadot => "polkadot",
			Network::KusamaAssetHub => "statemine", // grumble
			Network::KusamaBridgeHub => "bridge-hub-kusama",
			Network::PolkadotAssetHub => "statemint", // grumble
			Network::PolkadotCollectives => "collectives-polkadot",
			Network::PolkadotBridgeHub => "bridge-hub-polkadot",
		};
		let version = chain.version.trim_start_matches("v");
		let fname = format!("{}_runtime-v{}.compact.compressed.wasm", chain_name, version);
		let download_url = match chain.network {
			Network::Kusama | Network::Polkadot => {
				let semver = if let Some(sv) = upgrade_details.semver_override.clone() {
					sv
				} else {
					let mut chars = version.chars();
					let first = chars.next().unwrap(); // 9
					let second = chars.next().unwrap(); // 4
					let third = chars.next().unwrap(); // 3
					if chars.last() != Some('0') {
						println!("\n    You probably need to use `--relay-semver X.Y.Z` since this was not a normal release!\n")
					}
					format!("0.{}.{}{}", first, second, third)
				};
				let semver = semver.trim_start_matches("v");
				format!(
					"https://github.com/paritytech/polkadot/releases/download/v{}/{}",
					semver, fname
				)
			},
			_ => format!(
				"https://github.com/paritytech/cumulus/releases/download/parachains-v{}/{}",
				version, fname
			),
		};

		let download_url = download_url.as_str();
		let path_name = format!("{}/{}", upgrade_details.directory, fname);
		println!("Downloading... {}", fname.as_str());
		let response = reqwest::get(download_url).await.expect("we need files to work");
		let runtime = response.bytes().await.expect("need bytes");
		fs::write(path_name, runtime).expect("we can write");
	}
}
