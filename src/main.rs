mod types;
use crate::types::*;
mod functions;
use crate::functions::*;
mod build_upgrade;
use crate::build_upgrade::{build_upgrade, UpgradeArgs};
mod register_system_para;
use crate::register_system_para::{register_system_para, RegisterSystemParaArgs};
mod submit_referendum;
use crate::submit_referendum::{submit_referendum, ReferendumArgs};
use clap::Parser as ClapParser;

#[cfg(test)]
mod tests;

/// Utilities for submitting OpenGov referenda and constructing tedious calls.
#[derive(Debug, ClapParser)]
enum Command {
	BuildUpgrade(UpgradeArgs),
	RegisterSystemPara(RegisterSystemParaArgs),
	SubmitReferendum(ReferendumArgs),
}

#[tokio::main]
async fn main() {
	let args = Command::parse();
	match args {
		Command::BuildUpgrade(prefs) => build_upgrade(prefs).await,
		Command::RegisterSystemPara(prefs) => register_system_para(prefs).await,
		Command::SubmitReferendum(prefs) => submit_referendum(prefs).await,
	}
}
