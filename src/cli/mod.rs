//! The command line tool: argument parsing, downloads, files and printing around the library.

pub(crate) mod add_invulnerables;
pub(crate) mod build_upgrade;
pub(crate) mod submit_referendum;
mod types;
pub(crate) use types::*;

use add_invulnerables::{add_invulnerables, AddInvulnerablesArgs};
use build_upgrade::{build_upgrade, UpgradeArgs};
use clap::Parser as ClapParser;
use std::fs;
use submit_referendum::{submit_referendum, ReferendumArgs};

/// Utilities for submitting OpenGov referenda and constructing tedious calls.
#[derive(Debug, ClapParser)]
enum Command {
	BuildUpgrade(UpgradeArgs),
	SubmitReferendum(ReferendumArgs),
	AddInvulnerables(AddInvulnerablesArgs),
}

/// Parse the command line arguments and run the requested sub-command.
pub async fn run() {
	let args = Command::parse();
	match args {
		Command::BuildUpgrade(prefs) => build_upgrade(prefs).await,
		Command::SubmitReferendum(prefs) => submit_referendum(prefs).await,
		Command::AddInvulnerables(prefs) => add_invulnerables(prefs).await,
	}
}

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
