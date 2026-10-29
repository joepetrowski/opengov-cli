mod types;
use crate::types::*;
mod functions;
use crate::functions::*;
mod add_invulnerables;
use crate::add_invulnerables::{add_invulnerables, AddInvulnerablesArgs};
mod build_upgrade;
use crate::build_upgrade::{build_upgrade, UpgradeArgs};
mod submit_referendum;
use crate::submit_referendum::{submit_referendum, ReferendumArgs};
use clap::Parser as ClapParser;

#[cfg(test)]
mod tests;

/// Utilities for submitting OpenGov referenda and constructing tedious calls.
#[derive(Debug, ClapParser)]
enum Command {
	BuildUpgrade(UpgradeArgs),
	SubmitReferendum(ReferendumArgs),
	AddInvulnerables(AddInvulnerablesArgs),
}

#[tokio::main]
async fn main() {
	let args = Command::parse();
	match args {
		Command::BuildUpgrade(prefs) => build_upgrade(prefs).await,
		Command::SubmitReferendum(prefs) => submit_referendum(prefs).await,
		Command::AddInvulnerables(prefs) => add_invulnerables(prefs).await,
	}
}
