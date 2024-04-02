use crate::*;
use clap::Parser as ClapParser;
use std::fs;
use std::path::Path;

/// Generate a single call that will upgrade a Relay Chain and all of its system parachains.
#[derive(Debug, ClapParser)]
pub(crate) struct UpgradeArgs {
	/// Network on which to submit the referendum. `polkadot` or `kusama`.
	#[clap(long = "network", short)]
	network: String,

	/// The Fellowship release version. Should be semver and correspond to the release published.
	#[clap(long = "relay-version")]
	relay_version: String,

	/// Optional. The runtime version of Asset Hub to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "asset-hub")]
	asset_hub: Option<String>,

	/// Optional. The runtime version of Bridge Hub to which to upgrade. If not provided, it will
	/// use the Relay Chain's version.
	#[clap(long = "bridge-hub")]
	bridge_hub: Option<String>,

	/// Optional. The runtime version of Collectives to which to upgrade. If not provided, it will
	/// use the Relay Chain's version.
	#[clap(long = "collectives")]
	collectives: Option<String>,

	/// Optional. The runtime version of People to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "people")]
	people: Option<String>,

	/// Optional. The runtime version of Coretime to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "coretime")]
	coretime: Option<String>,

	/// Optional. The runtime version of Encointer to which to upgrade. If not provided, it will use
	/// the Relay Chain's version.
	#[clap(long = "encointer")]
	encointer: Option<String>,

	/// Name of the file to which to write the output. If not provided, a default will be
	/// constructed.
	#[clap(long = "filename")]
	filename: Option<String>,

	/// Some additional call that you want executed on the Relay Chain along with the upgrade.
	#[clap(long = "additional")]
	additional: Option<String>,
}

// The sub-command's "main" function.
pub(crate) async fn build_upgrade(prefs: UpgradeArgs) {
	// 0. Find out what to do.
	let upgrade_details = parse_inputs(prefs);

	// 1. Download all the Wasm files needed from the release pages.
	download_runtimes(&upgrade_details).await;

	// 2. Construct the `authorize_upgrade` call on each parachain.
	let authorization_calls = generate_authorize_upgrade_calls(&upgrade_details);

	// 3. Construct the `utility.with_weight(system.set_code(..), ..)` call on the Relay Chain.
	let relay_upgrade = generate_relay_upgrade_call(&upgrade_details);

	// 4. Call the runtime API of each parachain and get the needed `Transact` weight.
	// 5. Construct a `force_batch` call with everything.
	let batch = construct_batch(&upgrade_details, relay_upgrade, authorization_calls).await;

	// 6. Write this call as a file that can then be passed to `submit_referendum`.
	write_batch(&upgrade_details, batch);
}

// Parse the CLI inputs and return a typed struct with all the details needed.
fn parse_inputs(prefs: UpgradeArgs) -> UpgradeDetails {
	let mut networks = Vec::new();
	let relay_version = String::from(prefs.relay_version.trim_start_matches('v'));
	let asset_hub_version = if let Some(v) = prefs.asset_hub {
		String::from(v.trim_start_matches('v'))
	} else {
		relay_version.clone()
	};
	let bridge_hub_version = if let Some(v) = prefs.bridge_hub {
		String::from(v.trim_start_matches('v'))
	} else {
		relay_version.clone()
	};
	let encointer_version = if let Some(v) = prefs.encointer {
		String::from(v.trim_start_matches('v'))
	} else {
		relay_version.clone()
	};
	let collectives_version = if let Some(v) = prefs.collectives {
		String::from(v.trim_start_matches('v'))
	} else {
		relay_version.clone()
	};
	let people_version = if let Some(v) = prefs.people {
		String::from(v.trim_start_matches('v'))
	} else {
		relay_version.clone()
	};
	let coretime_version = if let Some(v) = prefs.coretime {
		String::from(v.trim_start_matches('v'))
	} else {
		relay_version.clone()
	};

	let relay = match prefs.network.to_ascii_lowercase().as_str() {
		"polkadot" => {
			// Relay must be first!
			networks.push(VersionedNetwork {
				network: Network::Polkadot,
				version: relay_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::PolkadotAssetHub,
				version: asset_hub_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::PolkadotCollectives,
				version: collectives_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::PolkadotBridgeHub,
				version: bridge_hub_version.clone(),
			});
			VersionedNetwork { network: Network::Polkadot, version: relay_version.clone() }
		},
		"kusama" => {
			// Relay must be first!
			networks.push(VersionedNetwork {
				network: Network::Kusama,
				version: relay_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::KusamaAssetHub,
				version: asset_hub_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::KusamaEncointer,
				version: encointer_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::KusamaBridgeHub,
				version: bridge_hub_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::KusamaPeople,
				version: people_version.clone(),
			});
			networks.push(VersionedNetwork {
				network: Network::KusamaCoretime,
				version: coretime_version.clone(),
			});
			VersionedNetwork { network: Network::Kusama, version: relay_version.clone() }
		},
		_ => panic!("`network` must be `polkadot` or `kusama`"),
	};

	let additional = match prefs.additional {
		Some(c) => {
			let additional_bytes = get_proposal_bytes(c.clone());
			match relay.network {
				Network::Polkadot =>
					Some(CallInfo::from_bytes(&additional_bytes, Network::Polkadot)),
				Network::Kusama => Some(CallInfo::from_bytes(&additional_bytes, Network::Kusama)),
				// for now, only support additional on the relay chain
				_ => panic!("`network` must be `polkadot` or `kusama`"),
			}
		},
		None => None,
	};

	let directory = format!("./upgrade-{}-{}/", &prefs.network, &relay_version);
	let output_file = if let Some(user_filename) = prefs.filename {
		format!("{}{}", directory, user_filename)
	} else {
		format!("{}{}-{}.call", directory, prefs.network, relay_version)
	};

	make_version_directory(directory.as_str());

	UpgradeDetails { relay, networks, directory, output_file, additional }
}

// Create a directory into which to place runtime blobs and the final call data.
fn make_version_directory(dir_name: &str) {
	if !Path::new(dir_name).is_dir() {
		fs::create_dir(dir_name).expect("it makes a dir");
	}
}

// Convert a semver version (e.g. "1.2.3") to an integer runtime version (e.g. 1002003).
fn semver_to_intver(semver: &str) -> String {
	// M.m.p => M_mmm_ppp
	let points =
		semver.bytes().enumerate().filter(|(_, b)| *b == b'.').map(|(i, _)| i).collect::<Vec<_>>();

	assert!(points.len() == 2, "not semver");

	let major = &semver[..points[0]];
	let minor = &semver[points[0] + 1..points[1]];
	let patch = &semver[points[1] + 1..];

	format!("{}{:0>3}{:0>3}", major, minor, patch)
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
		let chain_name = match chain.network {
			Network::Kusama => "kusama",
			Network::Polkadot => "polkadot",
			Network::KusamaAssetHub => "asset-hub-kusama",
			Network::KusamaBridgeHub => "bridge-hub-kusama",
			Network::KusamaPeople => "people-kusama",
			Network::KusamaCoretime => "coretime-kusama",
			Network::KusamaEncointer => "encointer-kusama",
			Network::PolkadotAssetHub => "asset-hub-polkadot",
			Network::PolkadotCollectives => "collectives-polkadot",
			Network::PolkadotBridgeHub => "bridge-hub-polkadot",
		};
		let runtime_version = semver_to_intver(&chain.version);
		let fname = format!("{}_runtime-v{}.compact.compressed.wasm", chain_name, runtime_version);

		let download_url = format!(
			"https://github.com/polkadot-fellows/runtimes/releases/download/v{}/{}",
			&chain.version, fname
		);

		let download_url = download_url.as_str();
		let path_name = format!("{}{}", upgrade_details.directory, fname);
		println!("Downloading... {}", fname.as_str());
		let response = reqwest::get(download_url).await.expect("we need files to work");
		let runtime = response.bytes().await.expect("need bytes");
		// todo: we could actually just hash the file, mutate UpgradeDetails, and not write it.
		// saving it may be more convenient anyway though, since someone needs to upload it after
		// the referendum enacts.
		fs::write(path_name, runtime).expect("we can write");
	}
}

// Generate the `authorize_upgrade` calls that will need to execute on each parachain.
fn generate_authorize_upgrade_calls(upgrade_details: &UpgradeDetails) -> Vec<CallInfo> {
	println!("\nGenerating parachain authorization calls. The runtime hashes are logged if you would like to verify them with srtool.\n");
	let mut authorization_calls = Vec::new();
	for chain in &upgrade_details.networks {
		let runtime_version = semver_to_intver(&chain.version);
		match chain.network {
			Network::Kusama | Network::Polkadot => continue, // do relay chain separately
			Network::KusamaAssetHub => {
				use kusama_asset_hub::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}asset-hub-kusama_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Kusama Asset Hub Runtime Hash:   0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
					KusamaAssetHubRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
			Network::KusamaBridgeHub => {
				use kusama_bridge_hub::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}bridge-hub-kusama_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Kusama Bridge Hub Runtime Hash:  0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaBridgeHub(
					KusamaBridgeHubRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
			Network::KusamaPeople => {
				use kusama_people::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}people-kusama_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Kusama People Runtime Hash:  0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaPeople(
					KusamaPeopleRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
			Network::KusamaCoretime => {
				use kusama_coretime::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}coretime-kusama_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Kusama Coretime Runtime Hash:  0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaCoretime(
					KusamaPeopleRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
			Network::KusamaEncointer => {
				use kusama_encointer::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}encointer-kusama_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Kusama Encointer Runtime Hash:  0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaEncointer(
					KusamaPeopleRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
			Network::PolkadotAssetHub => {
				use polkadot_asset_hub::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}asset-hub-polkadot_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Polkadot Asset Hub Runtime Hash:   0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
					PolkadotAssetHubRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
			Network::PolkadotCollectives => {
				use polkadot_collectives::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}collectives-polkadot_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Polkadot Collectives Runtime Hash: 0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
					CollectivesRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
			Network::PolkadotBridgeHub => {
				use polkadot_bridge_hub::runtime_types::cumulus_pallet_parachain_system::pallet::Call;
				let path = format!(
					"{}bridge-hub-polkadot_runtime-v{}.compact.compressed.wasm",
					upgrade_details.directory, runtime_version
				);
				let runtime = fs::read(path).expect("Should give a valid file path");
				let runtime_hash = blake2_256(&runtime);
				println!("Polkadot Bridge Hub Runtime Hash:  0x{}", hex::encode(runtime_hash));

				let call = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotBridgeHub(
					PolkadotBridgeHubRuntimeCall::ParachainSystem(Call::authorize_upgrade {
						code_hash: H256(runtime_hash),
						check_version: true,
					}),
				));
				authorization_calls.push(call);
			},
		};
	}
	authorization_calls
}

// Generate the `system.set_code` call, wrapped in `utility.with_weight`, that will upgrade the
// Relay Chain.
fn generate_relay_upgrade_call(upgrade_details: &UpgradeDetails) -> CallInfo {
	println!("\nGenerating Relay Chain upgrade call. The runtime hash is logged if you would like to verify it with srtool.\n");
	let runtime_version = semver_to_intver(&upgrade_details.relay.version);
	match upgrade_details.relay.network {
		Network::Kusama => {
			use kusama_relay::runtime_types::frame_system::pallet::Call as SystemCall;

			let path = format!(
				"{}kusama_runtime-v{}.compact.compressed.wasm",
				upgrade_details.directory, runtime_version
			);
			let runtime = fs::read(path).expect("Should give a valid file path");
			let runtime_hash = blake2_256(&runtime);
			println!("Kusama Relay Chain Runtime Hash: 0x{}", hex::encode(runtime_hash));

			CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(KusamaRuntimeCall::System(
				SystemCall::set_code { code: runtime },
			)))
		},
		Network::Polkadot => {
			use polkadot_relay::runtime_types::frame_system::pallet::Call as SystemCall;

			let path = format!(
				"{}polkadot_runtime-v{}.compact.compressed.wasm",
				upgrade_details.directory, runtime_version
			);
			let runtime = fs::read(path).expect("Should give a valid file path");
			let runtime_hash = blake2_256(&runtime);
			println!("Polkadot Relay Chain Runtime Hash: 0x{}", hex::encode(runtime_hash));

			CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(PolkadotRuntimeCall::System(
				SystemCall::set_code { code: runtime },
			)))
		},
		_ => panic!("Not a Relay Chain"),
	}
}

// Take the parachain authorization calls and the Relay Chain call, and batch them into one call
// that can be executed on the Relay Chain. The call returned here is the proposal to put to
// referendum.
async fn construct_batch(
	upgrade_details: &UpgradeDetails,
	relay_call: CallInfo,
	para_calls: Vec<CallInfo>,
) -> CallInfo {
	println!("\nBatching calls.");
	match upgrade_details.relay.network {
		Network::Kusama =>
			construct_kusama_batch(relay_call, para_calls, upgrade_details.additional.clone()).await,
		Network::Polkadot =>
			construct_polkadot_batch(relay_call, para_calls, upgrade_details.additional.clone())
				.await,
		_ => panic!("Not a Relay Chain"),
	}
}

// Construct the batch needed on Kusama.
async fn construct_kusama_batch(
	relay_call: CallInfo,
	para_calls: Vec<CallInfo>,
	additional: Option<CallInfo>,
) -> CallInfo {
	use kusama_relay::runtime_types::pallet_utility::pallet::Call as UtilityCall;

	let mut batch_calls = Vec::new();
	for auth in para_calls {
		match auth.network {
			Network::Kusama | Network::Polkadot =>
				panic!("para calls should not contain relay calls"),
			Network::PolkadotAssetHub
			| Network::PolkadotCollectives
			| Network::PolkadotBridgeHub => panic!("not kusama parachains"),
			Network::KusamaAssetHub => {
				let send_auth = send_as_superuser_from_kusama(&auth).await;
				batch_calls.push(send_auth);
			},
			Network::KusamaEncointer => {
				let send_auth = send_as_superuser_from_kusama(&auth).await;
				batch_calls.push(send_auth);
			},
			Network::KusamaBridgeHub => {
				let send_auth = send_as_superuser_from_kusama(&auth).await;
				batch_calls.push(send_auth);
			},
			Network::KusamaPeople => {
				let send_auth = send_as_superuser_from_kusama(&auth).await;
				batch_calls.push(send_auth);
			},
			Network::KusamaCoretime => {
				let send_auth = send_as_superuser_from_kusama(&auth).await;
				batch_calls.push(send_auth);
			}
		}
	}
	if let Some(a) = additional {
		batch_calls.push(a.get_kusama_call().expect("kusama call"))
	}
	// Relay set code goes last
	batch_calls.push(relay_call.get_kusama_call().expect("kusama call"));
	CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(KusamaRuntimeCall::Utility(
		UtilityCall::force_batch { calls: batch_calls },
	)))
}

// Construct the batch needed on Polkadot.
async fn construct_polkadot_batch(
	relay_call: CallInfo,
	para_calls: Vec<CallInfo>,
	additional: Option<CallInfo>,
) -> CallInfo {
	use polkadot_relay::runtime_types::pallet_utility::pallet::Call as UtilityCall;

	let mut batch_calls = Vec::new();
	for auth in para_calls {
		match auth.network {
			Network::Kusama | Network::Polkadot =>
				panic!("para calls should not contain relay calls"),
			Network::KusamaAssetHub
			| Network::KusamaBridgeHub
			| Network::KusamaPeople
			| Network::KusamaCoretime 
			| Network::KusamaEncointer => panic!("not polkadot parachains"),
			Network::PolkadotAssetHub => {
				let send_auth = send_as_superuser_from_polkadot(&auth).await;
				batch_calls.push(send_auth);
			},
			Network::PolkadotCollectives => {
				let send_auth = send_as_superuser_from_polkadot(&auth).await;
				batch_calls.push(send_auth);
			},
			Network::PolkadotBridgeHub => {
				let send_auth = send_as_superuser_from_polkadot(&auth).await;
				batch_calls.push(send_auth);
			},
		}
	}
	if let Some(a) = additional {
		batch_calls.push(a.get_polkadot_call().expect("polkadot call"))
	}
	// Relay set code goes last
	batch_calls.push(relay_call.get_polkadot_call().expect("polkadot call"));
	CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(PolkadotRuntimeCall::Utility(
		UtilityCall::force_batch { calls: batch_calls },
	)))
}

// Take a call, which includes its intended destination, and wrap it in XCM instructions to `send`
// it from the Kusama Relay Chain, with `Root` origin, and have it execute on its destination.
async fn send_as_superuser_from_kusama(auth: &CallInfo) -> KusamaRuntimeCall {
	use kusama_relay::runtime_types::{
		pallet_xcm::pallet::Call as XcmCall,
		sp_weights::weight_v2::Weight as KusamaWeight,
		staging_xcm::v3::multilocation::MultiLocation,
		xcm::{
			double_encoded::DoubleEncoded,
			v2::OriginKind,
			v3::{
				junction::Junction::Parachain, junctions::Junctions::X1, Instruction, WeightLimit,
				Xcm,
			},
			VersionedMultiLocation,
			VersionedXcm::V3,
		},
	};

	let (ref_time, proof_size) = get_weight(auth).await;
	let para_id = auth.network.get_para_id().unwrap();
	KusamaRuntimeCall::XcmPallet(XcmCall::send {
		dest: Box::new(VersionedMultiLocation::V3(MultiLocation {
			parents: 0,
			interior: X1(Parachain(para_id)),
		})),
		message: Box::new(V3(Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind: OriginKind::Superuser,
				require_weight_at_most: KusamaWeight { ref_time, proof_size },
				call: DoubleEncoded { encoded: auth.encoded.clone() },
			},
		]))),
	})
}

// Take a call, which includes its intended destination, and wrap it in XCM instructions to `send`
// it from the Polkadot Relay Chain, with `Root` origin, and have it execute on its destination.
async fn send_as_superuser_from_polkadot(auth: &CallInfo) -> PolkadotRuntimeCall {
	use polkadot_relay::runtime_types::{
		pallet_xcm::pallet::Call as XcmCall,
		sp_weights::weight_v2::Weight as PolkadotWeight,
		staging_xcm::v3::multilocation::MultiLocation,
		xcm::{
			double_encoded::DoubleEncoded,
			v2::OriginKind,
			v3::{
				junction::Junction::Parachain, junctions::Junctions::X1, Instruction, WeightLimit,
				Xcm,
			},
			VersionedMultiLocation,
			VersionedXcm::V3,
		},
	};

	let (ref_time, proof_size) = get_weight(auth).await;
	let para_id = auth.network.get_para_id().unwrap();
	PolkadotRuntimeCall::XcmPallet(XcmCall::send {
		dest: Box::new(VersionedMultiLocation::V3(MultiLocation {
			parents: 0,
			interior: X1(Parachain(para_id)),
		})),
		message: Box::new(V3(Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind: OriginKind::Superuser,
				require_weight_at_most: PolkadotWeight { ref_time, proof_size },
				call: DoubleEncoded { encoded: auth.encoded.clone() },
			},
		]))),
	})
}

// Get the weight needed to successfully `Transact` on a foreign chain.
async fn get_weight(call: &CallInfo) -> (u64, u64) {
	// Do some weight calculation for execution of Transact on a parachain.
	let weight_from = &call.network;
	let max_ref_time: u64 = 500_000_000_000 - 1;
	let max_proof_size: u64 = 3 * 1024 * 1024 - 1;
	let weight_needed = call
		.get_transact_weight_needed(
			weight_from,
			Weight { ref_time: 1_000_000_000, proof_size: 1024 },
		)
		.await;
	// Double the weight needed, just to be safe from a runtime upgrade that could change
	// things during the referendum period.
	(
		(2 * weight_needed.ref_time).min(max_ref_time),
		(2 * weight_needed.proof_size).max(1024).min(max_proof_size),
		//                            ^^^^^^^^^^
		// sometimes it gives a proof size of 0, which is scary. make it 1024.
	)
}

// Write the call needed to disk and provide instructions to the user about how to propose it.
fn write_batch(upgrade_details: &UpgradeDetails, batch: CallInfo) {
	let fname = upgrade_details.output_file.as_str();
	let mut info_to_write = "0x".to_owned();
	info_to_write.push_str(hex::encode(batch.encoded).as_str());
	fs::write(fname, info_to_write).expect("it should write");

	println!("\nSuccess! The call data was written to {}", fname);
	println!("To submit this as a referendum in OpenGov, run:");
	let network = match upgrade_details.relay.network {
		Network::Kusama => "kusama",
		Network::Polkadot => "polkadot",
		_ => panic!("not a relay network"),
	};
	println!("\nopengov-cli submit-referendum \\");
	println!("    --proposal \"{}\" \\", fname);
	println!("    --network \"{}\" --track <\"root\" or \"whitelistedcaller\">", network);
}
