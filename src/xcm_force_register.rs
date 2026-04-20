//! `xcm-force-register` subcommand.
//!
//! Builds the whitelist + preimage pattern for a relay chain `Registrar.force_register`
//! call. The ~1 MB WASM payload exceeds the 128 KB UMP limit, so the call is stored as
//! a preimage on the relay while AH governance whitelists + dispatches it via XCM.
//!
//! Outputs (in `--output` directory):
//! - `preimage.hex`        — raw relay `force_register` bytes (submit via `Preimage.note_preimage`)
//! - `xcm_whitelist.hex`   — AH XCM: `whitelist.whitelist_call(hash)`
//! - `xcm_dispatch.hex`    — AH XCM: `whitelist.dispatch_whitelisted_call(hash, len, weight)`,
//!                           optionally wrapped in `Scheduler.schedule_after(delay, ...)`

use crate::primitives::{
	build_dispatch_whitelisted_call, build_force_register_call, build_whitelist_call,
	wrap_in_relay_scheduler, wrap_in_xcm_send_from_ah, ForceRegisterParams, XcmDest,
};
use crate::*;
use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;

/// Generate the whitelist+preimage pattern for a relay-chain `force_register` call.
#[derive(Debug, ClapParser)]
pub(crate) struct XcmForceRegisterArgs {
	/// Path to the WASM validation code file.
	#[clap(long = "wasm")]
	wasm: String,

	/// Path to the genesis head hex file (from: polkadot-omni-node export-genesis-head).
	#[clap(long = "genesis-head")]
	genesis_head: String,

	/// Parachain ID to register.
	#[clap(long = "para-id")]
	para_id: u32,

	/// Manager account SS58 address.
	#[clap(long = "manager")]
	manager: String,

	/// Network. Currently only `polkadot` is supported.
	#[clap(long = "network", short, default_value = "polkadot")]
	network: String,

	/// Deposit in plancks.
	#[clap(long = "deposit", default_value = "0")]
	deposit: u128,

	/// Weight ref_time witness for dispatch_whitelisted_call.
	#[clap(long = "ref-time", default_value = "10000000000")]
	ref_time: u64,

	/// Weight proof_size witness for dispatch_whitelisted_call.
	#[clap(long = "proof-size", default_value = "5000")]
	proof_size: u64,

	/// Delay dispatch_whitelisted_call via `Scheduler.schedule_after(delay)`.
	/// Enables the free-preimage path: whitelist_call requests the hash, then after `delay`
	/// relay blocks the dispatch fires. During the window anyone can note_preimage for free.
	#[clap(long = "delay-whitelist-dispatch-relay")]
	delay_whitelist_dispatch_relay: Option<u32>,

	/// Output directory for the three hex files.
	#[clap(long = "output", short, default_value = ".")]
	output: String,
}

pub(crate) async fn xcm_force_register(args: XcmForceRegisterArgs) {
	if args.network.to_ascii_lowercase() != "polkadot" {
		panic!("Only `--network polkadot` is supported for now.");
	}

	// Read inputs
	let wasm_bytes = fs::read(&args.wasm).expect("Should read WASM file");
	let genesis_head_hex = fs::read_to_string(&args.genesis_head)
		.expect("Should read genesis head file")
		.trim()
		.to_string();
	let genesis_head_bytes =
		hex::decode(genesis_head_hex.trim_start_matches("0x")).expect("Valid hex for genesis head");

	use sp_core::crypto::Ss58Codec;
	let manager_account =
		sp_core::crypto::AccountId32::from_ss58check(&args.manager).expect("Valid SS58 address");
	let manager_bytes: [u8; 32] = manager_account.into();

	// Build force_register (the preimage)
	let force_register_info = build_force_register_call(ForceRegisterParams {
		wasm_bytes,
		genesis_head_bytes,
		para_id: args.para_id,
		manager_bytes,
		deposit: args.deposit,
	});

	// Build whitelist_call(hash) + XCM wrap
	let whitelist_info = build_whitelist_call(force_register_info.hash);
	let xcm_whitelist = wrap_in_xcm_send_from_ah(XcmDest::Relay, whitelist_info.encoded);
	let xcm_whitelist_info =
		CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(xcm_whitelist));

	// Build dispatch_whitelisted_call, optionally wrap in Scheduler, then XCM wrap
	let dispatch_call = build_dispatch_whitelisted_call(
		force_register_info.hash,
		force_register_info.length,
		args.ref_time,
		args.proof_size,
	);
	let dispatch_call = match args.delay_whitelist_dispatch_relay {
		Some(delay) => wrap_in_relay_scheduler(dispatch_call, delay),
		None => dispatch_call,
	};
	let dispatch_info = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(dispatch_call));
	let xcm_dispatch = wrap_in_xcm_send_from_ah(XcmDest::Relay, dispatch_info.encoded);
	let xcm_dispatch_info =
		CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(xcm_dispatch));

	// Ensure output directory exists
	let out_dir = PathBuf::from(&args.output);
	fs::create_dir_all(&out_dir).expect("create output directory");

	let preimage_path = out_dir.join("preimage.hex");
	let whitelist_path = out_dir.join("xcm_whitelist.hex");
	let dispatch_path = out_dir.join("xcm_dispatch.hex");

	write_hex(&preimage_path, &force_register_info.encoded);
	write_hex(&whitelist_path, &xcm_whitelist_info.encoded);
	write_hex(&dispatch_path, &xcm_dispatch_info.encoded);

	println!("XCM force_register (whitelist+preimage):");
	println!("  Para ID: {}", args.para_id);
	println!("  Manager: {}", args.manager);
	println!(
		"  Preimage (relay force_register): {} bytes, hash 0x{}",
		force_register_info.length,
		hex::encode(force_register_info.hash)
	);
	println!("    → {}", preimage_path.display());
	println!(
		"  XCM whitelist_call: {} bytes → {}",
		xcm_whitelist_info.length,
		whitelist_path.display()
	);
	if let Some(delay) = args.delay_whitelist_dispatch_relay {
		println!(
			"  XCM dispatch (scheduled +{} blocks): {} bytes → {}",
			delay,
			xcm_dispatch_info.length,
			dispatch_path.display()
		);
	} else {
		println!(
			"  XCM dispatch_whitelisted_call: {} bytes → {}",
			xcm_dispatch_info.length,
			dispatch_path.display()
		);
	}
}

fn write_hex(path: &std::path::Path, bytes: &[u8]) {
	let mut hex_out = "0x".to_owned();
	hex_out.push_str(&hex::encode(bytes));
	fs::write(path, &hex_out).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}
