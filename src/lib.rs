//! Build the calls needed to upgrade the Polkadot and Kusama system chains and to put proposals
//! to referendum in OpenGov.
//!
//! The library half of this crate is pure: no filesystem, network or stdout, and the same input
//! always gives the same bytes. It builds for `wasm32-unknown-unknown`, and the `wasm` feature
//! adds JavaScript bindings. The `cli` feature (on by default) adds the command line tool, which
//! is a shell around the library.
//!
//! - [`build_upgrade`] builds the proposal that authorizes new runtimes on a relay chain and its
//!   system parachains.
//! - [`referendum_calls`] builds everything that must be submitted to put a proposal to
//!   referendum.

mod error;
mod functions;
pub(crate) mod referendum;
mod types;
pub(crate) mod upgrade;

pub use error::Error;
pub(crate) use functions::*;
pub use referendum::{referendum_calls, ReferendumCalls, ReferendumInput};
pub use types::*;
pub use upgrade::{
	authorize_upgrade_call, build_upgrade, check_and_hash_runtime, runtime_file_name,
	semver_to_intver, UpgradeInput,
};

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(all(test, feature = "cli"))]
mod tests;
