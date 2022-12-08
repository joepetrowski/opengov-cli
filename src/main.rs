#[subxt::subxt(runtime_metadata_url = "wss://kusama-rpc.polkadot.io:443")]
pub mod kusama {
	#[subxt(substitute_type = "sp_runtime::multiaddress::MultiAddress")]
	use ::subxt::ext::sp_runtime::MultiAddress;
}

use parity_scale_codec::Encode as _;
use subxt::ext::sp_core;
use kusama::runtime_types::{
	frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
	kusama_runtime::{
		governance::origins::pallet_custom_origins::Origin as OpenGovOrigin,
		OriginCaller,
		RuntimeCall,
	},
	pallet_whitelist::pallet::Call as WhitelistCall,
	pallet_referenda::pallet::Call as ReferendaCall,
	pallet_preimage::pallet::Call as PreimageCall,
};

struct ProposalDetails {
	proposal: &'static str,
	track: OpenGovOrigin,
}

fn get_the_actual_proposed_action() -> ProposalDetails {
	return ProposalDetails {
		proposal: "0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a",
		track: OpenGovOrigin::WhitelistedCaller,
	}
}

// The set of calls that some user will need to sign and submit to initiate a referendum.
struct PossibleCallsToSubmit {
	// ```
	// preimage.note(whitelist.whitelist_call(hash(proposal)));
	// ```
	preimage_for_whitelist_call: Option<RuntimeCall>,
	// ```
	// // Without Fellowship
	// preimage.note(proposal);
	//
	// // With Fellowship
	// preimage.note(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
	// ```
	preimage_for_public_referendum: Option<RuntimeCall>,
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
	fellowship_referendum_submission: Option<RuntimeCall>,
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
	public_referendum_submission: Option<RuntimeCall>,
}

fn main() {
	let proposal_details = get_the_actual_proposed_action();
	let proposal_bytes = hex::decode(proposal_details.proposal.trim_start_matches("0x")).expect("Valid proposal; qed");
	let proposal_hash = sp_core::blake2_256(&proposal_bytes);
	let proposal_len: u32 = (*&proposal_bytes.len()).try_into().unwrap();

	let calls: PossibleCallsToSubmit = match proposal_details.track {
		OpenGovOrigin::WhitelistedCaller => {
			// First we need to whitelist this proposal. We will need:
			//   1. To wrap the proposal hash in `whitelist.whitelist_call()` and submit this as a preimage.
			//   2. To submit a referendum to the Fellowship Referenda pallet to dispatch this preimage.
			let whitelist_call = RuntimeCall::Whitelist(WhitelistCall::whitelist_call {
				call_hash: sp_core::H256(proposal_hash)
			});
			let whitelist_call_hash = sp_core::blake2_256(&whitelist_call.encode());
			let whitelist_call_len: u32 = (*&whitelist_call.encode().len()).try_into().unwrap();
			let preimage_for_whitelist_call = RuntimeCall::Preimage(PreimageCall::note_preimage {
				bytes: whitelist_call.encode(),
			});

			let fellowship_proposal = RuntimeCall::FellowshipReferenda(ReferendaCall::submit {
				proposal_origin: Box::new(OriginCaller::Origins(OpenGovOrigin::Fellows)),
				proposal: Lookup {
					hash: sp_core::H256(whitelist_call_hash),
					len: whitelist_call_len,
				},
				enactment_moment: DispatchTime::After(10),
			});

			// Now we put together the public referendum part. This still needs separate logic
			// because the actual proposal gets wrapped in a Whitelist call.
			let dispatch_whitelisted_call = RuntimeCall::Whitelist(
				WhitelistCall::dispatch_whitelisted_call_with_preimage { call: proposal_bytes }
			);
			let dispatch_whitelisted_call_hash =
				sp_core::blake2_256(&dispatch_whitelisted_call.encode());
			let dispatch_whitelisted_call_len: u32 =
				(*&dispatch_whitelisted_call.encode().len()).try_into().unwrap();
			
			let preimage_for_dispatch_whitelisted_call = RuntimeCall::Preimage(
				PreimageCall::note_preimage { bytes: dispatch_whitelisted_call.encode() }
			);
			let public_proposal = RuntimeCall::Referenda(ReferendaCall::submit {
				proposal_origin: Box::new(OriginCaller::Origins(OpenGovOrigin::WhitelistedCaller)),
				proposal: Lookup {
					hash: sp_core::H256(dispatch_whitelisted_call_hash),
					len: dispatch_whitelisted_call_len,
				},
				enactment_moment: DispatchTime::After(10),
			});
			PossibleCallsToSubmit {
				// preimage.note_preimage(whitelist.whitelist_call(hash(proposal)));
				preimage_for_whitelist_call: Some(preimage_for_whitelist_call),
				// preimage.note_preimage(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
				preimage_for_public_referendum: Some(preimage_for_dispatch_whitelisted_call),
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
				fellowship_referendum_submission: Some(fellowship_proposal),
				// ```
				// referenda.submit(
				//     proposal_origin: WhitelistedCaller,
				//     proposal: Lookup {
				//         hash: hash(whitelist.whitelist_call(proposal_hash)),
				//         len: len(whitelist.dispatch_whitelisted_call_with_preimage(proposal)),
				//     },
				//     enactment_moment: After(10),
				// )
				// ```
				public_referendum_submission: Some(public_proposal),
			}
		},
		_ => {
			let note_proposal_preimage =
				RuntimeCall::Preimage(PreimageCall::note_preimage { bytes: proposal_bytes });
			let public_proposal = RuntimeCall::Referenda(ReferendaCall::submit {
				proposal_origin: Box::new(OriginCaller::Origins(proposal_details.track)),
				proposal: Lookup {
					hash: sp_core::H256(proposal_hash),
					len: proposal_len,
				},
				enactment_moment: DispatchTime::After(10),
			});
			PossibleCallsToSubmit {
				// None
				preimage_for_whitelist_call: None,
				// preimage.note_preimage(proposal);
				preimage_for_public_referendum: Some(note_proposal_preimage),
				// None
				fellowship_referendum_submission: None,
				// ```
				// referenda.submit(
				//     proposal_origin: ProposalDetails.track,
				//     proposal: Lookup {
				//         hash: hash(proposal),
				//         len:  len(proposal),
				//     },
				//     enactment_moment: After(10),
				// )
				// ```
				public_referendum_submission: Some(public_proposal),
			}
		},
	};

	if let Some(c) = calls.preimage_for_whitelist_call {
		println!("\nSubmit the preimage for the Fellowship referendum:");
		println!("0x{}", hex::encode(c.encode()));
	}
	if let Some(c) = calls.fellowship_referendum_submission {
		println!("\nOpen a Fellowship referendum to whitelist the call:");
		println!("0x{}", hex::encode(c.encode()));
	}
	if let Some(c) = calls.preimage_for_public_referendum {
		println!("\nSubmit the preimage for the public referendum:");
		println!("0x{}", hex::encode(c.encode()));
	}
	if let Some(c) = calls.public_referendum_submission {
		println!("\nOpen a public referendum to dispatch the whitelisted call:");
		println!("0x{}", hex::encode(c.encode()));
	}
}
