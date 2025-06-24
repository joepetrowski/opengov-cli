mod types;
use crate::types::*;
mod functions;
use crate::functions::*;
mod build_upgrade;
use crate::build_upgrade::{UpgradeArgs, build_upgrade};
mod submit_referendum;
use crate::submit_referendum::{ReferendumArgs, submit_referendum};
use clap::Parser as ClapParser;

#[cfg(test)]
mod tests;

/// Utilities for submitting OpenGov referenda and constructing tedious calls.
#[derive(Debug, ClapParser)]
enum Command {
	BuildUpgrade(UpgradeArgs),
	SubmitReferendum(ReferendumArgs),
}

#[tokio::main]
async fn main() {
	let args = Command::parse();
	match args {
		Command::BuildUpgrade(prefs) => build_upgrade(prefs).await,
		Command::SubmitReferendum(prefs) => submit_referendum(prefs).await,
	}
}
