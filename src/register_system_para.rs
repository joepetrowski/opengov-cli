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
	#[clap(long = "ref-time", default_value = "60000000000")]
	ref_time: u64,

	/// Weight proof_size witness for dispatch_whitelisted_call.
	#[clap(long = "proof-size", default_value = "10000")]
	proof_size: u64,

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
	let dispatch_info =
		CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(dispatch_whitelisted_call));

	// 3. Wrap in XCM send calls (Asset Hub → Relay)
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

	// 4. Batch into single proposal
	let batch_call = PolkadotAssetHubRuntimeCall::Utility(UtilityCall::batch_all {
		calls: vec![xcm_whitelist, xcm_dispatch],
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

	// Build calls
	let output = build_polkadot_register_calls(RegisterSystemParaParams {
		wasm_bytes,
		genesis_head_bytes,
		para_id: args.para_id,
		manager_bytes,
		deposit: args.deposit,
		ref_time: args.ref_time,
		proof_size: args.proof_size,
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
