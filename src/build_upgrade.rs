use crate::*;
use clap::Parser as ClapParser;
use std::fs;

#[derive(Debug, ClapParser)]
pub(crate) struct UpgradeArgs {
	/// Network on which to submit the referendum. `polkadot` or `kusama`.
	#[clap(long = "network", short)]
	network: String,

	/// The runtime version of the Relay Chain to which to upgrade. E.g. "9430" or "latest".
	#[clap(long = "relay-version")]
	relay_version: String,

	/// The runtime version of the system parachains to which to upgrade. E.g. "9430" or "latest".
	#[clap(long = "parachain-version")]
	parachain_version: String,

	/// Name of the file to which to write the output. If not provided, a default will be
	/// constructed.
	#[clap(long = "filename")]
	filename: Option<String>,
}

pub(crate) fn build_upgrade(_prefs: UpgradeArgs) {}
