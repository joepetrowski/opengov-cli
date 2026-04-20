//! `batch-ah` subcommand.
//!
//! Reads one or more hex-encoded Asset Hub calls from input files and combines them
//! into a single `Utility.batch_all` AH proposal.

use crate::primitives::{batch_all_on_ah, decode_ah_call};
use clap::Parser as ClapParser;
use std::fs;

/// Combine Asset Hub calls into a single `Utility.batch_all` proposal.
#[derive(Debug, ClapParser)]
pub(crate) struct BatchAhArgs {
	/// Network. Currently only `polkadot` is supported.
	#[clap(long = "network", short, default_value = "polkadot")]
	network: String,

	/// Output file for the batched AH proposal (hex-encoded).
	#[clap(long = "output", short, default_value = "proposal.call")]
	output: String,

	/// Input files, each containing a hex-encoded Asset Hub call.
	/// The calls are batched in the order provided.
	#[clap(required = true)]
	inputs: Vec<String>,
}

pub(crate) async fn batch_ah(args: BatchAhArgs) {
	if args.network.to_ascii_lowercase() != "polkadot" {
		panic!("Only `--network polkadot` is supported for now.");
	}

	let mut calls = Vec::with_capacity(args.inputs.len());
	for path in &args.inputs {
		let hex_str = fs::read_to_string(path)
			.unwrap_or_else(|e| panic!("read {path}: {e}"))
			.trim()
			.to_string();
		let bytes = hex::decode(hex_str.trim_start_matches("0x"))
			.unwrap_or_else(|e| panic!("decode hex in {path}: {e}"));
		let call = decode_ah_call(&bytes)
			.unwrap_or_else(|e| panic!("decode AH call in {path}: {e}"));
		calls.push(call);
	}

	let proposal = batch_all_on_ah(calls);

	let mut hex_out = "0x".to_owned();
	hex_out.push_str(&hex::encode(&proposal.encoded));
	fs::write(&args.output, &hex_out).expect("write proposal output");

	println!("Batched Asset Hub proposal:");
	println!("  Inner calls: {}", args.inputs.len());
	for input in &args.inputs {
		println!("    - {input}");
	}
	println!("  Proposal size: {} bytes", proposal.length);
	println!("  Written to: {}", args.output);
}
