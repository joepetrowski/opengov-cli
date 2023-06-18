#[subxt::subxt(
	runtime_metadata_url = "wss://kusama-rpc.polkadot.io:443",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod kusama {}
pub(super) use kusama::runtime_types::kusama_runtime::{
	governance::origins::pallet_custom_origins::Origin as KusamaOpenGovOrigin,
	RuntimeCall as KusamaRuntimeCall,
};

#[subxt::subxt(
	runtime_metadata_url = "wss://rpc.polkadot.io:443",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod polkadot_relay {}
pub(super) use polkadot_relay::runtime_types::polkadot_runtime::{
	governance::origins::pallet_custom_origins::Origin as PolkadotOpenGovOrigin,
	RuntimeCall as PolkadotRuntimeCall,
};

#[subxt::subxt(runtime_metadata_url = "wss://polkadot-collectives-rpc.polkadot.io:443")]
pub mod polkadot_collectives {}
pub(super) use polkadot_collectives::runtime_types::collectives_polkadot_runtime::{
	fellowship::origins::pallet_origins::Origin as FellowshipOrigins,
	RuntimeCall as CollectivesRuntimeCall,
};

#[allow(dead_code)]
pub(super) enum Network {
	Kusama,
	Polkadot,
	PolkadotCollectives,
}

// Info and preferences provided by the user.
pub(super) struct ProposalDetails {
	// The proposal, generated elsewhere and pasted here.
	pub(super) proposal: &'static str,
	// The track to submit on.
	pub(super) track: NetworkTrack,
	// When do you want this to enact. `At(block)` or `After(blocks)`.
	pub(super) dispatch: DispatchTimeWrapper,
	// How you would like to view the output.
	pub(super) output: Output,
	// Cutoff length in bytes for printing the output. If too long, it will print the hash of the
	// call you would need to submit so that you can verify before submission.
	pub(super) output_len_limit: u32,
	// Whether or not to group all calls into a batch. Uses `force_batch` in case the account does
	// not have funds for pre-image deposits or is not a fellow.
	pub(super) print_batch: bool,
}

// The network and OpenGov track this proposal should be voted on.
#[allow(dead_code)]
pub(super) enum NetworkTrack {
	Kusama(KusamaOpenGovOrigin),
	Polkadot(PolkadotOpenGovOrigin),
}

// A runtime call wrapped in the network it should execute on.
#[allow(dead_code)]
pub(super) enum NetworkRuntimeCall {
	Kusama(KusamaRuntimeCall),
	Polkadot(PolkadotRuntimeCall),
	PolkadotCollectives(CollectivesRuntimeCall),
}

// How the user would like to see the output of the program.
#[allow(dead_code)]
pub(super) enum Output {
	// Print just the call data (e.g. 0x1234).
	CallData,
	// Print a clickable link to view the decoded call on Polkadot JS Apps UI.
	AppsUiLink,
}

// Local concrete type to use in each runtime's `DispatchTime`
#[allow(dead_code)]
pub(super) enum DispatchTimeWrapper {
	At(u32),
	After(u32),
}

// A call or a hash. Used for printing (or rather, to avoid printing large calls).
pub(super) enum CallOrHash {
	Call(NetworkRuntimeCall),
	Hash([u8; 32]),
}

// All the info associated with a call in the forms you may need it in.
pub(super) struct CallInfo {
	pub(super) call: NetworkRuntimeCall,
	pub(super) encoded: Vec<u8>,
	pub(super) hash: [u8; 32],
	pub(super) length: u32,
}

// The set of calls that some user will need to sign and submit to initiate a referendum.
pub(super) struct PossibleCallsToSubmit {
	// ```
	// preimage.note(whitelist.whitelist_call(hash(proposal)));
	// ```
	pub(super) preimage_for_whitelist_call: Option<(CallOrHash, u32)>,
	// ```
	// // Without Fellowship
	// preimage.note(proposal);
	//
	// // With Fellowship
	// preimage.note(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
	// ```
	pub(super) preimage_for_public_referendum: Option<(CallOrHash, u32)>,
	// ```
	// fellowship_referenda.submit(
	//     proposal_origin: Fellows,
	//     proposal: Lookup {
	//         hash: hash(whitelist.whitelist_call(proposal_hash)),
	//         len: len(whitelist.whitelist_call(proposal_hash)),
	//     },
	//     enactment_moment: After(10),
	// )
	// ```
	pub(super) fellowship_referendum_submission: Option<NetworkRuntimeCall>,
	// ```
	// referenda.submit(
	//     proposal_origin: ProposalDetails.track,
	//     proposal: Lookup {
	// //            No whitelist   ||  Whitelist
	//         hash: hash(proposal) OR hash(whitelist.whitelist_call(proposal_hash)),
	//         len:  len(proposal)  OR len(whitelist.dispatch_whitelisted_call_with_preimage(proposal)),
	//     },
	//     enactment_moment: After(10),
	// )
	// ```
	pub(super) public_referendum_submission: Option<NetworkRuntimeCall>,
}
