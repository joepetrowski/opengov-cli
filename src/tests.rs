use crate::get_proposal_bytes;
use crate::polkadot_relay::runtime_types::frame_system::pallet::Call as PolkadotRelaySystemCall;
use crate::{
	build_upgrade, submit_referendum::generate_calls, CallInfo, CallOrHash, KusamaOpenGovOrigin,
	Network, NetworkRuntimeCall, PolkadotOpenGovOrigin, PolkadotRuntimeCall, ProposalDetails,
	UpgradeArgs, VersionedNetwork, Weight,
};

fn polkadot_whitelist_remark_user_input() -> ProposalDetails {
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: Polkadot(PolkadotOpenGovOrigin::WhitelistedCaller),
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		transact_weight_override: Some(Weight { ref_time: 1_000_000_000, proof_size: 1_000_000 }),
	}
}

fn polkadot_staking_validator_user_input() -> ProposalDetails {
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
	ProposalDetails {
		// `staking.increase_validator_count(50)`
		proposal: String::from("0x070ac8"),
		track: Polkadot(PolkadotOpenGovOrigin::StakingAdmin),
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		transact_weight_override: Some(Weight { ref_time: 1_000_000_000, proof_size: 1_000_000 }),
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
		transact_weight_override: Some(Weight { ref_time: 1_000_000_000, proof_size: 1_000_000 }),
	}
}

fn kusama_whitelist_remark_user_input() -> ProposalDetails {
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: Kusama(KusamaOpenGovOrigin::WhitelistedCaller),
		dispatch: At(100_000_000),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		transact_weight_override: Some(Weight { ref_time: 1_000_000_000, proof_size: 1_000_000 }),
	}
}

fn kusama_staking_validator_user_input() -> ProposalDetails {
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
	ProposalDetails {
		// `staking.increase_validator_count(50)`
		proposal: String::from("0x060ac8"),
		track: Kusama(KusamaOpenGovOrigin::StakingAdmin),
		dispatch: At(100_000_000),
		output: AppsUiLink,
		output_len_limit: 1_000,
		print_batch: true,
		transact_weight_override: Some(Weight { ref_time: 1_000_000_000, proof_size: 1_000_000 }),
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
		transact_weight_override: Some(Weight { ref_time: 1_000_000_000, proof_size: 1_000_000 }),
	}
}

fn limited_length_user_input() -> ProposalDetails {
	use crate::DispatchTimeWrapper::*;
	use crate::NetworkTrack::*;
	use crate::Output::*;
	ProposalDetails {
		// `system.remark("opengov-submit test")`
		proposal: String::from("0x00004c6f70656e676f762d7375626d69742074657374"),
		track: Polkadot(PolkadotOpenGovOrigin::StakingAdmin),
		dispatch: After(10),
		output: AppsUiLink,
		output_len_limit: 5, // very limiting
		print_batch: true,
		transact_weight_override: Some(Weight { ref_time: 1_000_000_000, proof_size: 1_000_000 }),
	}
}

fn upgrade_args_for_only_relay() -> UpgradeArgs {
	UpgradeArgs {
		network: String::from("polkadot"),
		only: true,
		set_relay_directly: true,
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
		set_relay_directly: true,
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
		set_relay_directly: true,
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
		hex::decode("0x0a000c070ac8".trim_start_matches("0x")).expect("Valid call");
	let public_referendum = hex::decode("0x1500160002439a93279b25a49bf366c9fe1b06d4fc342f46b5a3b2734dcffe0c56c12b28ef03000000010a000000".trim_start_matches("0x")).expect("Valid call");

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
	// Fellowship XCM Send
	// 0x1f0003010003082f0000060302286bee02093d008817008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534
	// 0xadb9e4e4165f92f984690cac8816898978b7dfc8aff6db735ffd5ec9b0430097
	let proposal_details = polkadot_whitelist_remark_user_input();
	let calls = generate_calls(&proposal_details).await;

	let fellowship_preimage = hex::decode("0x2b00dc1f0004010004082f0000060302286bee02093d008817008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534".trim_start_matches("0x")).expect("Valid call");
	let fellowship_referendum = hex::decode("0x3d003e020270ace20636863d9122dea540102dda7df4a52d3a0fe5eaf673e4eca7598aeeca37000000010a000000".trim_start_matches("0x")).expect("Valid call");
	let public_preimage = hex::decode(
		"0x0a0060170300004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	let public_referendum = hex::decode("0x1500160d02e1ee5465c2c6cf6c8249591e1ddccb7b435e7797e4f58108b170d8ad43a313a918000000010a000000".trim_start_matches("0x")).expect("Valid call");

	assert!(calls.preimage_for_whitelist_call.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_whitelist_call {
		match coh {
			CallOrHash::Call(fellowship_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(fellowship_preimage_generated);
				assert_eq!(call_info.encoded, fellowship_preimage);
				assert_eq!(length, 58u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

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
		"0x0a005800004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	let public_referendum = hex::decode("0x15000000028821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca253416000000010a000000".trim_start_matches("0x")).expect("Valid call");

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
		hex::decode("0x20000c060ac8".trim_start_matches("0x")).expect("Valid call");
	let public_referendum = hex::decode("0x15002b00028fd8848a8f93980f5cea2de1c11f29ed7dced592aa207218a2e0ae5b78b9fffb030000000000e1f505".trim_start_matches("0x")).expect("Valid call");

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

	let fellowship_preimage = hex::decode(
		"0x2000882c008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534"
			.trim_start_matches("0x"),
	)
	.expect("Valid call");
	let fellowship_referendum = hex::decode("0x17002b0f02749aff5c635d7ebf11a5199f92cf566d7ae0244fa6c26da5c6e70a215a35c59522000000010a000000".trim_start_matches("0x")).expect("Valid call");
	let public_preimage = hex::decode(
		"0x2000602c0300004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	let public_referendum = hex::decode("0x15002b0d022c1a994725955d3635ce1969e52f25b79b4f8c9685637e63e8eff59ba3f8a9d0180000000000e1f505".trim_start_matches("0x")).expect("Valid call");

	assert!(calls.preimage_for_whitelist_call.is_some(), "it must generate this call");
	if let Some((coh, length)) = calls.preimage_for_whitelist_call {
		match coh {
			CallOrHash::Call(fellowship_preimage_generated) => {
				let call_info = CallInfo::from_runtime_call(fellowship_preimage_generated);
				assert_eq!(call_info.encoded, fellowship_preimage);
				assert_eq!(length, 37u32);
			},
			CallOrHash::Hash(_) => panic!("call length within the limit"),
		}
	}

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
		"0x20005800004c6f70656e676f762d7375626d69742074657374".trim_start_matches("0x"),
	)
	.expect("Valid call");
	let public_referendum = hex::decode("0x15000000028821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca253416000000010a000000".trim_start_matches("0x")).expect("Valid call");

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
	assert_eq!(details.relay_version, Some(String::from("1.2.0")));
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
	assert_eq!(details.relay_version, None);
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
	assert_eq!(details.relay_version, Some(String::from("1.2.0")));
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
