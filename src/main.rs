use parity_scale_codec::Encode as _;
use subxt::ext::sp_core;

#[subxt::subxt(runtime_metadata_url = "wss://kusama-rpc.polkadot.io:443")]
pub mod kusama {}
use kusama::runtime_types::{
	frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
	kusama_runtime::{
		governance::origins::pallet_custom_origins::Origin as KusamaOpenGovOrigin, OriginCaller,
		RuntimeCall as KusamaRuntimeCall,
	},
	pallet_preimage::pallet::Call as PreimageCall,
	pallet_referenda::pallet::Call as ReferendaCall,
	pallet_utility::pallet::Call as UtilityCall,
	pallet_whitelist::pallet::Call as WhitelistCall,
};

#[subxt::subxt(runtime_metadata_url = "wss://rpc.polkadot.io:443")]
pub mod polkadot_relay {}

#[subxt::subxt(runtime_metadata_url = "wss://polkadot-collectives-rpc.polkadot.io:443")]
pub mod polkadot_collectives {}

// This is the thing you need to edit to use this!
fn get_the_actual_proposed_action() -> ProposalDetails {
	use NetworkTrack::*;
	use Output::*;
	return ProposalDetails {
		// The encoded proposal that we want to submit.
		proposal: "0x1233",
		// The OpenGov track that it will use.
		track: Kusama(KusamaOpenGovOrigin::WhitelistedCaller),
		// Choose if you just want to see the hex-encoded `CallData`, or get a link to Polkadot JS
		// Apps UI (`AppsUiLink`).
		output: AppsUiLink,
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
	track: NetworkTrack,
	// How you would like to view the output.
	output: Output,
	// Cutoff length in bytes for printing the output. If too long, it will print the hash of the
	// call you would need to submit so that you can verify before submission.
	output_len_limit: u32,
	// Whether or not to group all calls into a batch. Uses `force_batch` in case the account does
	// not have funds for pre-image deposits or is not a fellow.
	print_batch: bool,
}

enum NetworkTrack {
	Kusama(KusamaOpenGovOrigin),
}

enum NetworkRuntimeCall {
	Kusama(KusamaRuntimeCall),
}

#[allow(dead_code)]
enum Output {
	// Print just the call data (e.g. 0x1234).
	CallData,
	// Print a clickable link to view the decoded call on Polkadot JS Apps UI.
	AppsUiLink,
}

enum CallOrHash {
	Call(NetworkRuntimeCall),
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
	fellowship_referendum_submission: Option<NetworkRuntimeCall>,
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
	public_referendum_submission: Option<NetworkRuntimeCall>,
}

fn main() {
	let proposal_details = get_the_actual_proposed_action();
	let proposal_bytes = hex::decode(proposal_details.proposal.trim_start_matches("0x"))
		.expect("Valid proposal; qed");
	let proposal_hash = sp_core::blake2_256(&proposal_bytes);
	let proposal_len: u32 = (*&proposal_bytes.len()).try_into().unwrap();

	let calls: PossibleCallsToSubmit = match proposal_details.track {
		NetworkTrack::Kusama(kusama_track) => {
			let proposal_as_runtime_call =
				<KusamaRuntimeCall as parity_scale_codec::Decode>::decode(&mut &proposal_bytes[..])
					.unwrap();
			match kusama_track {
				KusamaOpenGovOrigin::WhitelistedCaller => {
					// First we need to whitelist this proposal. We will need:
					//   1. To wrap the proposal hash in `whitelist.whitelist_call()` and submit
					//      this as a preimage.
					//   2. To submit a referendum to the Fellowship Referenda pallet to dispatch
					//      this preimage.
					let whitelist_call =
						KusamaRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
							call_hash: sp_core::H256(proposal_hash),
						});
					let whitelist_call_hash = sp_core::blake2_256(&whitelist_call.encode());
					let whitelist_call_len: u32 =
						(*&whitelist_call.encode().len()).try_into().unwrap();
					let preimage_for_whitelist_call =
						KusamaRuntimeCall::Preimage(PreimageCall::note_preimage {
							bytes: whitelist_call.encode(),
						});

					let fellowship_proposal =
						KusamaRuntimeCall::FellowshipReferenda(ReferendaCall::submit {
							proposal_origin: Box::new(OriginCaller::Origins(
								KusamaOpenGovOrigin::Fellows,
							)),
							proposal: Lookup {
								hash: sp_core::H256(whitelist_call_hash),
								len: whitelist_call_len,
							},
							enactment_moment: DispatchTime::After(10),
						});

					// Now we put together the public referendum part. This still needs separate
					// logic because the actual proposal gets wrapped in a Whitelist call.
					let dispatch_whitelisted_call = KusamaRuntimeCall::Whitelist(
						WhitelistCall::dispatch_whitelisted_call_with_preimage {
							call: Box::new(proposal_as_runtime_call),
						},
					);
					let dispatch_whitelisted_call_hash =
						sp_core::blake2_256(&dispatch_whitelisted_call.encode());
					let dispatch_whitelisted_call_len: u32 =
						(*&dispatch_whitelisted_call.encode().len())
							.try_into()
							.unwrap();

					let preimage_for_dispatch_whitelisted_call =
						KusamaRuntimeCall::Preimage(PreimageCall::note_preimage {
							bytes: dispatch_whitelisted_call.encode(),
						});
					let public_proposal = KusamaRuntimeCall::Referenda(ReferendaCall::submit {
						proposal_origin: Box::new(OriginCaller::Origins(
							KusamaOpenGovOrigin::WhitelistedCaller,
						)),
						proposal: Lookup {
							hash: sp_core::H256(dispatch_whitelisted_call_hash),
							len: dispatch_whitelisted_call_len,
						},
						enactment_moment: DispatchTime::After(10),
					});

					// Check the lengths and prepare preimages for printing.
					let (whitelist_preimage_print, whitelist_preimage_print_len) =
						create_print_output(
							preimage_for_whitelist_call,
							proposal_details.output_len_limit,
						);
					let (dispatch_preimage_print, dispatch_preimage_print_len) =
						create_print_output(
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
						fellowship_referendum_submission: Some(NetworkRuntimeCall::Kusama(
							fellowship_proposal,
						)),
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
						public_referendum_submission: Some(NetworkRuntimeCall::Kusama(
							public_proposal,
						)),
					}
				}
				_ => {
					let note_proposal_preimage =
						KusamaRuntimeCall::Preimage(PreimageCall::note_preimage {
							bytes: proposal_bytes,
						});
					let public_proposal = KusamaRuntimeCall::Referenda(ReferendaCall::submit {
						proposal_origin: Box::new(OriginCaller::Origins(kusama_track)),
						proposal: Lookup {
							hash: sp_core::H256(proposal_hash),
							len: proposal_len,
						},
						enactment_moment: DispatchTime::After(10),
					});
					let (preimage_print, preimage_print_len) = create_print_output(
						note_proposal_preimage,
						proposal_details.output_len_limit,
					);

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
						public_referendum_submission: Some(NetworkRuntimeCall::Kusama(
							public_proposal,
						)),
					}
				}
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

	if proposal_details.print_batch {
		handle_batch_of_calls(&proposal_details.output, batch_of_calls);
	}
}

fn handle_batch_of_calls(output: &Output, batch: Vec<NetworkRuntimeCall>) {
	let mut kusama_relay_batch = Vec::new();
	// let mut polkadot_relay_batch = Vec::new();
	// let mut polkadot_collectives_batch = Vec::new();
	for network_call in batch {
		match network_call {
			NetworkRuntimeCall::Kusama(cc) => kusama_relay_batch.push(cc),
		}
	}
	if kusama_relay_batch.len() > 0 {
		let batch = KusamaRuntimeCall::Utility(UtilityCall::force_batch {
			calls: kusama_relay_batch,
		});
		println!("\nBatch to submit on Kusama Relay Chain:");
		print_output(output, &NetworkRuntimeCall::Kusama(batch));
	}
}

// Format the data to print to console.
fn print_output(output: &Output, network_call: &NetworkRuntimeCall) {
	match network_call {
		NetworkRuntimeCall::Kusama(call) => {
			let rpc: &'static str = "wss%3A%2F%2Fkusama-rpc.polkadot.io";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://polkadot.js.org/apps/?rpc={}#/extrinsics/decode/0x{}",
					rpc,
					hex::encode(call.encode())
				),
			}
		}
	}
}

// Take some call and a length limit as input. If the call length exceeds the limit, just return its
// hash. Call length is recomputed and will be 2 bytes longer than the actual preimage length. This
// is because the call is `preimage.note_preimage(call)`, so the outer pallet/call indices have a
// length of 2 bytes.
fn create_print_output(call: KusamaRuntimeCall, length_limit: u32) -> (CallOrHash, u32) {
	let call_len = (*&call.encode().len()).try_into().unwrap();
	let print_output: CallOrHash;
	if call_len > length_limit {
		let call_hash = sp_core::blake2_256(&call.encode());
		print_output = CallOrHash::Hash(call_hash);
	} else {
		print_output = CallOrHash::Call(NetworkRuntimeCall::Kusama(call))
	}
	(print_output, call_len)
}
