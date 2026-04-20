//! `xcm-force-reserve` subcommand.
//!
//! Builds an Asset Hub `PolkadotXcm.send` call to the Coretime chain carrying
//! a `Transact(Superuser, Broker.force_reserve(schedule, core))`. The output
//! is a single hex-encoded AH call that can be batched with other AH calls via
//! `batch-ah`.

use crate::primitives::{build_force_reserve_call, wrap_in_xcm_send_from_ah, XcmDest};
use crate::*;
use clap::Parser as ClapParser;
use std::fs;

/// Generate an AH XCM send wrapping `Broker.force_reserve` for the Coretime chain.
#[derive(Debug, ClapParser)]
pub(crate) struct XcmForceReserveArgs {
	/// Parachain ID to assign to the reserved core.
	#[clap(long = "para-id")]
	para_id: u32,

	/// Core index to reserve.
	#[clap(long = "core")]
	core: u16,

	/// Network. Currently only `polkadot` is supported.
	#[clap(long = "network", short, default_value = "polkadot")]
	network: String,

	/// Output file for the hex-encoded Asset Hub call.
	#[clap(long = "output", short, default_value = "xcm_force_reserve.hex")]
	output: String,
}

pub(crate) async fn xcm_force_reserve(args: XcmForceReserveArgs) {
	if args.network.to_ascii_lowercase() != "polkadot" {
		panic!("Only `--network polkadot` is supported for now.");
	}

	let fr_info = build_force_reserve_call(args.para_id, args.core);
	let xcm = wrap_in_xcm_send_from_ah(XcmDest::Sibling(1005), fr_info.encoded.clone());
	let xcm_info = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(xcm));

	let mut hex_out = "0x".to_owned();
	hex_out.push_str(&hex::encode(&xcm_info.encoded));
	fs::write(&args.output, &hex_out).expect("write output");

	println!("XCM force_reserve (AH → Coretime):");
	println!("  Para ID: {}", args.para_id);
	println!("  Core: {}", args.core);
	println!("  Inner call (Broker.force_reserve) size: {} bytes", fr_info.length);
	println!("  XCM send size: {} bytes", xcm_info.length);
	println!("  Written to: {}", args.output);
}
