use crate::*;
use std::fs;

// Check what the user entered for the proposal. If it is just call data, return it back. Otherwise,
// we expect a path to a file that contains the call data. Read that in and return it.
pub(crate) fn get_proposal_bytes(proposal: String) -> Vec<u8> {
	let proposal = proposal.as_str();
	if proposal.starts_with("0x") {
		// This is just call data
		hex::decode(proposal.trim_start_matches("0x")).expect("Valid proposal")
	} else {
		// This is a file path
		let contents = fs::read_to_string(proposal).expect("Should give a valid file path");
		hex::decode(contents.as_str().trim().trim_start_matches("0x")).expect("Valid proposal")
	}
}

// Write some call data to a file as a "0x"-prefixed hex string.
pub(crate) fn write_call_data(fname: &str, encoded: &[u8]) {
	let mut info_to_write = "0x".to_owned();
	info_to_write.push_str(hex::encode(encoded).as_str());
	fs::write(fname, info_to_write).expect("it should write");
}

// Take a call, which includes its intended destination, and wrap it in XCM instructions to `send`
// it from Polkadot Asset Hub and have it execute on its destination with the given origin kind:
// `Superuser` to dispatch as Root, or `Xcm` to dispatch as the origin of the sending referendum
// track (e.g. the StakingAdmin plurality voice).
pub(crate) fn send_from_polkadot_asset_hub(
	auth: &CallInfo,
	origin_kind: polkadot_asset_hub::runtime_types::xcm::v3::OriginKind,
) -> PolkadotAssetHubRuntimeCall {
	use polkadot_asset_hub::runtime_types::{
		pallet_xcm::pallet::Call as XcmCall,
		staging_xcm::v5::{
			junction::Junction::Parachain, junctions::Junctions::Here, junctions::Junctions::X1,
			location::Location, Instruction, Xcm,
		},
		xcm::{
			double_encoded::DoubleEncoded, v3::WeightLimit, VersionedLocation, VersionedXcm::V5,
		},
	};

	let location = match auth.network.get_para_id() {
		Ok(para_id) => Location { parents: 1, interior: X1([Parachain(para_id)]) },
		Err(_) => Location { parents: 1, interior: Here },
	};

	PolkadotAssetHubRuntimeCall::PolkadotXcm(XcmCall::send {
		dest: Box::new(VersionedLocation::V5(location)),
		message: Box::new(V5(Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind,
				fallback_max_weight: None,
				call: DoubleEncoded { encoded: auth.encoded.clone() },
			},
		]))),
	})
}

// As `send_from_polkadot_asset_hub`, but sending from Kusama Asset Hub.
pub(crate) fn send_from_kusama_asset_hub(
	auth: &CallInfo,
	origin_kind: kusama_asset_hub::runtime_types::xcm::v3::OriginKind,
) -> KusamaAssetHubRuntimeCall {
	use kusama_asset_hub::runtime_types::{
		pallet_xcm::pallet::Call as XcmCall,
		staging_xcm::v5::{
			junction::Junction::Parachain, junctions::Junctions::Here, junctions::Junctions::X1,
			location::Location, Instruction, Xcm,
		},
		xcm::{
			double_encoded::DoubleEncoded, v3::WeightLimit, VersionedLocation, VersionedXcm::V5,
		},
	};

	let location = match auth.network.get_para_id() {
		Ok(para_id) => Location { parents: 1, interior: X1([Parachain(para_id)]) },
		Err(_) => Location { parents: 1, interior: Here },
	};

	KusamaAssetHubRuntimeCall::PolkadotXcm(XcmCall::send {
		dest: Box::new(VersionedLocation::V5(location)),
		message: Box::new(V5(Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind,
				fallback_max_weight: None,
				call: DoubleEncoded { encoded: auth.encoded.clone() },
			},
		]))),
	})
}
