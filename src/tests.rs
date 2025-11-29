use crate::get_proposal_bytes;
use crate::polkadot_asset_hub::runtime_types::frame_system::pallet::Call as PolkadotAssetHubSystemCall;
use crate::polkadot_relay::runtime_types::frame_system::pallet::Call as PolkadotRelaySystemCall;
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
