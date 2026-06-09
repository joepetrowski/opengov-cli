mod types;
use crate::types::*;
mod functions;
use crate::functions::*;
mod primitives;
mod build_upgrade;
use crate::build_upgrade::{build_upgrade, UpgradeArgs};
mod register_system_para;
use crate::register_system_para::{register_system_para, RegisterSystemParaArgs};
mod submit_referendum;
use crate::submit_referendum::{submit_referendum, ReferendumArgs};
mod batch_ah;
use crate::batch_ah::{batch_ah, BatchAhArgs};
mod xcm_force_register;
use crate::xcm_force_register::{xcm_force_register, XcmForceRegisterArgs};
mod xcm_force_reserve;
use crate::xcm_force_reserve::{xcm_force_reserve, XcmForceReserveArgs};
use clap::Parser as ClapParser;

#[cfg(test)]
mod tests;

/// Utilities for submitting OpenGov referenda and constructing tedious calls.
#[derive(Debug, ClapParser)]
enum Command {
	BuildUpgrade(UpgradeArgs),
	RegisterSystemPara(RegisterSystemParaArgs),
	SubmitReferendum(ReferendumArgs),
	XcmForceRegister(XcmForceRegisterArgs),
	XcmForceReserve(XcmForceReserveArgs),
	BatchAh(BatchAhArgs),
}

#[tokio::main]
async fn main() {
	let args = Command::parse();
	match args {
		Command::BuildUpgrade(prefs) => build_upgrade(prefs).await,
		Command::RegisterSystemPara(prefs) => register_system_para(prefs).await,
		Command::SubmitReferendum(prefs) => submit_referendum(prefs).await,
		Command::XcmForceRegister(prefs) => xcm_force_register(prefs).await,
		Command::XcmForceReserve(prefs) => xcm_force_reserve(prefs).await,
		Command::BatchAh(prefs) => batch_ah(prefs).await,
	}
}
