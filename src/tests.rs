use crate::get_proposal_bytes;
use crate::polkadot_asset_hub::runtime_types::frame_system::pallet::Call as PolkadotAssetHubSystemCall;
use crate::polkadot_relay::runtime_types::frame_system::pallet::Call as PolkadotRelaySystemCall;
use crate::{
	blake2_256, build_upgrade, submit_referendum::generate_calls, CallInfo, CallOrHash,
	KusamaAssetHubOpenGovOrigin, KusamaAssetHubRuntimeCall, Network, NetworkRuntimeCall,
	PolkadotAssetHubOpenGovOrigin, PolkadotAssetHubRuntimeCall, PolkadotRuntimeCall,
	ProposalDetails, UpgradeArgs, VersionedNetwork,
};

fn polkadot_whitelist_remark_user_input() -> ProposalDetails {
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
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
	use crate::add_invulnerables::{build_add_invulnerables_call, parse_accounts};

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
	use crate::add_invulnerables::build_add_invulnerables_call;

	let a = [1u8; 32];
	let b = [2u8; 32];
	let call = build_add_invulnerables_call(&Network::PolkadotBulletin, &[a, b]);
	// utility (pallet 6) . forceBatch (call 4) ++ compact(2) ++ two addInvulnerable calls
	let expected = format!("0604081505{}1505{}", hex::encode(a), hex::encode(b));
	assert_eq!(hex::encode(&call.encoded), expected);
}

#[test]
fn add_invulnerables_wraps_for_staking_admin_via_xcm() {
	use crate::add_invulnerables::{build_add_invulnerables_call, wrap_for_governance};
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
	use crate::add_invulnerables::{build_add_invulnerables_call, wrap_for_governance};

	let target = build_add_invulnerables_call(&Network::PolkadotAssetHub, &[[1u8; 32]]);
	let proposal = wrap_for_governance(&target);
	assert_eq!(proposal.network, Network::PolkadotAssetHub);
	assert_eq!(proposal.encoded, target.encoded);
}

#[test]
#[should_panic(expected = "listed more than once")]
fn add_invulnerables_rejects_duplicate_accounts() {
	use crate::add_invulnerables::parse_accounts;

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
			let runtime = format!("fake runtime {}", build_upgrade::runtime_name(&chain.network));
			build_upgrade::authorize_upgrade_call(&chain.network, blake2_256(runtime.as_bytes()))
		})
		.collect();
	let proposal = build_upgrade::construct_batch(&details, calls).await;
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
		let call = build_upgrade::authorize_upgrade_call(&network, code_hash);
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
		.map(|(i, chain)| build_upgrade::authorize_upgrade_call(&chain.network, [i as u8; 32]))
		.collect();
	let proposal = build_upgrade::construct_batch(&details, calls.clone()).await;

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
	use build_upgrade::semver_to_intver;
	assert_eq!(semver_to_intver("2.5.3"), "2005003");
	assert_eq!(semver_to_intver("1.10.0"), "1010000");
	assert_eq!(semver_to_intver("1.0.0"), "1000000");
	assert_eq!(semver_to_intver("12.345.678"), "12345678");
}

#[test]
#[should_panic(expected = "not semver")]
fn semver_conversion_rejects_non_semver() {
	build_upgrade::semver_to_intver("1.2");
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
	use crate::add_invulnerables::{build_add_invulnerables_call, parse_network};
	let who = [1u8; 32];
	for (name, _) in EXPECTED_ADD_INVULNERABLES_BATCHES {
		let call = build_add_invulnerables_call(&parse_network(name), &[who]);
		// collatorSelection (pallet 21 on every system chain) . addInvulnerable (call 5) ++ account
		assert_eq!(call.encoded, [&[21u8, 5][..], &who[..]].concat(), "{name}");
	}
}

#[test]
fn add_invulnerables_proposals_encode_correctly_on_every_chain() {
	use crate::add_invulnerables::{
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
	crate::add_invulnerables::parse_network("encointer-kusama");
}

#[test]
#[should_panic(expected = "Relay chains do not have collator selection")]
fn add_invulnerables_rejects_relay_chains() {
	crate::add_invulnerables::parse_network("polkadot");
}

#[test]
fn add_invulnerables_parses_kusama_addresses() {
	use crate::add_invulnerables::parse_accounts;

	// Alice, with the Kusama SS58 prefix (2) and with the generic Polkadot one (0).
	let alice = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
	let inputs = [String::from("HNZata7iMYWmk5RvZRTiAsSDhV8366zq2YGb3tLH5Upf74F")];
	assert_eq!(hex::encode(parse_accounts(&inputs, &Network::KusamaPeople)[0]), alice);
	// A mismatched prefix only warns: the public key is what matters on chain.
	let inputs = [String::from("15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5")];
	assert_eq!(hex::encode(parse_accounts(&inputs, &Network::KusamaPeople)[0]), alice);
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
