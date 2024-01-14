mod types;
use crate::types::*;
mod build_upgrade;
use crate::build_upgrade::{build_upgrade, UpgradeArgs};
mod submit_referendum;
use crate::submit_referendum::{submit_referendum, ReferendumArgs};
mod test_referendum;
use crate::test_referendum::{test_referendum, TestReferendumArgs};
use clap::Parser as ClapParser;

#[cfg(test)]
mod tests;

/// Utilities for submitting OpenGov referenda and constructing tedious calls.
#[derive(Debug, ClapParser)]
enum Command {
	BuildUpgrade(UpgradeArgs),
	SubmitReferendum(ReferendumArgs),
	TestReferendum(TestReferendumArgs),
}

#[tokio::main]
async fn main() {
	let args = Command::parse();
	match args {
		Command::BuildUpgrade(prefs) => build_upgrade(prefs).await,
		Command::SubmitReferendum(prefs) => submit_referendum(prefs).await,
		Command::TestReferendum(prefs) => test_referendum(prefs).await,
	}
}
