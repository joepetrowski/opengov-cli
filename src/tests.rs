use crate::cli::get_proposal_bytes;
use crate::polkadot_asset_hub::runtime_types::frame_system::pallet::Call as PolkadotAssetHubSystemCall;
use crate::polkadot_relay::runtime_types::frame_system::pallet::Call as PolkadotRelaySystemCall;
use crate::{
	blake2_256,
	cli::{
		build_upgrade::{self, UpgradeArgs},
		submit_referendum::generate_calls,
		CallOrHash, ProposalDetails, VersionedNetwork,
	},
	CallInfo, KusamaAssetHubOpenGovOrigin, KusamaAssetHubRuntimeCall, Network, NetworkRuntimeCall,
	PolkadotAssetHubOpenGovOrigin, PolkadotAssetHubRuntimeCall, PolkadotRuntimeCall,
};

fn polkadot_whitelist_remark_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: Polkadot(PolkadotAssetHubOpenGovOrigin::WhitelistedCaller),
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: false,
	}
}

fn polkadot_staking_validator_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `staking.increase_validator_count(50)`
		proposal: String::from("0x070ac8"),
		track: Polkadot(PolkadotAssetHubOpenGovOrigin::StakingAdmin),
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: false,
	}
}

fn polkadot_root_remark_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: PolkadotRoot,
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: false,
	}
}

fn kusama_whitelist_remark_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: Kusama(KusamaAssetHubOpenGovOrigin::WhitelistedCaller),
		dispatch: At(100_000_000),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: false,
	}
}

fn kusama_whitelist_polkadot_fellowship_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: Kusama(KusamaAssetHubOpenGovOrigin::WhitelistedCaller),
		dispatch: At(100_000_000),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: true,
	}
}

fn kusama_staking_validator_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `staking.increase_validator_count(50)`
		proposal: String::from("0x060ac8"),
		track: Kusama(KusamaAssetHubOpenGovOrigin::StakingAdmin),
		dispatch: At(100_000_000),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: false,
	}
}

fn kusama_root_remark_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: KusamaRoot,
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: false,
	}
}

fn limited_length_user_input() -> ProposalDetails {
	use crate::cli::Output::*;
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: Polkadot(PolkadotAssetHubOpenGovOrigin::StakingAdmin),
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 5, // very limiting
		print_batch: true,
		use_light_client: false,
		fellowship_on_polkadot: false,
	}
}

fn upgrade_args_for_only_relay() -> UpgradeArgs {
	UpgradeArgs {
		network: String::from("polkadot"),
		only: true,
		local: false,
		relay_version: Some(String::from("v1.2.0")),
		asset_hub: None,
		bridge_hub: None,
		collectives: None,
		encointer: None,
		people: None,
		coretime: None,
		bulletin: None,
		filename: None,
		additional: None,
		no_runtime_checks: false,
	}
}

fn upgrade_args_for_only_asset_hub() -> UpgradeArgs {
	UpgradeArgs {
		network: String::from("polkadot"),
		only: true,
		local: false,
		relay_version: None,
		asset_hub: Some(String::from("v1.2.0")),
		bridge_hub: None,
		collectives: None,
		encointer: None,
		people: None,
		coretime: None,
		bulletin: None,
		filename: None,
		additional: None,
		no_runtime_checks: false,
	}
}

fn upgrade_args_for_all() -> UpgradeArgs {
	UpgradeArgs {
		network: String::from("polkadot"),
		only: false,
		local: false,
		relay_version: Some(String::from("v1.2.0")),
		asset_hub: None,
		bridge_hub: None,
		collectives: None,
		encointer: None,
		people: None,
		coretime: None,
		bulletin: None,
		filename: None,
		additional: None,
		no_runtime_checks: false,
	}
}

fn upgrade_args_with_additional() -> UpgradeArgs {
	UpgradeArgs {
		network: String::from("polkadot"),
		only: true,
		local: false,
		relay_version: Some(String::from("v1.2.0")),
		asset_hub: None,
		bridge_hub: None,
		collectives: None,
		encointer: None,
		people: None,
		coretime: None,
		bulletin: None,
		filename: None,
		// `system.remark("test")` on Polkadot Asset Hub
		additional: Some(String::from("0x00001074657374")),
		no_runtime_checks: false,
	}
}

#[test]
fn call_info_from_bytes_works() {
	let proposal_details = polkadot_whitelist_remark_user_input();
	let proposal_bytes = get_proposal_bytes(proposal_details.proposal);
	let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Polkadot);

	let remark_to_verify = PolkadotRuntimeCall::System(PolkadotRelaySystemCall::remark {
		remark: b"opengov-submit test".to_vec(),
	});

	let hash_to_verify = "0x8821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534";
	let verification_bytes =
		hex::decode(hash_to_verify.trim_start_matches("0x")).expect("Valid hash");

	assert_eq!(proposal_call_info.get_polkadot_call().expect("polkadot"), remark_to_verify);
	assert_eq!(proposal_call_info.encoded, proposal_bytes);
	assert_eq!(proposal_call_info.hash, &verification_bytes[..]);
	assert_eq!(proposal_call_info.length, 22u32);

	let bad_remark = PolkadotRuntimeCall::System(PolkadotRelaySystemCall::remark {
		remark: b"another remark".to_vec(),
	});
	let bad_remark_hash = "0x8821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534";
	let bad_verification =
		hex::decode(bad_remark_hash.trim_start_matches("0x")).expect("Valid hash");
	assert_ne!(proposal_call_info.get_polkadot_call().expect("polkadot"), bad_remark);
	assert_eq!(proposal_call_info.hash, &bad_verification[..]);
}

#[test]
fn get_proposal_bytes_trims_file_contents() {
	let path = std::env::temp_dir()
		.join(format!("opengov-cli-proposal-{}-trim-test.call", std::process::id()));
	std::fs::write(&path, "\n  0x00001074657374\n").expect("write proposal file");

	let proposal_bytes = get_proposal_bytes(path.to_string_lossy().into_owned());
	std::fs::remove_file(&path).ok();

	assert_eq!(proposal_bytes, hex::decode("00001074657374").expect("hex"));
}

#[test]
fn call_info_from_runtime_call_works() {
	let remark_to_verify = PolkadotRuntimeCall::System(PolkadotRelaySystemCall::remark {
		remark: b"opengov-submit test".to_vec(),
	});
	let call_info = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(remark_to_verify));

	let encoded_to_verify =
		hex::decode("0x00004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"))
			.expect("Valid encoded");

	let hash_to_verify = "0x8821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534";
	let verification_bytes =
		hex::decode(hash_to_verify.trim_start_matches("0x")).expect("Valid hash");

	assert_eq!(call_info.encoded, encoded_to_verify);
	assert_eq!(call_info.hash, &verification_bytes[..]);
	assert_eq!(call_info.length, 22u32);
}

#[tokio::test]
async fn it_starts_polkadot_non_fellowship_referenda_correctly() {
	let proposal_details = polkadot_staking_validator_user_input();
	let calls = generate_calls(&proposal_details).await;

	let public_preimage =
		hex::decode("0x05000c070ac8".trim_start_matches("0x")).expect("Valid call");
	let public_referendum = hex::decode("0x3e003f0002439a93279b25a49bf366c9fe1b06d4fc342f46b5a3b2734dcffe0c56c12b28ef03000000010a000000".trim_start_matches("0x")).expect("Valid call");

	assert!(calls.preimage_for_whitelist_call.is_none(), "it must not generate this call");
	assert!(calls.fellowship_referendum_submission.is_none(), "it must not generate this call");

	assert!(calls.preimage_for_public_referendum.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_public_referendum {
		match coh {
			CallOrHash::Call(public_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(public_preimage_generated);
				assert_eq!(call_info.encoded, public_preimage);
				assert_eq!(length, 6u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

	assert!(calls.public_referendum_submission.is_some(), "it must generate this call");
	if let Some(public_referendum_generated) = calls.public_referendum_submission {
		let call_info = CallInfo::from_runtime_call(public_referendum_generated);
		assert_eq!(call_info.encoded, public_referendum);
	}
}

#[tokio::test]
async fn it_starts_polkadot_fellowship_referenda_correctly() {
	// Fellowship is on Collectives, send XCM to Asset Hub to whitelist.
	let proposal_details = polkadot_whitelist_remark_user_input();
	let calls = generate_calls(&proposal_details).await;

	let public_preimage = hex::decode(
		"0x050060400300004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	// Fellowship referendum now uses Inline, so it contains the XCM call directly.
	let fellowship_referendum = hex::decode("0x3d003e0201cc1f0005010100a10f05082f00000603008840008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534010a000000".trim_start_matches("0x")).expect("Valid call");
	let public_referendum = hex::decode("0x3e003f0d02a322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e18000000010a000000".trim_start_matches("0x")).expect("Valid call");

	// No preimage needed for whitelist call - it's inlined in the fellowship referendum.
	assert!(calls.preimage_for_whitelist_call.is_none(), "should be None with Inline");

	assert!(calls.preimage_for_public_referendum.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_public_referendum {
		match coh {
			CallOrHash::Call(public_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(public_preimage_generated);
				assert_eq!(call_info.encoded, public_preimage);
				assert_eq!(length, 27u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

	assert!(calls.fellowship_referendum_submission.is_some(), "it must generate this call");
	if let Some(fellowship_referendum_generated) = calls.fellowship_referendum_submission {
		let call_info = CallInfo::from_runtime_call(fellowship_referendum_generated);
		assert_eq!(call_info.encoded, fellowship_referendum);
	}

	assert!(calls.public_referendum_submission.is_some(), "it must generate this call");
	if let Some(public_referendum_generated) = calls.public_referendum_submission {
		let call_info = CallInfo::from_runtime_call(public_referendum_generated);
		assert_eq!(call_info.encoded, public_referendum);
	}
}

#[tokio::test]
async fn it_starts_polkadot_root_referenda_correctly() {
	let proposal_details = polkadot_root_remark_user_input();
	let calls = generate_calls(&proposal_details).await;

	let public_preimage = hex::decode(
		"0x05005800004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	let public_referendum = hex::decode("0x3e000000028821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca253416000000010a000000".trim_start_matches("0x")).expect("Valid call");

	assert!(calls.preimage_for_whitelist_call.is_none(), "it must not generate this call");
	assert!(calls.fellowship_referendum_submission.is_none(), "it must not generate this call");

	assert!(calls.preimage_for_public_referendum.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_public_referendum {
		match coh {
			CallOrHash::Call(public_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(public_preimage_generated);
				assert_eq!(call_info.encoded, public_preimage);
				assert_eq!(length, 25u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

	assert!(calls.public_referendum_submission.is_some(), "it must generate this call");
	if let Some(public_referendum_generated) = calls.public_referendum_submission {
		let call_info = CallInfo::from_runtime_call(public_referendum_generated);
		assert_eq!(call_info.encoded, public_referendum);
	}
}

#[tokio::test]
async fn it_starts_kusama_non_fellowship_referenda_correctly() {
	let proposal_details = kusama_staking_validator_user_input();
	let calls = generate_calls(&proposal_details).await;

	let public_preimage =
		hex::decode("0x06000c060ac8".trim_start_matches("0x")).expect("Valid call");
	let public_referendum = hex::decode("0x5c005d00028fd8848a8f93980f5cea2de1c11f29ed7dced592aa207218a2e0ae5b78b9fffb030000000000e1f505".trim_start_matches("0x")).expect("Valid call");

	assert!(calls.preimage_for_whitelist_call.is_none(), "it must not generate this call");
	assert!(calls.fellowship_referendum_submission.is_none(), "it must not generate this call");

	assert!(calls.preimage_for_public_referendum.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_public_referendum {
		match coh {
			CallOrHash::Call(public_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(public_preimage_generated);
				assert_eq!(call_info.encoded, public_preimage);
				assert_eq!(length, 6u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

	assert!(calls.public_referendum_submission.is_some(), "it must generate this call");
	if let Some(public_referendum_generated) = calls.public_referendum_submission {
		let call_info = CallInfo::from_runtime_call(public_referendum_generated);
		assert_eq!(call_info.encoded, public_referendum);
	}
}

#[tokio::test]
async fn it_starts_kusama_fellowship_referenda_correctly() {
	let proposal_details = kusama_whitelist_remark_user_input();
	let calls = generate_calls(&proposal_details).await;

	// On Kusama, the fellowship is on the Relay Chain and uses inline calls,
	// so preimage_for_whitelist_call is None. The fellowship referendum is submitted
	// on the Relay Chain and sends XCM to Asset Hub to whitelist.
	let public_preimage = hex::decode(
		"0x0600605e0300004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	let fellowship_referendum = hex::decode("0x17002b0f01cc630005000100a10f05082f0000060300885e008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534010a000000".trim_start_matches("0x")).expect("Valid call");
	let public_referendum = hex::decode("0x5c005d0d02dd86316423e1bc1ca2ac30b36d9384c7edea7b4e033a2b81c1a45c75091c2f15180000000000e1f505".trim_start_matches("0x")).expect("Valid call");

	// Kusama fellowship uses inline preimage, so no separate preimage note call
	assert!(
		calls.preimage_for_whitelist_call.is_none(),
		"kusama uses inline preimages for fellowship"
	);

	assert!(calls.preimage_for_public_referendum.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_public_referendum {
		match coh {
			CallOrHash::Call(public_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(public_preimage_generated);
				assert_eq!(call_info.encoded, public_preimage);
				assert_eq!(length, 27u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

	assert!(calls.fellowship_referendum_submission.is_some(), "it must generate this call");
	if let Some(fellowship_referendum_generated) = calls.fellowship_referendum_submission {
		let call_info = CallInfo::from_runtime_call(fellowship_referendum_generated);
		assert_eq!(call_info.encoded, fellowship_referendum);
	}

	assert!(calls.public_referendum_submission.is_some(), "it must generate this call");
	if let Some(public_referendum_generated) = calls.public_referendum_submission {
		let call_info = CallInfo::from_runtime_call(public_referendum_generated);
		assert_eq!(call_info.encoded, public_referendum);
	}
}

#[tokio::test]
async fn it_starts_polkadot_fellowship_whitelisted_kusama_referenda_correctly() {
	let proposal_details = kusama_whitelist_polkadot_fellowship_user_input();
	let calls = generate_calls(&proposal_details).await;

	// The fellowship referendum should be on Polkadot Collectives.
	assert!(
		calls.fellowship_referendum_submission.is_some(),
		"it must generate a fellowship referendum"
	);
	if let Some(ref fellowship_ref) = calls.fellowship_referendum_submission {
		match fellowship_ref {
			NetworkRuntimeCall::PolkadotCollectives(_) => (),
			other => panic!(
				"Fellowship referendum should be on PolkadotCollectives, got {:?}",
				std::mem::discriminant(other)
			),
		}
	}

	// The public referendum should be on Kusama Asset Hub.
	assert!(calls.public_referendum_submission.is_some(), "it must generate a public referendum");
	if let Some(ref public_ref) = calls.public_referendum_submission {
		match public_ref {
			NetworkRuntimeCall::KusamaAssetHub(_) => (),
			other => panic!(
				"Public referendum should be on KusamaAssetHub, got {:?}",
				std::mem::discriminant(other)
			),
		}
	}

	// The preimage for public referendum should exist.
	assert!(
		calls.preimage_for_public_referendum.is_some(),
		"it must generate the preimage for the public referendum"
	);
}

#[tokio::test]
async fn it_starts_kusama_root_referenda_correctly() {
	let proposal_details = kusama_root_remark_user_input();
	let calls = generate_calls(&proposal_details).await;

	let public_preimage = hex::decode(
		"0x06005800004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	let public_referendum = hex::decode("0x5c000000028821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca253416000000010a000000".trim_start_matches("0x")).expect("Valid call");

	assert!(calls.preimage_for_whitelist_call.is_none(), "it must not generate this call");
	assert!(calls.fellowship_referendum_submission.is_none(), "it must not generate this call");

	assert!(calls.preimage_for_public_referendum.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_public_referendum {
		match coh {
			CallOrHash::Call(public_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(public_preimage_generated);
				assert_eq!(call_info.encoded, public_preimage);
				assert_eq!(length, 25u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

	assert!(calls.public_referendum_submission.is_some(), "it must generate this call");
	if let Some(public_referendum_generated) = calls.public_referendum_submission {
		let call_info = CallInfo::from_runtime_call(public_referendum_generated);
		assert_eq!(call_info.encoded, public_referendum);
	}
}

#[test]
fn only_relay_chain() {
	let args = upgrade_args_for_only_relay();
	let details = build_upgrade::parse_inputs(args);
	assert_eq!(details.relay, Network::Polkadot);
	let expected_networks =
		vec![VersionedNetwork { network: Network::Polkadot, version: String::from("1.2.0") }];
	assert_eq!(details.networks, expected_networks);
	assert!(details.additional.is_none());
}

#[test]
fn only_asset_hub() {
	let args = upgrade_args_for_only_asset_hub();
	let details = build_upgrade::parse_inputs(args);
	assert_eq!(details.relay, Network::Polkadot);
	let expected_networks = vec![VersionedNetwork {
		network: Network::PolkadotAssetHub,
		version: String::from("1.2.0"),
	}];
	assert_eq!(details.networks, expected_networks);
	assert!(details.additional.is_none());
}

#[test]
fn upgrade_everything_works_with_just_relay_version() {
	let args = upgrade_args_for_all();
	let details = build_upgrade::parse_inputs(args);
	assert_eq!(details.relay, Network::Polkadot);
	let expected_networks = vec![
		VersionedNetwork { network: Network::Polkadot, version: String::from("1.2.0") },
		VersionedNetwork { network: Network::PolkadotAssetHub, version: String::from("1.2.0") },
		VersionedNetwork { network: Network::PolkadotCollectives, version: String::from("1.2.0") },
		VersionedNetwork { network: Network::PolkadotBridgeHub, version: String::from("1.2.0") },
		VersionedNetwork { network: Network::PolkadotPeople, version: String::from("1.2.0") },
		VersionedNetwork { network: Network::PolkadotCoretime, version: String::from("1.2.0") },
		VersionedNetwork { network: Network::PolkadotBulletin, version: String::from("1.2.0") },
	];
	assert_eq!(details.networks, expected_networks);
	assert!(details.additional.is_none());
}

#[test]
fn additional_call_decodes_correctly() {
	let args = upgrade_args_with_additional();
	let details = build_upgrade::parse_inputs(args);

	assert!(details.additional.is_some(), "additional should be set");
	let additional = details.additional.unwrap();

	// Verify the call decodes to the expected remark on Asset Hub
	let expected_remark = PolkadotAssetHubRuntimeCall::System(PolkadotAssetHubSystemCall::remark {
		remark: b"test".to_vec(),
	});
	assert_eq!(
		additional.get_polkadot_asset_hub_call().expect("polkadot asset hub call"),
		expected_remark
	);

	// Verify length (0x00001074657374 = 7 bytes)
	assert_eq!(additional.length, 7u32);
}

#[test]
fn it_creates_constrained_print_output() {
	let proposal_details = limited_length_user_input();
	let proposal_bytes = get_proposal_bytes(proposal_details.proposal);
	let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Polkadot);
	let (coh, length) = proposal_call_info.create_print_output(proposal_details.output_len_limit);

	let expected_hash = hex::decode(
		"0x8821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534"
			.trim_start_matches("0x"),
	)
	.expect("Valid hash");

	match coh {
		CallOrHash::Call(_) => panic!("this should not have a call"),
		CallOrHash::Hash(h) => {
			assert_eq!(h, &expected_hash[..]);
		},
	}
	assert_eq!(length, proposal_call_info.length);
}

// Add Invulnerables -------------------------------------------------------------------------------

#[test]
fn add_invulnerables_parses_and_encodes_a_single_account() {
	use crate::cli::add_invulnerables::{build_add_invulnerables_call, parse_accounts};

	// One of the Bulletin genesis invulnerables.
	let who = String::from("1A1WrKowzJD4yQQcETugEV5UWoNo1o7ujuA3f1fBfpxPjZL");
	let accounts = parse_accounts(&[who], &Network::PolkadotBulletin);
	let expected_account = "06def0ef07d9b5153276dd785525839706f4696c8cb86227a2af27fd7495ee63";
	assert_eq!(hex::encode(accounts[0]), expected_account);

	let call = build_add_invulnerables_call(&Network::PolkadotBulletin, &accounts);
	// collatorSelection (pallet 21) . addInvulnerable (call 5) ++ account
	assert_eq!(hex::encode(&call.encoded), format!("1505{expected_account}"));
}

#[test]
fn add_invulnerables_batches_multiple_accounts() {
	use crate::cli::add_invulnerables::build_add_invulnerables_call;

	let a = [1u8; 32];
	let b = [2u8; 32];
	let call = build_add_invulnerables_call(&Network::PolkadotBulletin, &[a, b]);
	// utility (pallet 6) . forceBatch (call 4) ++ compact(2) ++ two addInvulnerable calls
	let expected = format!("0604081505{}1505{}", hex::encode(a), hex::encode(b));
	assert_eq!(hex::encode(&call.encoded), expected);
}

#[test]
fn add_invulnerables_wraps_for_staking_admin_via_xcm() {
	use crate::cli::add_invulnerables::{build_add_invulnerables_call, wrap_for_governance};
	use crate::polkadot_asset_hub::runtime_types::{
		pallet_xcm::pallet::Call as XcmCall,
		staging_xcm::v5::{junction::Junction::Parachain, junctions::Junctions::X1, Instruction},
		xcm::{v3::OriginKind, VersionedLocation, VersionedXcm::V5},
	};

	let target = build_add_invulnerables_call(&Network::PolkadotBulletin, &[[1u8; 32]]);
	let proposal = wrap_for_governance(&target);
	assert_eq!(proposal.network, Network::PolkadotAssetHub);

	let call = proposal.get_polkadot_asset_hub_call().expect("polkadot asset hub call");
	let PolkadotAssetHubRuntimeCall::PolkadotXcm(XcmCall::send { dest, message }) = call else {
		panic!("expected a PolkadotXcm::send call");
	};
	let VersionedLocation::V5(dest) = *dest else { panic!("expected a V5 location") };
	assert_eq!(dest.parents, 1);
	assert!(matches!(dest.interior, X1([Parachain(1010)])));

	let V5(xcm) = *message else { panic!("expected a V5 message") };
	assert_eq!(xcm.0.len(), 2);
	assert!(matches!(&xcm.0[0], Instruction::UnpaidExecution { .. }));
	let Instruction::Transact { origin_kind, call, .. } = &xcm.0[1] else {
		panic!("expected a Transact instruction");
	};
	assert!(matches!(origin_kind, OriginKind::Xcm));
	assert_eq!(call.encoded, target.encoded);
}

#[test]
fn add_invulnerables_does_not_wrap_asset_hub_calls() {
	use crate::cli::add_invulnerables::{build_add_invulnerables_call, wrap_for_governance};

	let target = build_add_invulnerables_call(&Network::PolkadotAssetHub, &[[1u8; 32]]);
	let proposal = wrap_for_governance(&target);
	assert_eq!(proposal.network, Network::PolkadotAssetHub);
	assert_eq!(proposal.encoded, target.encoded);
}

#[test]
#[should_panic(expected = "listed more than once")]
fn add_invulnerables_rejects_duplicate_accounts() {
	use crate::cli::add_invulnerables::parse_accounts;

	let hex_account =
		String::from("0x0101010101010101010101010101010101010101010101010101010101010101");
	let ss58_account = String::from("1A1WrKowzJD4yQQcETugEV5UWoNo1o7ujuA3f1fBfpxPjZL");
	parse_accounts(&[ss58_account.clone(), hex_account, ss58_account], &Network::PolkadotBulletin);
}

// Build Upgrade: encoding -------------------------------------------------------------------------

// The expected proposals below were produced by `build-upgrade --local` and `add-invulnerables`
// on master before `build_upgrade.rs` was refactored, against runtime metadata at spec 2005002
// (Polkadot) and 2005001 (Kusama). Each chain's runtime was a stand-in file containing
// `fake runtime <runtime name>`. If one of these tests fails after a metadata refresh, a call or
// type that this tool encodes has changed on chain: check the new encoding by hand before
// updating the expected value.

fn upgrade_args(network: &str, relay_version: Option<&str>) -> UpgradeArgs {
	UpgradeArgs {
		network: String::from(network),
		only: relay_version.is_none(),
		local: true,
		relay_version: relay_version.map(String::from),
		asset_hub: None,
		bridge_hub: None,
		collectives: None,
		encointer: None,
		people: None,
		coretime: None,
		bulletin: None,
		filename: None,
		additional: None,
		no_runtime_checks: false,
	}
}

// Build the proposal that `build-upgrade` would produce, hashing stand-in bytes in place of each
// chain's real runtime.
async fn upgrade_proposal_with_stand_in_runtimes(args: UpgradeArgs) -> String {
	let details = build_upgrade::parse_inputs(args);
	let calls = details
		.networks
		.iter()
		.map(|chain| {
			let runtime = format!("fake runtime {}", chain.network.runtime_name());
			crate::authorize_upgrade_call(&chain.network, blake2_256(runtime.as_bytes()))
		})
		.collect();
	let proposal = crate::upgrade::batch_calls(&details.relay, calls, details.additional.clone())
		.expect("valid batch");
	format!("0x{}", hex::encode(proposal.encoded))
}

#[test]
fn authorize_upgrade_encodes_correctly_on_every_chain() {
	use Network::*;
	let code_hash = [7u8; 32];
	for network in [
		Kusama,
		KusamaAssetHub,
		KusamaBridgeHub,
		KusamaPeople,
		KusamaCoretime,
		KusamaEncointer,
		Polkadot,
		PolkadotAssetHub,
		PolkadotCollectives,
		PolkadotBridgeHub,
		PolkadotPeople,
		PolkadotCoretime,
		PolkadotBulletin,
	] {
		let call = crate::authorize_upgrade_call(&network, code_hash);
		assert_eq!(call.network, network);
		// system (pallet 0) . authorize_upgrade (call 9) ++ code hash
		assert_eq!(call.encoded, [&[0u8, 9][..], &code_hash[..]].concat(), "{network:?}");
	}
}

#[tokio::test]
async fn polkadot_upgrade_of_all_chains_encodes_correctly() {
	let args = upgrade_args("polkadot", Some("1.2.0"));
	assert_eq!(upgrade_proposal_with_stand_in_runtimes(args).await, EXPECTED_UPGRADE_POLKADOT_ALL);
}

#[tokio::test]
async fn polkadot_upgrade_with_additional_call_encodes_correctly() {
	let mut args = upgrade_args("polkadot", Some("1.2.0"));
	// `system.remark("test")` on Polkadot Asset Hub
	args.additional = Some(String::from("0x00001074657374"));
	assert_eq!(
		upgrade_proposal_with_stand_in_runtimes(args).await,
		EXPECTED_UPGRADE_POLKADOT_ALL_WITH_ADDITIONAL
	);
}

#[tokio::test]
async fn polkadot_upgrade_of_only_bulletin_encodes_correctly() {
	let mut args = upgrade_args("polkadot", None);
	args.bulletin = Some(String::from("1.2.0"));
	assert_eq!(
		upgrade_proposal_with_stand_in_runtimes(args).await,
		EXPECTED_UPGRADE_POLKADOT_ONLY_BULLETIN
	);
}

#[tokio::test]
async fn kusama_upgrade_of_all_chains_encodes_correctly() {
	let args = upgrade_args("kusama", Some("1.2.0"));
	assert_eq!(upgrade_proposal_with_stand_in_runtimes(args).await, EXPECTED_UPGRADE_KUSAMA_ALL);
}

#[tokio::test]
async fn kusama_upgrade_with_additional_call_encodes_correctly() {
	let mut args = upgrade_args("kusama", Some("1.2.0"));
	// `system.remark("test")` on Kusama Asset Hub
	args.additional = Some(String::from("0x00001074657374"));
	assert_eq!(
		upgrade_proposal_with_stand_in_runtimes(args).await,
		EXPECTED_UPGRADE_KUSAMA_ALL_WITH_ADDITIONAL
	);
}

#[tokio::test]
async fn kusama_upgrade_of_only_encointer_encodes_correctly() {
	let mut args = upgrade_args("kusama", None);
	args.encointer = Some(String::from("1.2.0"));
	assert_eq!(
		upgrade_proposal_with_stand_in_runtimes(args).await,
		EXPECTED_UPGRADE_KUSAMA_ONLY_ENCOINTER
	);
}

#[tokio::test]
async fn kusama_upgrade_sends_authorizations_as_superuser() {
	use crate::kusama_asset_hub::runtime_types::{
		frame_system::pallet::Call as SystemCall,
		pallet_utility::pallet::Call as UtilityCall,
		pallet_xcm::pallet::Call as XcmCall,
		staging_xcm::v5::{
			junction::Junction::Parachain,
			junctions::Junctions::{Here, X1},
			Instruction,
		},
		xcm::{v3::OriginKind, VersionedLocation, VersionedXcm::V5},
	};

	let mut args = upgrade_args("kusama", None);
	args.relay_version = Some(String::from("1.2.0"));
	args.asset_hub = Some(String::from("1.2.0"));
	args.encointer = Some(String::from("1.2.0"));
	let details = build_upgrade::parse_inputs(args);
	let calls: Vec<CallInfo> = details
		.networks
		.iter()
		.enumerate()
		.map(|(i, chain)| crate::authorize_upgrade_call(&chain.network, [i as u8; 32]))
		.collect();
	let proposal =
		crate::upgrade::batch_calls(&details.relay, calls.clone(), details.additional.clone())
			.expect("valid batch");

	let batch = proposal.get_kusama_asset_hub_call().expect("kusama asset hub call");
	let KusamaAssetHubRuntimeCall::Utility(UtilityCall::force_batch { calls: batch }) = batch
	else {
		panic!("expected a utility.force_batch call");
	};
	assert_eq!(batch.len(), 3);

	// Relay Chain and Encointer authorizations are sent over XCM to execute as Root.
	let expect_superuser_send = |call: &KusamaAssetHubRuntimeCall,
	                             para: Option<u32>,
	                             inner: &[u8]| {
		let KusamaAssetHubRuntimeCall::PolkadotXcm(XcmCall::send { dest, message }) = call else {
			panic!("expected a PolkadotXcm::send call");
		};
		let VersionedLocation::V5(dest) = dest.as_ref() else { panic!("expected a V5 location") };
		assert_eq!(dest.parents, 1);
		match para {
			None => assert!(matches!(dest.interior, Here)),
			Some(id) => assert!(matches!(dest.interior, X1([Parachain(p)]) if p == id)),
		}
		let V5(xcm) = message.as_ref() else { panic!("expected a V5 message") };
		assert_eq!(xcm.0.len(), 2);
		assert!(matches!(&xcm.0[0], Instruction::UnpaidExecution { .. }));
		let Instruction::Transact { origin_kind, call, .. } = &xcm.0[1] else {
			panic!("expected a Transact instruction");
		};
		assert!(matches!(origin_kind, OriginKind::Superuser));
		assert_eq!(call.encoded, inner);
	};
	expect_superuser_send(&batch[0], None, &calls[0].encoded);
	expect_superuser_send(&batch[2], Some(1_001), &calls[2].encoded);

	// Asset Hub's own authorization is dispatched locally, not sent.
	let KusamaAssetHubRuntimeCall::System(SystemCall::authorize_upgrade { code_hash }) = &batch[1]
	else {
		panic!("expected a local system.authorize_upgrade call");
	};
	assert_eq!(code_hash.0, [1u8; 32]);
}

// Build Upgrade: inputs ---------------------------------------------------------------------------

#[test]
fn kusama_upgrade_everything_includes_kusama_chains_only() {
	let details = build_upgrade::parse_inputs(upgrade_args("kusama", Some("v1.2.0")));
	assert_eq!(details.relay, Network::Kusama);
	let version = || String::from("1.2.0");
	let expected_networks = vec![
		VersionedNetwork { network: Network::Kusama, version: version() },
		VersionedNetwork { network: Network::KusamaAssetHub, version: version() },
		VersionedNetwork { network: Network::KusamaEncointer, version: version() },
		VersionedNetwork { network: Network::KusamaBridgeHub, version: version() },
		VersionedNetwork { network: Network::KusamaPeople, version: version() },
		VersionedNetwork { network: Network::KusamaCoretime, version: version() },
	];
	assert_eq!(details.networks, expected_networks);
}

#[test]
fn only_bulletin_strips_v_prefix() {
	let mut args = upgrade_args("polkadot", None);
	args.bulletin = Some(String::from("v1.2.0"));
	let details = build_upgrade::parse_inputs(args);
	assert_eq!(details.relay, Network::Polkadot);
	let expected_networks = vec![VersionedNetwork {
		network: Network::PolkadotBulletin,
		version: String::from("1.2.0"),
	}];
	assert_eq!(details.networks, expected_networks);
}

#[test]
fn semver_converts_to_runtime_version() {
	let semver_to_intver = |semver| crate::semver_to_intver(semver).expect("semver");
	assert_eq!(semver_to_intver("2.5.3"), "2005003");
	assert_eq!(semver_to_intver("1.10.0"), "1010000");
	assert_eq!(semver_to_intver("1.0.0"), "1000000");
	assert_eq!(semver_to_intver("12.345.678"), "12345678");
}

#[test]
#[should_panic(expected = "not semver")]
fn semver_conversion_rejects_non_semver() {
	crate::semver_to_intver("1.2").unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn runtime_file_names_match_fellowship_releases() {
	let chain = |network| VersionedNetwork { network, version: String::from("2.5.3") };
	assert_eq!(
		build_upgrade::runtime_file_name(&chain(Network::Polkadot)),
		"polkadot_runtime-v2005003.compact.compressed.wasm"
	);
	assert_eq!(
		build_upgrade::runtime_file_name(&chain(Network::PolkadotBulletin)),
		"bulletin-polkadot_runtime-v2005003.compact.compressed.wasm"
	);
	assert_eq!(
		build_upgrade::runtime_file_name(&chain(Network::KusamaEncointer)),
		"encointer-kusama_runtime-v2005003.compact.compressed.wasm"
	);
}

// Add Invulnerables: all chains -------------------------------------------------------------------

#[test]
fn add_invulnerable_encodes_correctly_on_every_chain() {
	use crate::cli::add_invulnerables::{build_add_invulnerables_call, parse_network};
	let who = [1u8; 32];
	for (name, _) in EXPECTED_ADD_INVULNERABLES_BATCHES {
		let call = build_add_invulnerables_call(&parse_network(name), &[who]);
		// collatorSelection (pallet 21 on every system chain) . addInvulnerable (call 5) ++ account
		assert_eq!(call.encoded, [&[21u8, 5][..], &who[..]].concat(), "{name}");
	}
}

#[test]
fn add_invulnerables_proposals_encode_correctly_on_every_chain() {
	use crate::cli::add_invulnerables::{
		build_add_invulnerables_call, parse_network, wrap_for_governance,
	};
	for (name, expected) in EXPECTED_ADD_INVULNERABLES_BATCHES {
		let target = build_add_invulnerables_call(&parse_network(name), &[[1u8; 32], [2u8; 32]]);
		let proposal = wrap_for_governance(&target);
		assert_eq!(format!("0x{}", hex::encode(&proposal.encoded)), expected, "{name}");
	}
}

#[test]
#[should_panic(expected = "Encointer's collator selection")]
fn add_invulnerables_rejects_encointer() {
	crate::cli::add_invulnerables::parse_network("encointer-kusama");
}

#[test]
#[should_panic(expected = "Relay chains do not have collator selection")]
fn add_invulnerables_rejects_relay_chains() {
	crate::cli::add_invulnerables::parse_network("polkadot");
}

#[test]
fn add_invulnerables_parses_kusama_addresses() {
	use crate::cli::add_invulnerables::parse_accounts;

	// Alice, with the Kusama SS58 prefix (2) and with the generic Polkadot one (0).
	let alice = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
	let inputs = [String::from("HNZata7iMYWmk5RvZRTiAsSDhV8366zq2YGb3tLH5Upf74F")];
	assert_eq!(hex::encode(parse_accounts(&inputs, &Network::KusamaPeople)[0]), alice);
	// A mismatched prefix only warns: the public key is what matters on chain.
	let inputs = [String::from("15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5")];
	assert_eq!(hex::encode(parse_accounts(&inputs, &Network::KusamaPeople)[0]), alice);
}

// Library -----------------------------------------------------------------------------------------

// The library must return the same bytes as the command line tool. The expected upgrade proposals
// are the ones that `build-upgrade --local` produced (see above), and referenda are compared with
// both the calls that `submit-referendum` prints and the encodings asserted in the tests above.

use crate::{
	build_upgrade as library_build_upgrade, referendum_calls, DispatchTimeWrapper, Error,
	NetworkTrack, ReferendumInput, UpgradeInput,
};

// Stand-in code hashes for `networks`, matching `upgrade_proposal_with_stand_in_runtimes`.
fn stand_in_runtimes(networks: &[Network]) -> Vec<(Network, [u8; 32])> {
	networks
		.iter()
		.map(|network| {
			let runtime = format!("fake runtime {}", network.runtime_name());
			(network.clone(), blake2_256(runtime.as_bytes()))
		})
		.collect()
}

const ALL_POLKADOT_CHAINS: [Network; 7] = [
	Network::Polkadot,
	Network::PolkadotAssetHub,
	Network::PolkadotCollectives,
	Network::PolkadotBridgeHub,
	Network::PolkadotPeople,
	Network::PolkadotCoretime,
	Network::PolkadotBulletin,
];

const ALL_KUSAMA_CHAINS: [Network; 6] = [
	Network::Kusama,
	Network::KusamaAssetHub,
	Network::KusamaEncointer,
	Network::KusamaBridgeHub,
	Network::KusamaPeople,
	Network::KusamaCoretime,
];

fn library_upgrade(relay: Network, networks: &[Network], additional: Option<&str>) -> String {
	let proposal = library_build_upgrade(UpgradeInput {
		relay: relay.clone(),
		runtimes: stand_in_runtimes(networks),
		additional: additional.map(|a| hex::decode(a.trim_start_matches("0x")).expect("hex")),
	})
	.expect("valid upgrade");
	let asset_hub = if relay == Network::Polkadot {
		Network::PolkadotAssetHub
	} else {
		Network::KusamaAssetHub
	};
	assert_eq!(proposal.network, asset_hub, "the proposal executes on Asset Hub");
	assert_eq!(proposal.hash, blake2_256(&proposal.encoded));
	assert_eq!(proposal.length as usize, proposal.encoded.len());
	format!("0x{}", hex::encode(proposal.encoded))
}

#[test]
fn library_builds_the_full_polkadot_upgrade() {
	assert_eq!(
		library_upgrade(Network::Polkadot, &ALL_POLKADOT_CHAINS, None),
		EXPECTED_UPGRADE_POLKADOT_ALL
	);
}

#[test]
fn library_builds_the_polkadot_upgrade_with_an_additional_call() {
	assert_eq!(
		library_upgrade(Network::Polkadot, &ALL_POLKADOT_CHAINS, Some("0x00001074657374")),
		EXPECTED_UPGRADE_POLKADOT_ALL_WITH_ADDITIONAL
	);
}

#[test]
fn library_builds_a_single_chain_upgrade() {
	assert_eq!(
		library_upgrade(Network::Polkadot, &[Network::PolkadotBulletin], None),
		EXPECTED_UPGRADE_POLKADOT_ONLY_BULLETIN
	);
	assert_eq!(
		library_upgrade(Network::Kusama, &[Network::KusamaEncointer], None),
		EXPECTED_UPGRADE_KUSAMA_ONLY_ENCOINTER
	);
}

#[test]
fn library_builds_the_full_kusama_upgrade() {
	assert_eq!(
		library_upgrade(Network::Kusama, &ALL_KUSAMA_CHAINS, None),
		EXPECTED_UPGRADE_KUSAMA_ALL
	);
	assert_eq!(
		library_upgrade(Network::Kusama, &ALL_KUSAMA_CHAINS, Some("0x00001074657374")),
		EXPECTED_UPGRADE_KUSAMA_ALL_WITH_ADDITIONAL
	);
}

#[test]
fn library_upgrade_rejects_invalid_input() {
	let upgrade = |relay, networks: &[Network], additional: Option<Vec<u8>>| {
		library_build_upgrade(UpgradeInput {
			relay,
			runtimes: stand_in_runtimes(networks),
			additional,
		})
	};
	assert_eq!(
		upgrade(Network::PolkadotAssetHub, &[Network::PolkadotAssetHub], None),
		Err(Error::NotARelayChain)
	);
	assert_eq!(
		upgrade(Network::Polkadot, &[Network::Polkadot, Network::KusamaAssetHub], None),
		Err(Error::NotInRelayFamily { relay: Network::Polkadot, network: Network::KusamaAssetHub })
	);
	assert_eq!(upgrade(Network::Polkadot, &[], None), Err(Error::NoCalls));
	assert_eq!(
		upgrade(Network::Polkadot, &[Network::Polkadot], Some(vec![0xff, 0xff, 0xff])),
		Err(Error::UndecodableCall(Network::PolkadotAssetHub))
	);
}

// Check that the library and the command line path produce the same calls for some input.
async fn assert_library_matches_cli(proposal_details: ProposalDetails) {
	let library = referendum_calls(ReferendumInput {
		proposal: get_proposal_bytes(proposal_details.proposal.clone()),
		track: proposal_details.track.clone(),
		dispatch: proposal_details.dispatch,
		fellowship_on_polkadot: proposal_details.fellowship_on_polkadot,
	})
	.expect("valid referendum");
	let cli = generate_calls(&proposal_details).await;

	let cli_call = |call: Option<(CallOrHash, u32)>| match call {
		Some((CallOrHash::Call(call), _)) => Some(CallInfo::from_runtime_call(call)),
		Some((CallOrHash::Hash(_), _)) => panic!("call length within the limit"),
		None => None,
	};
	assert_eq!(library.fellowship_preimage, cli_call(cli.preimage_for_whitelist_call));
	assert_eq!(
		library.fellowship_referendum,
		cli.fellowship_referendum_submission.map(CallInfo::from_runtime_call)
	);
	assert_eq!(Some(library.public_preimage.clone()), cli_call(cli.preimage_for_public_referendum));
	assert_eq!(
		Some(library.public_referendum.clone()),
		cli.public_referendum_submission.map(CallInfo::from_runtime_call)
	);

	// The values that describe the preimage are consistent with the calls.
	assert_eq!(library.public_preimage_hash, blake2_256(&library.public_preimage_bytes));
	assert_eq!(library.public_preimage_len as usize, library.public_preimage_bytes.len());
	assert!(library.public_preimage.encoded.ends_with(&library.public_preimage_bytes));
	assert_eq!(library.whitelisted_call_hash.is_some(), library.fellowship_referendum.is_some());
}

#[tokio::test]
async fn library_referendum_calls_match_the_cli() {
	assert_library_matches_cli(polkadot_whitelist_remark_user_input()).await;
	assert_library_matches_cli(polkadot_staking_validator_user_input()).await;
	assert_library_matches_cli(polkadot_root_remark_user_input()).await;
	assert_library_matches_cli(kusama_whitelist_remark_user_input()).await;
	assert_library_matches_cli(kusama_whitelist_polkadot_fellowship_user_input()).await;
	assert_library_matches_cli(kusama_staking_validator_user_input()).await;
	assert_library_matches_cli(kusama_root_remark_user_input()).await;
}

#[test]
fn library_builds_a_polkadot_whitelisted_caller_referendum() {
	// `system.remark("opengov-submit test")`
	let proposal = hex::decode("00004c6f70656e676f762d7375626d69742074657374").expect("hex");
	let calls = referendum_calls(ReferendumInput {
		proposal: proposal.clone(),
		track: NetworkTrack::parse("polkadot", "whitelistedcaller").expect("valid track"),
		dispatch: DispatchTimeWrapper::After(10),
		fellowship_on_polkadot: false,
	})
	.expect("valid referendum");

	// The same encodings as `it_starts_polkadot_fellowship_referenda_correctly`.
	assert_eq!(calls.fellowship_preimage, None, "the Fellowship proposal is inlined");
	let fellowship_referendum = calls.fellowship_referendum.expect("whitelisting needs one");
	assert_eq!(fellowship_referendum.network, Network::PolkadotCollectives);
	assert_eq!(
		hex::encode(&fellowship_referendum.encoded),
		"3d003e0201cc1f0005010100a10f05082f00000603008840008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534010a000000"
	);
	assert_eq!(calls.public_preimage.network, Network::PolkadotAssetHub);
	assert_eq!(
		hex::encode(&calls.public_preimage.encoded),
		"050060400300004c6f70656e676f762d7375626d69742074657374"
	);
	assert_eq!(calls.public_referendum.network, Network::PolkadotAssetHub);
	assert_eq!(
		hex::encode(&calls.public_referendum.encoded),
		"3e003f0d02a322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e18000000010a000000"
	);

	// The public referendum looks up `whitelist.dispatch_whitelisted_call_with_preimage(proposal)`.
	assert_eq!(
		hex::encode(&calls.public_preimage_bytes),
		"400300004c6f70656e676f762d7375626d69742074657374"
	);
	assert_eq!(
		hex::encode(calls.public_preimage_hash),
		"a322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e"
	);
	assert_eq!(calls.public_preimage_len, 24);
	// The Fellowship whitelists the hash of the proposal itself.
	assert_eq!(calls.whitelisted_call_hash, Some(blake2_256(&proposal)));
	assert_eq!(
		hex::encode(blake2_256(&proposal)),
		"8821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534"
	);
}

#[tokio::test]
async fn library_builds_a_polkadot_root_referendum() {
	// `system.remark("opengov-submit test")`
	let proposal = hex::decode("00004c6f70656e676f762d7375626d69742074657374").expect("hex");
	let calls = referendum_calls(ReferendumInput {
		proposal: proposal.clone(),
		track: NetworkTrack::parse("Polkadot", "Root").expect("valid track"),
		dispatch: DispatchTimeWrapper::After(10),
		fellowship_on_polkadot: false,
	})
	.expect("valid referendum");

	assert_eq!(calls.fellowship_preimage, None);
	assert_eq!(calls.fellowship_referendum, None);
	assert_eq!(calls.whitelisted_call_hash, None);
	assert_eq!(calls.public_preimage_bytes, proposal);
	assert_eq!(calls.public_preimage_hash, blake2_256(&proposal));
	assert_eq!(calls.public_preimage_len, 22);
	assert_eq!(
		hex::encode(&calls.public_preimage.encoded),
		"05005800004c6f70656e676f762d7375626d69742074657374"
	);

	// The same calls as the command line tool gives, which
	// `it_starts_polkadot_root_referenda_correctly` checks the encoding of.
	let cli = generate_calls(&polkadot_root_remark_user_input()).await;
	assert_eq!(
		Some(calls.public_referendum),
		cli.public_referendum_submission.map(CallInfo::from_runtime_call)
	);
}

#[test]
fn library_inlines_short_fellowship_proposals_and_looks_up_long_ones() {
	use crate::polkadot_collectives::runtime_types::frame_system::pallet::Call as SystemCall;
	use crate::referendum::polkadot_fellowship_proposal;
	use crate::CollectivesRuntimeCall;

	// A Collectives call of exactly `length` bytes: `system.remark` has two bytes of indices and,
	// at these sizes, a two byte length prefix.
	let call_of_length = |length: usize| {
		let call = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
			CollectivesRuntimeCall::System(SystemCall::remark { remark: vec![7u8; length - 4] }),
		));
		assert_eq!(call.length as usize, length);
		call
	};

	// At the limit of 128 bytes the proposal is inlined: `Bounded::Inline` is variant 1.
	let short = call_of_length(128);
	let (referendum, preimage) = polkadot_fellowship_proposal(short.clone());
	assert_eq!(preimage, None);
	assert_eq!(referendum.network, Network::PolkadotCollectives);
	let mut expected = hex::decode("3d003e0201").expect("hex");
	expected.extend([0x01, 0x02]); // compact(128)
	expected.extend(&short.encoded);
	expected.extend(hex::decode("010a000000").expect("hex")); // After(10)
	assert_eq!(referendum.encoded, expected);

	// One byte more and it is noted as a preimage and looked up: `Bounded::Lookup` is variant 2.
	let long = call_of_length(129);
	let (referendum, preimage) = polkadot_fellowship_proposal(long.clone());
	let preimage = preimage.expect("a long proposal needs a preimage");
	assert_eq!(preimage.network, Network::PolkadotCollectives);
	let mut expected = hex::decode("2b00").expect("hex"); // preimage.note_preimage
	expected.extend([0x05, 0x02]); // compact(129)
	expected.extend(&long.encoded);
	assert_eq!(preimage.encoded, expected);
	let mut expected = hex::decode("3d003e0202").expect("hex");
	expected.extend(long.hash);
	expected.extend(129u32.to_le_bytes());
	expected.extend(hex::decode("010a000000").expect("hex")); // After(10)
	assert_eq!(referendum.encoded, expected);
}

#[test]
fn library_returns_full_calls_whatever_their_length() {
	// The command line tool prints a hash for this input, because of its output length limit.
	let proposal_details = limited_length_user_input();
	let calls = referendum_calls(ReferendumInput {
		proposal: get_proposal_bytes(proposal_details.proposal),
		track: proposal_details.track,
		dispatch: proposal_details.dispatch,
		fellowship_on_polkadot: false,
	})
	.expect("valid referendum");
	assert!(calls.public_preimage.length > proposal_details.output_len_limit);
	assert_eq!(
		hex::encode(&calls.public_preimage.encoded),
		"05005800004c6f70656e676f762d7375626d69742074657374"
	);
}

#[test]
fn library_referendum_rejects_invalid_input() {
	let referendum = |proposal: &[u8], network, track, fellowship_on_polkadot| {
		referendum_calls(ReferendumInput {
			proposal: proposal.to_vec(),
			track: NetworkTrack::parse(network, track).expect("valid track"),
			dispatch: DispatchTimeWrapper::After(10),
			fellowship_on_polkadot,
		})
	};
	// A proposal that has to be wrapped in a whitelist call must decode.
	assert_eq!(
		referendum(&[0xff, 0xff, 0xff], "polkadot", "whitelistedcaller", false),
		Err(Error::UndecodableCall(Network::PolkadotAssetHub))
	);
	assert_eq!(
		referendum(&[0xff, 0xff, 0xff], "kusama", "whitelistedcaller", false),
		Err(Error::UndecodableCall(Network::KusamaAssetHub))
	);
	// The Polkadot Fellowship is only an option for Kusama whitelisted calls.
	assert_eq!(
		referendum(&[0, 0, 0], "polkadot", "whitelistedcaller", true),
		Err(Error::FellowshipNotSupported)
	);
	assert_eq!(referendum(&[0, 0, 0], "kusama", "root", true), Err(Error::FellowshipNotSupported));
	assert!(referendum(&[0, 0, 0], "kusama", "whitelistedcaller", true).is_ok());

	assert!(matches!(NetworkTrack::parse("polkadot", "nonsense"), Err(Error::UnsupportedTrack)));
	assert!(matches!(NetworkTrack::parse("westend", "root"), Err(Error::NotARelayChain)));
	assert!(matches!(
		NetworkTrack::parse("kusama", "General-Admin"),
		Ok(NetworkTrack::Kusama(KusamaAssetHubOpenGovOrigin::GeneralAdmin))
	));

	use DispatchTimeWrapper::{After, At};
	assert_eq!(DispatchTimeWrapper::from_at_after(None, None), Ok(None));
	assert_eq!(DispatchTimeWrapper::from_at_after(Some(5), None), Ok(Some(At(5))));
	assert_eq!(DispatchTimeWrapper::from_at_after(None, Some(6)), Ok(Some(After(6))));
	assert_eq!(DispatchTimeWrapper::from_at_after(Some(5), Some(6)), Err(Error::BothAtAndAfter));
}

#[test]
fn library_checks_and_hashes_runtimes() {
	use crate::check_and_hash_runtime;
	const ZSTD_PREFIX: [u8; 8] = [82, 188, 83, 118, 70, 219, 142, 5];
	let blob_of_size = |size: usize| {
		let mut blob = ZSTD_PREFIX.to_vec();
		blob.resize(size, 0);
		blob
	};

	let runtime = blob_of_size(100 * 1024);
	assert_eq!(check_and_hash_runtime(&runtime, false), Ok(blake2_256(&runtime)));
	assert_eq!(
		check_and_hash_runtime(&blob_of_size(100 * 1024 - 1), false),
		Err(Error::RuntimeSize(100 * 1024 - 1))
	);
	assert_eq!(
		check_and_hash_runtime(&blob_of_size(10 * 1024 * 1024 + 1), false),
		Err(Error::RuntimeSize(10 * 1024 * 1024 + 1))
	);
	let not_compressed = vec![0u8; 200 * 1024];
	assert_eq!(check_and_hash_runtime(&not_compressed, false), Err(Error::RuntimeNotCompressed));
	// With the checks skipped, anything is hashed.
	assert_eq!(check_and_hash_runtime(&not_compressed, true), Ok(blake2_256(&not_compressed)));
	assert_eq!(check_and_hash_runtime(b"tiny", true), Ok(blake2_256(b"tiny")));
}

#[test]
fn library_names_networks_and_runtime_files() {
	for network in Network::ALL {
		assert_eq!(Network::from_runtime_name(network.runtime_name()), Ok(network));
	}
	assert_eq!(
		Network::from_runtime_name("westend"),
		Err(Error::UnknownNetwork(String::from("westend")))
	);

	assert_eq!(
		crate::runtime_file_name(&Network::PolkadotAssetHub, "2.6.0").as_deref(),
		Ok("asset-hub-polkadot_runtime-v2006000.compact.compressed.wasm")
	);
	// A leading `v`, as in the release's tag, is accepted.
	assert_eq!(
		crate::runtime_file_name(&Network::Kusama, "v1.10.3").as_deref(),
		Ok("kusama_runtime-v1010003.compact.compressed.wasm")
	);
	assert_eq!(crate::runtime_file_name(&Network::Polkadot, "2.6"), Err(Error::NotSemver));
}

#[test]
fn library_validates_versions() {
	use crate::semver_to_intver;
	assert_eq!(semver_to_intver("2.6.0").as_deref(), Ok("2006000"));
	assert_eq!(semver_to_intver("1.999.999").as_deref(), Ok("1999999"));
	// Leading zeros are not significant.
	assert_eq!(semver_to_intver("1.02.003").as_deref(), Ok("1002003"));
	assert_eq!(semver_to_intver("01.0002.3").as_deref(), Ok("1002003"));

	for not_semver in
		["", "1", "1.2", "1.2.3.4", "1.x.3", "x.2.3", "1.2.x", "1..3", ".2.3", "1.2.", "1.2.3-rc1"]
	{
		assert_eq!(semver_to_intver(not_semver), Err(Error::NotSemver), "{not_semver}");
	}
	for too_large in ["1.1000.0", "1.0.1000", "1.12345.0", "99999999999.0.0"] {
		assert_eq!(semver_to_intver(too_large), Err(Error::VersionOutOfRange), "{too_large}");
	}
	// The leading `v` of a release tag is only accepted where a file name is asked for.
	assert_eq!(semver_to_intver("v2.6.0"), Err(Error::NotSemver));
}

#[test]
fn library_rejects_an_empty_proposal_on_every_track() {
	for (network, track) in [
		("polkadot", "root"),
		("polkadot", "whitelistedcaller"),
		("polkadot", "staking-admin"),
		("kusama", "root"),
		("kusama", "whitelistedcaller"),
		("kusama", "treasurer"),
	] {
		let calls = referendum_calls(ReferendumInput {
			proposal: Vec::new(),
			track: NetworkTrack::parse(network, track).expect("valid track"),
			dispatch: DispatchTimeWrapper::After(10),
			fellowship_on_polkadot: false,
		});
		assert_eq!(calls, Err(Error::EmptyProposal), "{network} {track}");
	}
}

// Expected encodings ------------------------------------------------------------------------------

const EXPECTED_UPGRADE_POLKADOT_ALL: &str =
	"0x28041c1f0005010005082f0000060200880009c109b832656a7c6d63edcd91264b1d779978c0d1b166aad9de2167abeb6b834f00096c832cee8087822c18bd75162c0ee6ed7424497f94c35f1c47e61c9bc32732641f0005010100a50f05082f00000602008800093f218f7bafe960dda4d6e1298946036ccacd68712a84174a0f1eab39fcc1d9041f0005010100a90f05082f00000602008800095b79128e0a8d86a651cf100eda8ef42a79fddc08dd860d0868755152f34317471f0005010100b10f05082f0000060200880009791c0b0e4fb6bebf0290f8cdb548bfe671381c60edb7329e04310778e2cc39251f0005010100b50f05082f00000602008800091b5169d970cd169ba37167bf6b8843bb6da323b7b5319c1d1e1705c13cc6ef341f0005010100c90f05082f0000060200880009d71d8efe5e48431864aa41d98b6d40bc5f2bcda4253b33dc4913810bac323c80";
const EXPECTED_UPGRADE_POLKADOT_ALL_WITH_ADDITIONAL: &str =
	"0x2804201f0005010005082f0000060200880009c109b832656a7c6d63edcd91264b1d779978c0d1b166aad9de2167abeb6b834f00096c832cee8087822c18bd75162c0ee6ed7424497f94c35f1c47e61c9bc32732641f0005010100a50f05082f00000602008800093f218f7bafe960dda4d6e1298946036ccacd68712a84174a0f1eab39fcc1d9041f0005010100a90f05082f00000602008800095b79128e0a8d86a651cf100eda8ef42a79fddc08dd860d0868755152f34317471f0005010100b10f05082f0000060200880009791c0b0e4fb6bebf0290f8cdb548bfe671381c60edb7329e04310778e2cc39251f0005010100b50f05082f00000602008800091b5169d970cd169ba37167bf6b8843bb6da323b7b5319c1d1e1705c13cc6ef341f0005010100c90f05082f0000060200880009d71d8efe5e48431864aa41d98b6d40bc5f2bcda4253b33dc4913810bac323c8000001074657374";
const EXPECTED_UPGRADE_POLKADOT_ONLY_BULLETIN: &str =
	"0x1f0005010100c90f05082f0000060200880009d71d8efe5e48431864aa41d98b6d40bc5f2bcda4253b33dc4913810bac323c80";
const EXPECTED_UPGRADE_KUSAMA_ALL: &str =
	"0x2804181f0005010005082f00000602008800094f82394ca4332435884afb85bc9d430fe2e3a206f5f4033e646b7b69666bf29a0009b5320cd5a0078cf14e68264087529d30522807669b052149a383a075f43439091f0005010100a50f05082f000006020088000998a75f41b2d1bea3355b0a373977932e2593372327fb3e29f425ff97fc3b8bda1f0005010100a90f05082f00000602008800090220f5fe749ec7bf767208aa7e9f45e031800c425d75c2beae2923994f57a4851f0005010100b10f05082f0000060200880009c986f5cf278bec87a3ba7e03bd115c3324e392c3a03c2542fe996345801c72701f0005010100b50f05082f00000602008800096f36cc02069f6b975df3719cf5e498dd98b8123d5d5db81eddaf85126cf30a1a";
const EXPECTED_UPGRADE_KUSAMA_ALL_WITH_ADDITIONAL: &str =
	"0x28041c1f0005010005082f00000602008800094f82394ca4332435884afb85bc9d430fe2e3a206f5f4033e646b7b69666bf29a0009b5320cd5a0078cf14e68264087529d30522807669b052149a383a075f43439091f0005010100a50f05082f000006020088000998a75f41b2d1bea3355b0a373977932e2593372327fb3e29f425ff97fc3b8bda1f0005010100a90f05082f00000602008800090220f5fe749ec7bf767208aa7e9f45e031800c425d75c2beae2923994f57a4851f0005010100b10f05082f0000060200880009c986f5cf278bec87a3ba7e03bd115c3324e392c3a03c2542fe996345801c72701f0005010100b50f05082f00000602008800096f36cc02069f6b975df3719cf5e498dd98b8123d5d5db81eddaf85126cf30a1a00001074657374";
const EXPECTED_UPGRADE_KUSAMA_ONLY_ENCOINTER: &str =
	"0x1f0005010100a50f05082f000006020088000998a75f41b2d1bea3355b0a373977932e2593372327fb3e29f425ff97fc3b8bda";
const EXPECTED_ADD_INVULNERABLES_BATCHES: [(&str, &str); 10] = [
	(
		"asset-hub-polkadot",
		"0x2804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"bridge-hub-polkadot",
		"0x1f0005010100a90f05082f00000603001d012804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"collectives-polkadot",
		"0x1f0005010100a50f05082f00000603001d012804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"people-polkadot",
		"0x1f0005010100b10f05082f00000603001d012804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"coretime-polkadot",
		"0x1f0005010100b50f05082f00000603001d012804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"bulletin-polkadot",
		"0x1f0005010100c90f05082f00000603001d010604081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"asset-hub-kusama",
		"0x2804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"bridge-hub-kusama",
		"0x1f0005010100a90f05082f00000603001d012804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"people-kusama",
		"0x1f0005010100b10f05082f00000603001d012804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
	(
		"coretime-kusama",
		"0x1f0005010100b50f05082f00000603001d012804081505010101010101010101010101010101010101010101010101010101010101010115050202020202020202020202020202020202020202020202020202020202020202",
	),
];
