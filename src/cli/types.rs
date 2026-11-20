use crate::*;

// Info and preferences provided by the user for proposal submission.
pub(crate) struct ProposalDetails {
	// The proposal, generated elsewhere and pasted here.
	pub(crate) proposal: String,
	// The track to submit on.
	pub(crate) track: NetworkTrack,
	// When do you want this to enact. `At(block)` or `After(blocks)`.
	pub(crate) dispatch: DispatchTimeWrapper,
	// How you would like to view the output.
	pub(crate) output: Output,
	// Cutoff length in bytes for printing the output. If too long, it will print the hash of the
	// call you would need to submit so that you can verify before submission.
	pub(crate) output_len_limit: u32,
	// Whether or not to group all calls into a batch. Uses `force_batch` in case the account does
	// not have funds for pre-image deposits or is not a fellow.
	pub(crate) print_batch: bool,
	// Whether to use light client endpoints in PAPI links (default true).
	pub(crate) use_light_client: bool,
	// Whether to use the Polkadot Fellowship (on Collectives) instead of the Kusama Fellowship.
	// Only applicable for Kusama WhitelistedCaller track.
	pub(crate) fellowship_on_polkadot: bool,
}

// Info and preferences provided by the user for runtime upgrade construction.
pub(crate) struct UpgradeDetails {
	// The Relay Network for this upgrade, Polkadot or Kusama.
	pub(crate) relay: Network,
	// All networks to upgrade.
	pub(crate) networks: Vec<VersionedNetwork>,
	// The directory into which to write information needed.
	pub(crate) directory: String,
	// The filename of the output.
	pub(crate) output_file: String,
	// An additional call to be enacted in the same batch as the system upgrade.
	pub(crate) additional: Option<CallInfo>,
	// Skip sanity checks on the downloaded runtime blobs.
	pub(crate) no_runtime_checks: bool,
}

// A network and the version to which it will upgrade.
#[derive(Debug, PartialEq)]
pub(crate) struct VersionedNetwork {
	// A network identifier.
	pub(crate) network: Network,
	// A runtime version number (i.e. "9430", not "0.9.43").
	pub(crate) version: String,
}

// How the user would like to see the output of the program.
pub(crate) enum Output {
	// Print just the call data (e.g. 0x1234).
	CallData,
	// Print a clickable link to view the decoded call on Polkadot JS Apps UI.
	AppsUiLink,
}

// A call or a hash. Used for printing (or rather, to avoid printing large calls).
// The Hash variant is only used when calls exceed the output length limit, which is rare.
#[allow(clippy::large_enum_variant)]
pub(crate) enum CallOrHash {
	Call(NetworkRuntimeCall),
	Hash([u8; 32]),
}

impl CallInfo {
	// Take `Self` and a length limit as input. If the call length exceeds the limit, just return
	// its hash. Call length is recomputed and will be 2 bytes longer than the actual preimage
	// length. This is because the call is `preimage.note_preimage(call)`, so the outer pallet/call
	// indices have a length of 2 bytes.
	pub(crate) fn create_print_output(&self, length_limit: u32) -> (CallOrHash, u32) {
		let print_output = if self.length > length_limit {
			CallOrHash::Hash(self.hash)
		} else {
			CallOrHash::Call(self.to_runtime_call())
		};
		(print_output, self.length)
	}

	// Decode into the typed call of the network it is for. Only the chains that referenda are
	// submitted on are supported.
	pub(crate) fn to_runtime_call(&self) -> NetworkRuntimeCall {
		match &self.network {
			Network::Kusama => NetworkRuntimeCall::Kusama(self.get_kusama_call().expect("kusama")),
			Network::KusamaAssetHub => NetworkRuntimeCall::KusamaAssetHub(
				self.get_kusama_asset_hub_call().expect("kusama asset hub"),
			),
			Network::Polkadot =>
				NetworkRuntimeCall::Polkadot(self.get_polkadot_call().expect("polkadot")),
			Network::PolkadotAssetHub => NetworkRuntimeCall::PolkadotAssetHub(
				self.get_polkadot_asset_hub_call().expect("polkadot asset hub"),
			),
			Network::PolkadotCollectives => NetworkRuntimeCall::PolkadotCollectives(
				self.get_polkadot_collectives_call().expect("collectives"),
			),
			_ => panic!("to do"),
		}
	}
}

// The set of calls that some user will need to sign and submit to initiate a referendum.
pub(crate) struct PossibleCallsToSubmit {
	// `Some` if using the Fellowship to Whitelist a call. The second value is the length of the
	// call, which may be relevant to the print output.
	//
	// ```
	// preimage.note(whitelist.whitelist_call(hash(proposal)));
	// ```
	pub(crate) preimage_for_whitelist_call: Option<(CallOrHash, u32)>,
	// The preimage for the public referendum. Should always be `Some`. When not using the
	// Whitelist, this will just be the proposal itself. When using the Whitelist, it will be the
	// proposal nested in a call to dispatch via Whitelist. The second value is the length of the
	// call, which may be relevant to the print output.
	//
	// ```
	// // Without Fellowship
	// preimage.note(proposal);
	//
	// // With Fellowship
	// preimage.note(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
	// ```
	pub(crate) preimage_for_public_referendum: Option<(CallOrHash, u32)>,
	// The actual submission of the Fellowship referendum to Whitelist a call. `None` when not using
	// Whitelist.
	pub(crate) fellowship_referendum_submission: Option<NetworkRuntimeCall>,
	// The actual submission of the public referendum. The `proposal` is the proposal itself when
	// not using the Whitelist, or the dispatch call with nested proposal when using the Whitelist.
	pub(crate) public_referendum_submission: Option<NetworkRuntimeCall>,
}
