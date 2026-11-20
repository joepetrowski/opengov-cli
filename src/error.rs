use crate::Network;
use std::fmt;

/// Everything that can go wrong when building calls from user input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
	/// The string is not the runtime name of a supported network.
	UnknownNetwork(String),
	/// A relay network was expected: `polkadot` or `kusama`.
	NotARelayChain,
	/// The track is not one that referenda can be submitted on.
	UnsupportedTrack,
	/// The version is not of the form `major.minor.patch`.
	NotSemver,
	/// The string is not valid hex.
	InvalidHex(String),
	/// A hash that is not 32 bytes long.
	InvalidHashLength(usize),
	/// Both an `at` and an `after` enactment time were given.
	BothAtAndAfter,
	/// The Fellowship proposal is too long to inline, and the Kusama Relay Chain has no preimage
	/// pallet to fall back to. Carries the proposal's length in bytes.
	InlineLimitExceeded(u32),
	/// The Polkadot Fellowship was requested for something other than a Kusama `WhitelistedCaller`
	/// referendum.
	FellowshipNotSupported,
	/// The runtime blob does not start with the zstd prefix of a compressed Substrate runtime.
	RuntimeNotCompressed,
	/// The runtime blob has an implausible size. Carries the size in bytes.
	RuntimeSize(usize),
	/// The bytes do not decode as a call of this network.
	UndecodableCall(Network),
	/// A call for one network was used where a call for another was needed.
	WrongNetwork { expected: Network, found: Network },
	/// A network that does not belong to the relay chain being upgraded.
	NotInRelayFamily { relay: Network, network: Network },
	/// There is nothing to put in the proposal.
	NoCalls,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::UnknownNetwork(name) => write!(f, "`{name}` is not a supported network"),
			Error::NotARelayChain => write!(f, "`network` must be `polkadot` or `kusama`"),
			Error::UnsupportedTrack => write!(
				f,
				"Unsupported track! Tracks should be in the form `general-admin` or `generaladmin`."
			),
			Error::NotSemver => write!(f, "not semver"),
			Error::InvalidHex(input) => write!(f, "`{input}` is not valid hex"),
			Error::InvalidHashLength(len) => write!(f, "expected a 32 byte hash, got {len} bytes"),
			Error::BothAtAndAfter => write!(
				f,
				"\nBoth `At` and `After` dispatch times provided. You can only use one.\n"
			),
			Error::InlineLimitExceeded(len) => write!(
				f,
				"Fellowship proposal exceeds Inline limit of 128 bytes ({len} bytes). There is no \
				 longer a preimage pallet on Kusama Relay Chain, try again as a root ref."
			),
			Error::FellowshipNotSupported => write!(
				f,
				"`--fellowship polkadot` is only supported with `--network kusama --track \
				 whitelistedcaller`."
			),
			Error::RuntimeNotCompressed =>
				write!(f, "not a zstd-compressed Substrate runtime blob"),
			Error::RuntimeSize(size) =>
				write!(f, "{size} bytes; expected between 100 KiB and 10 MiB"),
			Error::UndecodableCall(network) =>
				write!(f, "the bytes do not decode as a `{}` call", network.runtime_name()),
			Error::WrongNetwork { expected, found } => write!(
				f,
				"expected a `{}` call, got a `{}` call",
				expected.runtime_name(),
				found.runtime_name()
			),
			Error::NotInRelayFamily { relay, network } => write!(
				f,
				"`{}` is not upgraded by `{}` governance",
				network.runtime_name(),
				relay.runtime_name()
			),
			Error::NoCalls => write!(f, "no calls"),
		}
	}
}

impl std::error::Error for Error {}
