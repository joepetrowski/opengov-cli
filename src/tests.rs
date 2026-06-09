use crate::get_proposal_bytes;
use crate::polkadot_asset_hub::runtime_types::frame_system::pallet::Call as PolkadotAssetHubSystemCall;
use crate::polkadot_relay::runtime_types::frame_system::pallet::Call as PolkadotRelaySystemCall;
use crate::register_system_para::{
	build_polkadot_register_calls, RegisterSystemParaParams,
};
use crate::{
	build_upgrade, submit_referendum::generate_calls, CallInfo, CallOrHash,
	KusamaAssetHubOpenGovOrigin, Network, NetworkRuntimeCall, PolkadotAssetHubOpenGovOrigin,
	PolkadotAssetHubRuntimeCall, PolkadotRuntimeCall, ProposalDetails, UpgradeArgs,
	VersionedNetwork,
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

// ---------------------------------------------------------------------------
// Register System Para tests
// ---------------------------------------------------------------------------

/// Small synthetic test params (avoids needing real WASM files).
fn small_register_params() -> RegisterSystemParaParams {
	RegisterSystemParaParams {
		wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x00], // minimal wasm-like
		genesis_head_bytes: vec![0x01, 0x02, 0x03, 0x04],
		para_id: 1234,
		manager_bytes: [0u8; 32], // zero account
		deposit: 0,
		ref_time: 60_000_000_000,
		proof_size: 10_000,
		assign_core: None,
		delay_whitelist_dispatch_relay: None,
	}
}

#[test]
fn register_force_register_call_encodes_correctly() {
	let params = small_register_params();
	let output = build_polkadot_register_calls(params);

	// force_register is a relay chain call
	assert_eq!(output.force_register_info.network, Network::Polkadot);
	// Encoded bytes should start with Registrar pallet + force_register call index
	assert!(output.force_register_info.length > 0);
	// Hash should be 32 bytes
	assert_eq!(output.force_register_info.hash.len(), 32);
	// Should be decodable back to a Polkadot call
	assert!(output.force_register_info.get_polkadot_call().is_ok());
}

#[test]
fn register_proposal_is_asset_hub_call() {
	let params = small_register_params();
	let output = build_polkadot_register_calls(params);

	// Proposal is an Asset Hub call (batch_all)
	assert_eq!(output.proposal.network, Network::PolkadotAssetHub);
	// Should be decodable as a Polkadot Asset Hub call
	assert!(output.proposal.get_polkadot_asset_hub_call().is_ok());
}

#[test]
fn register_proposal_is_within_ump_limit() {
	let params = small_register_params();
	let output = build_polkadot_register_calls(params);

	// The proposal must be small enough to eventually travel as an XCM UMP message.
	// The proposal itself is a batch_all of two polkadotXcm.send calls.
	// It should be well under 128 KB (the UMP limit).
	assert!(
		output.proposal.length < 1024,
		"Proposal is {} bytes, expected < 1024 for small inputs",
		output.proposal.length
	);
}

#[test]
fn register_whitelist_hash_matches_force_register_hash() {
	let params = small_register_params();
	let output = build_polkadot_register_calls(params);

	// Verify the proposal is decodable and contains the force_register hash.
	let _proposal_call = output.proposal.get_polkadot_asset_hub_call().expect("valid AH call");

	// The proposal is batch_all([xcm_send_whitelist, xcm_send_dispatch]).
	// We verify the force_register hash is embedded in the proposal by checking
	// that the proposal bytes contain the force_register hash.
	let hash_bytes = output.force_register_info.hash;
	let proposal_bytes = &output.proposal.encoded;

	// The hash should appear twice in the proposal (once in whitelist_call, once in dispatch)
	let hash_occurrences = proposal_bytes
		.windows(32)
		.filter(|w| *w == hash_bytes)
		.count();
	assert_eq!(
		hash_occurrences, 2,
		"force_register hash should appear exactly twice in proposal (whitelist + dispatch)"
	);
}

#[test]
fn register_delay_whitelist_dispatch_relay_wraps_dispatch_in_scheduler() {
	let mut params = small_register_params();
	params.delay_whitelist_dispatch_relay = Some(100);
	let output_delayed = build_polkadot_register_calls(params);

	let output_normal = build_polkadot_register_calls(small_register_params());

	// The relay preimage should be the same (force_register is unchanged)
	assert_eq!(
		output_delayed.force_register_info.encoded, output_normal.force_register_info.encoded,
		"Relay preimage should be identical regardless of --free-preimage-delay"
	);

	// The AH proposal should be larger (dispatch wrapped in Scheduler.schedule_after)
	assert!(
		output_delayed.proposal.length > output_normal.proposal.length,
		"Proposal with delay ({}) should be larger than without ({})",
		output_delayed.proposal.length,
		output_normal.proposal.length,
	);

	// Both should still be valid AH calls
	assert!(output_delayed.proposal.get_polkadot_asset_hub_call().is_ok());
}

#[test]
fn register_with_assign_core_has_three_expect_transact_status() {
	let mut params = small_register_params();
	params.assign_core = Some(67);
	let output = build_polkadot_register_calls(params);

	// With assign_core, there are 3 XCM messages: whitelist, dispatch, force_reserve.
	// Each should have ExpectTransactStatus(Success).
	// V5 enum variant (0x05) followed by compact(3) = 0x0c means 3 instructions per XCM.
	let v5_three_instructions: [u8; 2] = [0x05, 0x0c];
	let occurrences = output
		.proposal
		.encoded
		.windows(2)
		.filter(|w| *w == v5_three_instructions)
		.count();
	assert_eq!(
		occurrences, 3,
		"Each of the 3 XCM messages should have 3 instructions (UnpaidExecution + Transact + ExpectTransactStatus)"
	);
}

#[test]
fn register_with_assign_core_adds_xcm_to_coretime() {
	let params = small_register_params();
	let output_without = build_polkadot_register_calls(params);

	let mut params_with_core = small_register_params();
	params_with_core.assign_core = Some(67);
	let output_with = build_polkadot_register_calls(params_with_core);

	// The relay preimage (force_register) should be identical — assign_core goes via
	// separate XCM to Coretime chain, not in the relay batch.
	assert_eq!(
		output_with.force_register_info.encoded, output_without.force_register_info.encoded,
		"Relay preimage should be the same with or without assign_core"
	);

	// The AH proposal should be larger (extra XCM send to Coretime chain)
	assert!(
		output_with.proposal.length > output_without.proposal.length,
		"AH proposal with assign_core ({}) should be larger than without ({})",
		output_with.proposal.length,
		output_without.proposal.length,
	);

	// Both proposals should still be valid AH calls
	assert!(output_with.proposal.get_polkadot_asset_hub_call().is_ok());
}

#[test]
fn register_deterministic_output() {
	// Same inputs should produce identical output.
	let output1 = build_polkadot_register_calls(small_register_params());
	let output2 = build_polkadot_register_calls(small_register_params());

	assert_eq!(output1.force_register_info.encoded, output2.force_register_info.encoded);
	assert_eq!(output1.force_register_info.hash, output2.force_register_info.hash);
	assert_eq!(output1.proposal.encoded, output2.proposal.encoded);
}

#[tokio::test]
async fn register_proposal_works_with_submit_referendum() {
	// Build the registration proposal, then feed it into submit-referendum
	// to verify the full pipeline works.
	let params = small_register_params();
	let output = build_polkadot_register_calls(params);

	// Create a ProposalDetails as if the user piped the proposal into submit-referendum
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;

	let proposal_hex = format!("0x{}", hex::encode(&output.proposal.encoded));

	let proposal_details = ProposalDetails {
		proposal: proposal_hex,
		track: Polkadot(PolkadotAssetHubOpenGovOrigin::WhitelistedCaller),
		dispatch: After(10),
		output: CallData,
		output_len_limit: 1_000,
		print_batch: false,
		use_light_client: false,
		fellowship_on_polkadot: false,
	};

	let calls = generate_calls(&proposal_details).await;

	// Should generate fellowship referendum (on Collectives)
	assert!(
		calls.fellowship_referendum_submission.is_some(),
		"must generate fellowship referendum"
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

	// Should generate public referendum preimage (on Asset Hub)
	assert!(
		calls.preimage_for_public_referendum.is_some(),
		"must generate public referendum preimage"
	);

	// Should generate public referendum (on Asset Hub)
	assert!(
		calls.public_referendum_submission.is_some(),
		"must generate public referendum"
	);
	if let Some(ref public_ref) = calls.public_referendum_submission {
		match public_ref {
			NetworkRuntimeCall::PolkadotAssetHub(_) => (),
			other => panic!(
				"Public referendum should be on PolkadotAssetHub, got {:?}",
				std::mem::discriminant(other)
			),
		}
	}
}

#[test]
fn register_xcm_includes_expect_transact_status() {
	let params = small_register_params();
	let output = build_polkadot_register_calls(params);

	// Build a proposal WITHOUT ExpectTransactStatus to compare.
	// We verify by checking that each XCM has 3 instructions (not 2):
	// [UnpaidExecution, Transact, ExpectTransactStatus].
	//
	// XCM Vec<Instruction> is SCALE-encoded with a compact length prefix.
	// 3 instructions = compact(3) = 0x0c as the first byte of the XCM body.
	// If ExpectTransactStatus were missing, it would be compact(2) = 0x08.
	//
	// The proposal is batch_all([send(xcm1), send(xcm2)]).
	// Each send's message contains V5(Xcm(vec![...3 instructions...])).
	// The compact(3) = 0x0c prefix should appear exactly twice.

	// Count how many XCM instruction vectors have length 3 (0x0c prefix).
	// We look for the byte pattern that represents "3 instructions in a Vec".
	// In the SCALE encoding of Xcm(Vec<Instruction>), the vec length comes right
	// after the V5 enum variant tag.
	//
	// V5 variant index in VersionedXcm is 5 (V2=0, V3=1, V4=2... but encoded as
	// actual enum index which is 05 for V5). The pattern is:
	// 0x05 (V5) followed by 0x0c (compact 3 = three instructions)
	let v5_three_instructions: [u8; 2] = [0x05, 0x0c];
	let occurrences = output
		.proposal
		.encoded
		.windows(2)
		.filter(|w| *w == v5_three_instructions)
		.count();
	assert_eq!(
		occurrences, 2,
		"Each XCM message should have 3 instructions (UnpaidExecution + Transact + ExpectTransactStatus)"
	);
}

#[test]
fn register_dispatch_whitelisted_has_correct_length() {
	let params = small_register_params();
	let output = build_polkadot_register_calls(params);

	// The dispatch_whitelisted_call inside the XCM should reference the correct
	// call_encoded_len matching the force_register call length.
	let force_register_len = output.force_register_info.length;

	// call_encoded_len is a u32, SCALE-encoded as 4 bytes little-endian
	let len_le = force_register_len.to_le_bytes();
	assert!(
		output.proposal.encoded.windows(4).any(|w| w == len_le),
		"force_register length ({}, le bytes {:?}) should appear in the proposal",
		force_register_len, len_le,
	);
}

// ---------------------------------------------------------------------------
// Regression tests: register-system-para vs composed primitives
// ---------------------------------------------------------------------------

/// Verify that composing primitives (xcm-force-register + xcm-force-reserve + batch-ah)
/// produces byte-identical output to `build_polkadot_register_calls`.
#[test]
fn composed_primitives_match_register_system_para() {
	use crate::primitives::{
		batch_all_on_ah, build_dispatch_whitelisted_call, build_force_register_call,
		build_force_reserve_call, build_whitelist_call, wrap_in_xcm_send_from_ah,
		ForceRegisterParams, XcmDest,
	};

	let mut params = small_register_params();
	params.assign_core = Some(77);
	let monolithic = build_polkadot_register_calls(RegisterSystemParaParams {
		wasm_bytes: params.wasm_bytes.clone(),
		genesis_head_bytes: params.genesis_head_bytes.clone(),
		para_id: params.para_id,
		manager_bytes: params.manager_bytes,
		deposit: params.deposit,
		ref_time: params.ref_time,
		proof_size: params.proof_size,
		assign_core: params.assign_core,
		delay_whitelist_dispatch_relay: None,
	});

	// Compose the same proposal manually using the primitives.
	let force_register_info = build_force_register_call(ForceRegisterParams {
		wasm_bytes: params.wasm_bytes.clone(),
		genesis_head_bytes: params.genesis_head_bytes.clone(),
		para_id: params.para_id,
		manager_bytes: params.manager_bytes,
		deposit: params.deposit,
	});
	let whitelist_info = build_whitelist_call(force_register_info.hash);
	let xcm_whitelist = wrap_in_xcm_send_from_ah(XcmDest::Relay, whitelist_info.encoded);

	let dispatch_call = build_dispatch_whitelisted_call(
		force_register_info.hash,
		force_register_info.length,
		params.ref_time,
		params.proof_size,
	);
	let dispatch_info = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(dispatch_call));
	let xcm_dispatch = wrap_in_xcm_send_from_ah(XcmDest::Relay, dispatch_info.encoded);

	let fr_info = build_force_reserve_call(params.para_id, 77);
	let xcm_reserve = wrap_in_xcm_send_from_ah(XcmDest::Sibling(1005), fr_info.encoded.clone());

	let composed = batch_all_on_ah(vec![xcm_whitelist, xcm_dispatch, xcm_reserve]);

	// The composed proposal must be byte-identical to the monolithic build.
	assert_eq!(
		composed.encoded, monolithic.proposal.encoded,
		"composed proposal bytes must match register-system-para output"
	);
	assert_eq!(composed.hash, monolithic.proposal.hash);

	// And the force_register preimage / force_reserve call must match too.
	assert_eq!(force_register_info.encoded, monolithic.force_register_info.encoded);
	assert_eq!(
		fr_info.encoded,
		monolithic.force_reserve_info.as_ref().expect("force_reserve_info set").encoded
	);
}
