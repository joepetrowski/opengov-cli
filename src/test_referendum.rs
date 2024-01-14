use crate::*;
use clap::Parser as ClapParser;
// use std::fs;

/// Generate all the calls needed to submit a proposal as a referendum in OpenGov.
#[derive(Debug, ClapParser)]
pub(crate) struct TestReferendumArgs {
	/// The encoded proposal that we want to submit. This can either be the call data itself,
	/// e.g. "0x0102...", or a file path that contains the data, e.g. "./my_proposal.call".
	#[clap(long = "proposal", short)]
	proposal: String,

	/// Network on which to submit the referendum. `polkadot` or `kusama`.
	#[clap(long = "network", short)]
	network: String,
}

// The sub-command's "main" function.
pub(crate) async fn test_referendum(prefs: TestReferendumArgs) {
	// Find out what the user wants to do.
	let (_proposal, _network) = parse_inputs(prefs);
}

// Parse the CLI inputs and return a typed struct with all the details needed.
fn parse_inputs(prefs: TestReferendumArgs) -> (String, String) {

	let proposal = prefs.proposal;
	let network = prefs.network;

	return (proposal, network)
}
