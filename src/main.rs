#[subxt::subxt(runtime_metadata_url = "wss://kusama-rpc.polkadot.io:443")]
pub mod kusama {}

use kusama::runtime_types::{
	frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
	kusama_runtime::{
		governance::origins::pallet_custom_origins::Origin as OpenGovOrigin, OriginCaller,
		RuntimeCall,
	},
	pallet_preimage::pallet::Call as PreimageCall,
	pallet_referenda::pallet::Call as ReferendaCall,
	pallet_utility::pallet::Call as UtilityCall,
	pallet_whitelist::pallet::Call as WhitelistCall,
};
use parity_scale_codec::Encode as _;
use subxt::ext::sp_core;

// This is the thing you need to edit to use this!
fn get_the_actual_proposed_action() -> ProposalDetails {
	return ProposalDetails {
		// The encoded proposal that we want to submit.
		proposal: "0x180010630001000100a10f0204060202286bee880102fe5476bc7ba7c8044e1fd07b20aa90523709fb25ceb177b7cf1b54bbf2ce7689630001000100a90f0204060202286bee880102c5eecc0cff46384cc3d2e77f4f6ddf5c7c0f7092967edf50f3cacdff362cb7391d045802000000006304000100a10f030000001d045802000000006304000100a90f03000000",
		// The OpenGov track that it will use.
		track: OpenGovOrigin::WhitelistedCaller,
		// Choose if you just want to see the hex-encoded `CallData`, or get a link to Polkadot JS
		// Apps UI (`AppsUiLink`).
		output: Output::AppsUiLink,
		// Limit the length of calls printed to console. Prevents massive hex dumps for proposals
		// like runtime upgrades.
		output_len_limit: 1_000,
		// Whether or not to print a single `force_batch` call.
		print_batch: true,
	};
}

// Info and preferences provided by the user.
struct ProposalDetails {
	// The proposal, generated elsewhere and pasted here.
	proposal: &'static str,
	// The track to submit on.
	track: OpenGovOrigin,
	// How you would like to view the output.
	output: Output,
	// Cutoff length in bytes for printing the output. If too long, it will print the hash of the
	// call you would need to submit so that you can verify before submission.
	output_len_limit: u32,
	// Whether or not to group all calls into a batch. Uses `force_batch` in case the account does
	// not have funds for pre-image deposits or is not a fellow.
	print_batch: bool,
}

#[allow(dead_code)]
enum Output {
	// Print just the call data (e.g. 0x1234).
	CallData,
	// Print a clickable link to view the decoded call on Polkadot JS Apps UI.
	AppsUiLink,
}

enum CallOrHash {
	Call(RuntimeCall),
	Hash([u8; 32]),
}

// The set of calls that some user will need to sign and submit to initiate a referendum.
struct PossibleCallsToSubmit {
	// ```
	// preimage.note(whitelist.whitelist_call(hash(proposal)));
	// ```
	preimage_for_whitelist_call: Option<(CallOrHash, u32)>,
	// ```
	// // Without Fellowship
	// preimage.note(proposal);
	//
	// // With Fellowship
	// preimage.note(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
	// ```
	preimage_for_public_referendum: Option<(CallOrHash, u32)>,
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
	let proposal_bytes = hex::decode(proposal_details.proposal.trim_start_matches("0x"))
		.expect("Valid proposal; qed");
	let proposal_as_runtime_call =
		<RuntimeCall as parity_scale_codec::Decode>::decode(&mut &proposal_bytes[..]).unwrap();
	let proposal_hash = sp_core::blake2_256(&proposal_bytes);
	let proposal_len: u32 = (*&proposal_bytes.len()).try_into().unwrap();

	let calls: PossibleCallsToSubmit = match proposal_details.track {
		OpenGovOrigin::WhitelistedCaller => {
			// First we need to whitelist this proposal. We will need:
			//   1. To wrap the proposal hash in `whitelist.whitelist_call()` and submit this as a
			//      preimage.
			//   2. To submit a referendum to the Fellowship Referenda pallet to dispatch this
			//      preimage.
			let whitelist_call = RuntimeCall::Whitelist(WhitelistCall::whitelist_call {
				call_hash: sp_core::H256(proposal_hash),
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
			let dispatch_whitelisted_call =
				RuntimeCall::Whitelist(WhitelistCall::dispatch_whitelisted_call_with_preimage {
					call: Box::new(proposal_as_runtime_call),
				});
			let dispatch_whitelisted_call_hash =
				sp_core::blake2_256(&dispatch_whitelisted_call.encode());
			let dispatch_whitelisted_call_len: u32 = (*&dispatch_whitelisted_call.encode().len())
				.try_into()
				.unwrap();

			let preimage_for_dispatch_whitelisted_call =
				RuntimeCall::Preimage(PreimageCall::note_preimage {
					bytes: dispatch_whitelisted_call.encode(),
				});
			let public_proposal = RuntimeCall::Referenda(ReferendaCall::submit {
				proposal_origin: Box::new(OriginCaller::Origins(OpenGovOrigin::WhitelistedCaller)),
				proposal: Lookup {
					hash: sp_core::H256(dispatch_whitelisted_call_hash),
					len: dispatch_whitelisted_call_len,
				},
				enactment_moment: DispatchTime::After(10),
			});

			// Check the lengths and prepare preimages for printing.
			let (whitelist_preimage_print, whitelist_preimage_print_len) = create_print_output(
				preimage_for_whitelist_call,
				proposal_details.output_len_limit,
			);
			let (dispatch_preimage_print, dispatch_preimage_print_len) = create_print_output(
				preimage_for_dispatch_whitelisted_call,
				proposal_details.output_len_limit,
			);

			PossibleCallsToSubmit {
				// preimage.note_preimage(whitelist.whitelist_call(hash(proposal)));
				preimage_for_whitelist_call: Some((
					whitelist_preimage_print,
					whitelist_preimage_print_len,
				)),
				// preimage.note_preimage(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
				preimage_for_public_referendum: Some((
					dispatch_preimage_print,
					dispatch_preimage_print_len,
				)),
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
		}
		_ => {
			let note_proposal_preimage = RuntimeCall::Preimage(PreimageCall::note_preimage {
				bytes: proposal_bytes,
			});
			let public_proposal = RuntimeCall::Referenda(ReferendaCall::submit {
				proposal_origin: Box::new(OriginCaller::Origins(proposal_details.track)),
				proposal: Lookup {
					hash: sp_core::H256(proposal_hash),
					len: proposal_len,
				},
				enactment_moment: DispatchTime::After(10),
			});
			let (preimage_print, preimage_print_len) =
				create_print_output(note_proposal_preimage, proposal_details.output_len_limit);
			PossibleCallsToSubmit {
				// None
				preimage_for_whitelist_call: None,
				// preimage.note_preimage(proposal);
				preimage_for_public_referendum: Some((preimage_print, preimage_print_len)),
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
		}
	};

	let mut batch_of_calls = Vec::new();

	if let Some((call_or_hash, len)) = calls.preimage_for_whitelist_call {
		match call_or_hash {
			CallOrHash::Call(c) => {
				println!("\nSubmit the preimage for the Fellowship referendum:");
				print_output(&proposal_details.output, &c);
				batch_of_calls.push(c);
			}
			CallOrHash::Hash(h) => {
				println!(
					"\nPreimage for the public whitelist call too large ({} bytes). Not included in batch.",
					len
				);
				println!("Submission should have the hash: 0x{}", hex::encode(h));
			}
		}
	}
	if let Some(c) = calls.fellowship_referendum_submission {
		println!("\nOpen a Fellowship referendum to whitelist the call:");
		print_output(&proposal_details.output, &c);
		batch_of_calls.push(c);
	}
	if let Some((call_or_hash, len)) = calls.preimage_for_public_referendum {
		match call_or_hash {
			CallOrHash::Call(c) => {
				println!("\nSubmit the preimage for the public referendum:");
				print_output(&proposal_details.output, &c);
				batch_of_calls.push(c);
			}
			CallOrHash::Hash(h) => {
				println!(
					"\nPreimage for the public referendum too large ({} bytes). Not included in batch.",
					len
				);
				println!("Submission should have the hash: 0x{}", hex::encode(h));
			}
		}
	}
	if let Some(c) = calls.public_referendum_submission {
		println!("\nOpen a public referendum to dispatch the call:");
		print_output(&proposal_details.output, &c);
		batch_of_calls.push(c);
	}

	let batch = RuntimeCall::Utility(UtilityCall::force_batch { calls: batch_of_calls });
	if proposal_details.print_batch {
		println!("\nBatch including all calls:");
		print_output(&proposal_details.output, &batch);
	}

}

// Format the data to print to console.
fn print_output(output: &Output, call: &RuntimeCall) {
	match output {
		Output::CallData => println!("0x{}", hex::encode(call.encode())),
		Output::AppsUiLink => println!("https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fkusama-rpc.polkadot.io#/extrinsics/decode/0x{}", hex::encode(call.encode())),
	}
}

// Take some call and a length limit as input. If the call length exceeds the limit, just return its
// hash. Call length is recomputed and will be 2 bytes longer than the actual preimage length. This
// is because the call is `preimage.note_preimage(call)`, so the outer pallet/call indices have a
// length of 2 bytes.
fn create_print_output(call: RuntimeCall, length_limit: u32) -> (CallOrHash, u32) {
	let call_len = (*&call.encode().len()).try_into().unwrap();
	let print_output: CallOrHash;
	if call_len > length_limit {
		let call_hash = sp_core::blake2_256(&call.encode());
		print_output = CallOrHash::Hash(call_hash);
	} else {
		print_output = CallOrHash::Call(call)
	}
	(print_output, call_len)
}
