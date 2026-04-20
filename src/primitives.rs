//! Reusable building blocks for constructing Asset Hub governance proposals.
//!
//! Each function produces a `CallInfo` for a specific encoded runtime call on one of
//! the supported chains. Higher-level commands (e.g. `register-system-para`) compose
//! these primitives into complete proposals.
//!
//! The public primitives here drive three CLI subcommands:
//! - `xcm-force-reserve` wraps [`build_force_reserve_call`] in an AH→Coretime XCM send.
//! - `xcm-force-register` builds [`build_force_register_call`] as a preimage and wraps
//!   the whitelist + dispatch pair in AH→Relay XCM sends.
//! - `batch-ah` combines multiple AH-level call files via [`batch_all_on_ah`].

use crate::*;

/// Destination chain for an XCM sent from Asset Hub.
pub(crate) enum XcmDest {
	/// Relay chain (parent, `Location { parents: 1, interior: Here }`).
	Relay,
	/// Sibling parachain by ID (`Location { parents: 1, interior: X1(Parachain(id)) }`).
	Sibling(u32),
}

/// Parameters for `Registrar.force_register` on the relay chain.
pub(crate) struct ForceRegisterParams {
	pub wasm_bytes: Vec<u8>,
	pub genesis_head_bytes: Vec<u8>,
	pub para_id: u32,
	pub manager_bytes: [u8; 32],
	pub deposit: u128,
}

/// Build a relay chain `Registrar.force_register` call.
///
/// Returns the encoded relay `RuntimeCall`, ready to be stored as a preimage or
/// wrapped in the whitelist pattern.
pub(crate) fn build_force_register_call(params: ForceRegisterParams) -> CallInfo {
	use polkadot_relay::runtime_types::{
		polkadot_parachain_primitives::primitives::{HeadData, Id, ValidationCode},
		polkadot_runtime_common::paras_registrar::pallet::Call as RegistrarCall,
	};

	let call = PolkadotRuntimeCall::Registrar(RegistrarCall::force_register {
		who: subxt::utils::AccountId32(params.manager_bytes),
		deposit: params.deposit,
		id: Id(params.para_id),
		genesis_head: HeadData(params.genesis_head_bytes),
		validation_code: ValidationCode(params.wasm_bytes),
	});
	CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(call))
}

/// Build a relay chain `Whitelist.whitelist_call(hash)` call.
pub(crate) fn build_whitelist_call(call_hash: [u8; 32]) -> CallInfo {
	use polkadot_relay::runtime_types::pallet_whitelist::pallet::Call as WhitelistCall;
	let call = PolkadotRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
		call_hash: H256(call_hash),
	});
	CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(call))
}

/// Build a relay chain `Whitelist.dispatch_whitelisted_call(hash, len, weight)` call.
///
/// Returns the `RuntimeCall` itself (not `CallInfo`) so it can optionally be wrapped
/// in `Scheduler.schedule_after` via [`wrap_in_relay_scheduler`] before encoding.
pub(crate) fn build_dispatch_whitelisted_call(
	call_hash: [u8; 32],
	call_encoded_len: u32,
	ref_time: u64,
	proof_size: u64,
) -> PolkadotRuntimeCall {
	use polkadot_relay::runtime_types::{
		pallet_whitelist::pallet::Call as WhitelistCall, sp_weights::weight_v2::Weight,
	};
	PolkadotRuntimeCall::Whitelist(WhitelistCall::dispatch_whitelisted_call {
		call_hash: H256(call_hash),
		call_encoded_len,
		call_weight_witness: Weight { ref_time, proof_size },
	})
}

/// Wrap a relay chain call in `Scheduler.schedule_after(delay, call)` with priority 0.
///
/// Used for the free-preimage path: `whitelist_call` runs immediately (marking the
/// preimage hash as requested), then the dispatch is delayed to give time to note the
/// preimage for free.
pub(crate) fn wrap_in_relay_scheduler(
	call: PolkadotRuntimeCall,
	delay: u32,
) -> PolkadotRuntimeCall {
	use polkadot_relay::runtime_types::pallet_scheduler::pallet::Call as SchedulerCall;
	PolkadotRuntimeCall::Scheduler(SchedulerCall::schedule_after {
		after: delay,
		maybe_periodic: None,
		priority: 0,
		call: Box::new(call),
	})
}

/// Build a Coretime chain `Broker.force_reserve(workload, core)` call reserving
/// a complete core mask for the given task (parachain ID).
pub(crate) fn build_force_reserve_call(para_id: u32, core: u16) -> CallInfo {
	use polkadot_coretime::runtime_types::{
		bounded_collections::bounded_vec::BoundedVec,
		pallet_broker::{
			core_mask::CoreMask, coretime_interface::CoreAssignment, pallet::Call as BrokerCall,
			types::ScheduleItem,
		},
	};

	// CoreMask::complete() = all 80 bits set = 0xffffffffffffffffffff
	let complete_mask = CoreMask([0xff; 10]);
	let call = PolkadotCoretimeRuntimeCall::Broker(BrokerCall::force_reserve {
		workload: BoundedVec(vec![ScheduleItem {
			mask: complete_mask,
			assignment: CoreAssignment::Task(para_id),
		}]),
		core,
	});
	CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCoretime(call))
}

/// Wrap an encoded runtime call in an AH `PolkadotXcm.send(dest, Xcm([UnpaidExecution,
/// Transact(Superuser, inner_call_bytes), ExpectTransactStatus(Success)]))`.
///
/// The `inner_call_bytes` are the SCALE-encoded `RuntimeCall` on the destination chain.
pub(crate) fn wrap_in_xcm_send_from_ah(
	dest: XcmDest,
	inner_call_bytes: Vec<u8>,
) -> PolkadotAssetHubRuntimeCall {
	use polkadot_asset_hub::runtime_types::{
		pallet_xcm::pallet::Call as XcmCall,
		staging_xcm::v5::{
			junction::Junction::Parachain, junctions::Junctions::Here,
			junctions::Junctions::X1, location::Location, Instruction, Xcm,
		},
		xcm::{
			double_encoded::DoubleEncoded, v3::MaybeErrorCode, v3::OriginKind, v3::WeightLimit,
			VersionedLocation, VersionedXcm::V5,
		},
	};

	let dest_location = match dest {
		XcmDest::Relay => Location { parents: 1, interior: Here },
		XcmDest::Sibling(id) => Location { parents: 1, interior: X1([Parachain(id)]) },
	};

	PolkadotAssetHubRuntimeCall::PolkadotXcm(XcmCall::send {
		dest: Box::new(VersionedLocation::V5(dest_location)),
		message: Box::new(V5(Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind: OriginKind::Superuser,
				fallback_max_weight: None,
				call: DoubleEncoded { encoded: inner_call_bytes },
			},
			Instruction::ExpectTransactStatus(MaybeErrorCode::Success),
		]))),
	})
}

/// Wrap a list of Asset Hub calls in `Utility.batch_all`.
///
/// Used to compose multiple primitive XCM sends into a single proposal.
pub(crate) fn batch_all_on_ah(calls: Vec<PolkadotAssetHubRuntimeCall>) -> CallInfo {
	use polkadot_asset_hub::runtime_types::pallet_utility::pallet::Call as UtilityCall;
	let batch = PolkadotAssetHubRuntimeCall::Utility(UtilityCall::batch_all { calls });
	CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(batch))
}

/// Decode an Asset Hub runtime call from its SCALE-encoded bytes.
///
/// Used by `batch-ah` to read input files and reassemble the calls for batching.
pub(crate) fn decode_ah_call(bytes: &[u8]) -> Result<PolkadotAssetHubRuntimeCall, String> {
	use parity_scale_codec::Decode;
	PolkadotAssetHubRuntimeCall::decode(&mut &bytes[..])
		.map_err(|e| format!("Failed to decode Asset Hub call: {e}"))
}
