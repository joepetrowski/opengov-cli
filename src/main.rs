mod types;
use crate::types::*;
mod submit_referendum;
use crate::submit_referendum::{submit_referendum, ReferendumArgs};
use clap::Parser as ClapParser;

#[cfg(test)]
mod tests;

#[derive(Debug, ClapParser)]
enum Command {
	SubmitReferendum(ReferendumArgs),
}

#[tokio::main]
async fn main() {
	let args = Command::parse();
	match args {
		Command::SubmitReferendum(prefs) => submit_referendum(prefs).await,
	}
}
