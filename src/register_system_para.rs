use crate::*;
use clap::Parser as ClapParser;
use std::fs;

/// Generate the calls needed to register a system parachain via Asset Hub governance.
///
/// Produces:
///   - A `force_register_call.hex` file with raw preimage bytes for the relay chain
///   - A proposal file with a batched Asset Hub call: `utility.batch_all([xcm_whitelist, xcm_dispatch])`
///
/// The proposal uses the whitelist+preimage pattern to work around the 128 KB UMP limit.
/// Asset Hub has Root on the relay chain via LocationAsSuperuser, so the XCM Transacts
/// execute whitelist.whitelist_call and whitelist.dispatch_whitelisted_call with Root origin.
#[derive(Debug, ClapParser)]
pub(crate) struct RegisterSystemParaArgs {
	/// Path to the WASM validation code file.
	#[clap(long = "wasm")]
	wasm: String,

	/// Path to the genesis head hex file (from: polkadot-omni-node export-genesis-head).
	#[clap(long = "genesis-head")]
	genesis_head: String,

	/// Parachain ID to register.
	#[clap(long = "para-id")]
	para_id: u32,

	/// Network. Currently only `polkadot` is supported.
	#[clap(long = "network", short)]
	network: String,

	/// Manager account SS58 address.
	#[clap(long = "manager")]
	manager: String,

	/// Deposit in plancks.
	#[clap(long = "deposit", default_value = "0")]
	deposit: u128,

	/// Weight ref_time witness for dispatch_whitelisted_call.
	/// Default based on Polkadot benchmarks for force_register (~7.7B) with headroom.
	#[clap(long = "ref-time", default_value = "10000000000")]
	ref_time: u64,

	/// Weight proof_size witness for dispatch_whitelisted_call.
	/// Default based on Polkadot benchmarks for force_register (~3697) with headroom.
	#[clap(long = "proof-size", default_value = "5000")]
	proof_size: u64,

	/// Reserve a dedicated core for this parachain on the Coretime chain via broker.force_reserve.
	/// Provide the core index. Sent as XCM from Asset Hub to the Coretime chain.
	#[clap(long = "assign-core")]
	assign_core: Option<u16>,

	/// Delay dispatch_whitelisted_call using Scheduler.schedule_after so that the preimage
	/// can be noted for free after whitelist_call requests it. Value is the delay in relay
	/// chain blocks (e.g. 100 = ~10 minutes on Polkadot).
	#[clap(long = "delay-whitelist-dispatch-relay")]
	delay_whitelist_dispatch_relay: Option<u32>,

	/// Output file name for the Asset Hub proposal.
	#[clap(long = "filename")]
	filename: Option<String>,
}

/// Parameters for building registration calls (decoupled from CLI/file I/O).
pub(crate) struct RegisterSystemParaParams {
	pub wasm_bytes: Vec<u8>,
	pub genesis_head_bytes: Vec<u8>,
	pub para_id: u32,
	pub manager_bytes: [u8; 32],
	pub deposit: u128,
	pub ref_time: u64,
	pub proof_size: u64,
	/// If set, reserve this core index for the para via broker.force_reserve on the Coretime chain.
	pub assign_core: Option<u16>,
	/// If set, wrap dispatch_whitelisted_call in Scheduler.schedule_after with this delay (blocks).
	/// This allows the preimage to be noted for free after whitelist_call requests it.
	pub delay_whitelist_dispatch_relay: Option<u32>,
}

/// Output of building registration calls.
pub(crate) struct RegisterSystemParaOutput {
	/// The encoded force_register call (relay chain). Used as preimage content.
	pub force_register_info: CallInfo,
	/// The batched Asset Hub proposal (utility.batch_all with two XCM sends).
	pub proposal: CallInfo,
}

/// Build the registration calls from parameters (pure logic, no file I/O).
pub(crate) fn build_polkadot_register_calls(params: RegisterSystemParaParams) -> RegisterSystemParaOutput {
	use polkadot_asset_hub::runtime_types::{
		pallet_utility::pallet::Call as UtilityCall,
		pallet_xcm::pallet::Call as XcmCall,
		staging_xcm::v5::{junctions::Junctions::Here, location::Location, Instruction, Xcm},
		xcm::{
			double_encoded::DoubleEncoded, v3::MaybeErrorCode, v3::OriginKind, v3::WeightLimit,
			VersionedLocation, VersionedXcm::V5,
		},
	};
	use polkadot_relay::runtime_types::{
		polkadot_parachain_primitives::primitives::{HeadData, Id, ValidationCode},
		polkadot_runtime_common::paras_registrar::pallet::Call as RegistrarCall,
		pallet_whitelist::pallet::Call as WhitelistCall,
		sp_weights::weight_v2::Weight,
	};

	// 1. Encode force_register call (relay chain)
	let force_register_call = PolkadotRuntimeCall::Registrar(RegistrarCall::force_register {
		who: subxt::utils::AccountId32(params.manager_bytes),
		deposit: params.deposit,
		id: Id(params.para_id),
		genesis_head: HeadData(params.genesis_head_bytes),
		validation_code: ValidationCode(params.wasm_bytes),
	});

	let force_register_info =
		CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(force_register_call));

	// 2. Encode whitelist calls (relay chain, for XCM Transact)
	let whitelist_call = PolkadotRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
		call_hash: H256(force_register_info.hash),
	});
	let whitelist_info = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(whitelist_call));

	let dispatch_whitelisted_call =
		PolkadotRuntimeCall::Whitelist(WhitelistCall::dispatch_whitelisted_call {
			call_hash: H256(force_register_info.hash),
			call_encoded_len: force_register_info.length,
			call_weight_witness: Weight {
				ref_time: params.ref_time,
				proof_size: params.proof_size,
			},
		});

	// Optionally wrap dispatch in Scheduler.schedule_after for the free preimage path:
	// whitelist_call runs immediately (requesting the preimage hash), then dispatch
	// is delayed by N blocks, giving time to note the preimage for free.
	let dispatch_relay_call = if let Some(delay) = params.delay_whitelist_dispatch_relay {
		use polkadot_relay::runtime_types::pallet_scheduler::pallet::Call as SchedulerCall;
		PolkadotRuntimeCall::Scheduler(SchedulerCall::schedule_after {
			after: delay,
			maybe_periodic: None,
			priority: 0,
			call: Box::new(dispatch_whitelisted_call),
		})
	} else {
		dispatch_whitelisted_call
	};
	let dispatch_info =
		CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(dispatch_relay_call));

	// 3. Wrap in XCM send calls (Asset Hub → Relay / Coretime)
	use polkadot_asset_hub::runtime_types::staging_xcm::v5::junction::Junction::Parachain;
	use polkadot_asset_hub::runtime_types::staging_xcm::v5::junctions::Junctions::X1;

	let relay_dest = Box::new(VersionedLocation::V5(Location {
		parents: 1,
		interior: Here,
	}));

	let xcm_whitelist = PolkadotAssetHubRuntimeCall::PolkadotXcm(XcmCall::send {
		dest: relay_dest.clone(),
		message: Box::new(V5(Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind: OriginKind::Superuser,
				fallback_max_weight: None,
				call: DoubleEncoded { encoded: whitelist_info.encoded },
			},
			Instruction::ExpectTransactStatus(MaybeErrorCode::Success),
		]))),
	});

	let xcm_dispatch = PolkadotAssetHubRuntimeCall::PolkadotXcm(XcmCall::send {
		dest: relay_dest,
		message: Box::new(V5(Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind: OriginKind::Superuser,
				fallback_max_weight: None,
				call: DoubleEncoded { encoded: dispatch_info.encoded },
			},
			Instruction::ExpectTransactStatus(MaybeErrorCode::Success),
		]))),
	});

	// 4. Optionally add XCM to Coretime chain for force_reserve
	let mut batch_calls = vec![xcm_whitelist, xcm_dispatch];

	if let Some(core_index) = params.assign_core {
		use polkadot_coretime::runtime_types::{
			bounded_collections::bounded_vec::BoundedVec,
			pallet_broker::pallet::Call as BrokerCall,
			pallet_broker::types::ScheduleItem,
			pallet_broker::core_mask::CoreMask,
			pallet_broker::coretime_interface::CoreAssignment as CoretimeCoreAssignment,
		};

		// Encode broker.force_reserve using Coretime chain types
		// CoreMask::complete() = all 80 bits set = 0xffffffffffffffffffff
		let complete_mask = CoreMask([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
		let force_reserve_call =
			PolkadotCoretimeRuntimeCall::Broker(BrokerCall::force_reserve {
				workload: BoundedVec(vec![ScheduleItem {
					mask: complete_mask,
					assignment: CoretimeCoreAssignment::Task(params.para_id),
				}]),
				core: core_index,
			});
		let force_reserve_info =
			CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCoretime(force_reserve_call));

		// XCM from Asset Hub → Coretime chain (sibling, Parachain 1005)
		let coretime_dest = Box::new(VersionedLocation::V5(Location {
			parents: 1,
			interior: X1([Parachain(1005)]),
		}));

		let xcm_force_reserve = PolkadotAssetHubRuntimeCall::PolkadotXcm(XcmCall::send {
			dest: coretime_dest,
			message: Box::new(V5(Xcm(vec![
				Instruction::UnpaidExecution {
					weight_limit: WeightLimit::Unlimited,
					check_origin: None,
				},
				Instruction::Transact {
					origin_kind: OriginKind::Superuser,
					fallback_max_weight: None,
					call: DoubleEncoded { encoded: force_reserve_info.encoded },
				},
				Instruction::ExpectTransactStatus(MaybeErrorCode::Success),
			]))),
		});

		batch_calls.push(xcm_force_reserve);
	}

	// 5. Batch into single proposal
	let batch_call = PolkadotAssetHubRuntimeCall::Utility(UtilityCall::batch_all {
		calls: batch_calls,
	});

	let proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(batch_call));

	RegisterSystemParaOutput { force_register_info, proposal }
}

pub(crate) async fn register_system_para(args: RegisterSystemParaArgs) {
	match args.network.to_ascii_lowercase().as_str() {
		"polkadot" => register_polkadot_system_para(args).await,
		"kusama" => {
			panic!("Kusama support not yet implemented. Use --network polkadot.");
		},
		_ => panic!("`network` must be `polkadot` or `kusama`"),
	}
}

async fn register_polkadot_system_para(args: RegisterSystemParaArgs) {
	// Read inputs
	let wasm_bytes = fs::read(&args.wasm).expect("Should read WASM file");
	println!("WASM size: {} bytes", wasm_bytes.len());

	let genesis_head_hex = fs::read_to_string(&args.genesis_head)
		.expect("Should read genesis head file")
		.trim()
		.to_string();
	let genesis_head_bytes =
		hex::decode(genesis_head_hex.trim_start_matches("0x")).expect("Valid hex for genesis head");
	println!("Genesis head size: {} bytes", genesis_head_bytes.len());

	use sp_core::crypto::Ss58Codec;
	let manager_account =
		sp_core::crypto::AccountId32::from_ss58check(&args.manager).expect("Valid SS58 address");
	let manager_bytes: [u8; 32] = manager_account.into();

	println!("ParaId: {}", args.para_id);
	println!("Manager: {}", args.manager);
	println!("Deposit: {}", args.deposit);

	if let Some(core) = args.assign_core {
		println!("Assign core: {} (via broker.force_reserve on Coretime chain)", core);
	}
	if let Some(delay) = args.delay_whitelist_dispatch_relay {
		println!("Free preimage: dispatch delayed by {} blocks (~{} minutes)", delay, delay * 6 / 60);
	}

	// Build calls
	let output = build_polkadot_register_calls(RegisterSystemParaParams {
		wasm_bytes,
		genesis_head_bytes,
		para_id: args.para_id,
		manager_bytes,
		deposit: args.deposit,
		ref_time: args.ref_time,
		proof_size: args.proof_size,
		assign_core: args.assign_core,
		delay_whitelist_dispatch_relay: args.delay_whitelist_dispatch_relay,
	});

	println!("\nforce_register call:");
	println!("  Encoded size: {} bytes", output.force_register_info.length);
	println!("  Hash: 0x{}", hex::encode(output.force_register_info.hash));

	// Write raw preimage bytes
	let mut preimage_hex = "0x".to_owned();
	preimage_hex.push_str(&hex::encode(&output.force_register_info.encoded));
	fs::write("force_register_call.hex", &preimage_hex).expect("write force_register_call.hex");
	println!("  Written to: force_register_call.hex");

	println!("\nWhitelist+dispatch wrapped in XCM batch:");
	println!("  Proposal size: {} bytes", output.proposal.length);

	let fname = args.filename.unwrap_or_else(|| {
		format!("register-system-para-{}.call", args.para_id)
	});

	let mut proposal_hex = "0x".to_owned();
	proposal_hex.push_str(&hex::encode(&output.proposal.encoded));
	fs::write(&fname, &proposal_hex).expect("write proposal file");

	// Output summary
	println!("\n{}", "=".repeat(60));
	println!("  REGISTER SYSTEM PARACHAIN {}", args.para_id);
	println!("{}", "=".repeat(60));
	println!();
	println!("  STEP 1 — Submit on Relay Chain (permissionless):");
	println!(
		"    Use force_register_call.hex ({} bytes) as the bytes for",
		output.force_register_info.length
	);
	println!("    Preimage.note_preimage in any wallet (e.g. Polkadot JS Apps).");
	println!();
	println!("  STEP 2 — Submit as referendum:");
	println!("    opengov-cli submit-referendum \\");
	println!("      --proposal \"{}\" \\", fname);
	println!("      --network \"polkadot\" --track whitelistedcaller");
	println!("{}", "=".repeat(60));
}
