use std::fs;
mod types;
use crate::types::*;

#[cfg(test)]
mod tests;

// This is the thing you need to edit to use this!
fn get_the_actual_proposed_action() -> ProposalDetails {
	use DispatchTimeWrapper::*;
	use NetworkTrack::*;
	use Output::*;
	return ProposalDetails {
		// The encoded proposal that we want to submit. This can either be the call data itself,
		// e.g. "0x0102...", or a file path that contains the data, e.g. "./my_proposal.call".
		proposal: "0x0000645468652046656c6c6f777368697020736179732068656c6c6f",
		// The OpenGov track that it will use.
		track: Polkadot(PolkadotOpenGovOrigin::WhitelistedCaller),
		// When do you want this to enact. `At(block)` or `After(blocks)`.
		dispatch: After(10),
		// Choose if you just want to see the hex-encoded `CallData`, or get a link to Polkadot JS
		// Apps UI (`AppsUiLink`).
		output: AppsUiLink,
		// Limit the length of calls printed to console. Prevents massive hex dumps for proposals
		// like runtime upgrades.
		output_len_limit: 1_000,
		// Whether or not to print a single `force_batch` call.
		print_batch: true,
		// If `None`, will fetch the needed weight from an API. You probably want `None`, unless
		// you know what you're doing.
		transact_weight_override: None,
	}
}

#[tokio::main]
async fn main() {
	// Find out what the user wants to do.
	let proposal_details = get_the_actual_proposed_action();
	// Generate the calls necessary.
	let calls = generate_calls(&proposal_details).await;
	// Tell the user what to do.
	deliver_output(proposal_details, calls);
}

async fn generate_calls(proposal_details: &ProposalDetails) -> PossibleCallsToSubmit {
	let proposal_bytes = get_proposal_bytes(proposal_details.proposal);

	match &proposal_details.track {
		NetworkTrack::Kusama(kusama_track) => {
			use kusama::runtime_types::{
				frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
				kusama_runtime::OriginCaller,
				pallet_preimage::pallet::Call as PreimageCall,
				pallet_referenda::pallet::Call as ReferendaCall,
				pallet_referenda::pallet::Call2 as FellowshipReferendaCall,
				pallet_whitelist::pallet::Call as WhitelistCall,
			};

			let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Kusama);

			let public_referendum_dispatch_time = match proposal_details.dispatch {
				DispatchTimeWrapper::At(block) => DispatchTime::At(block),
				DispatchTimeWrapper::After(block) => DispatchTime::After(block),
			};

			match kusama_track {
				// Whitelisted calls are special.
				KusamaOpenGovOrigin::WhitelistedCaller => {
					// First we need to whitelist this proposal. We will need:
					//   1. To wrap the proposal hash in `whitelist.whitelist_call()` and submit
					//      this as a preimage.
					//   2. To submit a referendum to the Fellowship Referenda pallet to dispatch
					//      this preimage.
					let whitelist_call = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
						KusamaRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
							call_hash: sp_core::H256(proposal_call_info.hash),
						}),
					));
					let preimage_for_whitelist_call = CallInfo::from_runtime_call(
						NetworkRuntimeCall::Kusama(KusamaRuntimeCall::Preimage(
							PreimageCall::note_preimage { bytes: whitelist_call.encoded },
						)),
					);

					let fellowship_proposal = CallInfo::from_runtime_call(
						NetworkRuntimeCall::Kusama(KusamaRuntimeCall::FellowshipReferenda(
							FellowshipReferendaCall::submit {
								proposal_origin: Box::new(OriginCaller::Origins(
									KusamaOpenGovOrigin::Fellows,
								)),
								proposal: Lookup {
									hash: sp_core::H256(whitelist_call.hash),
									len: whitelist_call.length,
								},
								enactment_moment: DispatchTime::After(10),
							},
						)),
					);

					// Now we put together the public referendum part. This still needs separate
					// logic because the actual proposal gets wrapped in a Whitelist call.
					let dispatch_whitelisted_call = CallInfo::from_runtime_call(
						NetworkRuntimeCall::Kusama(KusamaRuntimeCall::Whitelist(
							WhitelistCall::dispatch_whitelisted_call_with_preimage {
								call: Box::new(
									proposal_call_info.get_kusama_call().expect("kusama"),
								),
							},
						)),
					);

					let preimage_for_dispatch_whitelisted_call =
						CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
							KusamaRuntimeCall::Preimage(PreimageCall::note_preimage {
								bytes: dispatch_whitelisted_call.encoded.clone(),
							}),
						));
					let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
						KusamaRuntimeCall::Referenda(ReferendaCall::submit {
							proposal_origin: Box::new(OriginCaller::Origins(
								KusamaOpenGovOrigin::WhitelistedCaller,
							)),
							proposal: Lookup {
								hash: sp_core::H256(dispatch_whitelisted_call.hash),
								len: dispatch_whitelisted_call.length,
							},
							enactment_moment: public_referendum_dispatch_time,
						}),
					));

					// Check the lengths and prepare preimages for printing.
					let (whitelist_preimage_print, whitelist_preimage_print_len) =
						preimage_for_whitelist_call
							.create_print_output(proposal_details.output_len_limit);
					let (dispatch_preimage_print, dispatch_preimage_print_len) =
						preimage_for_dispatch_whitelisted_call
							.create_print_output(proposal_details.output_len_limit);

					// If it's a hash, let's write the data to a file you can upload.
					match dispatch_preimage_print {
						CallOrHash::Call(_) => (),
						CallOrHash::Hash(_) => {
							let mut info_to_write = "0x".to_owned();
							info_to_write
								.push_str(hex::encode(dispatch_whitelisted_call.encoded).as_str());
							fs::write(
								"kusama_relay_public_referendum_preimage_to_note.call",
								info_to_write,
							)
							.expect("it should write");
						},
					}

					PossibleCallsToSubmit {
						preimage_for_whitelist_call: Some((
							whitelist_preimage_print,
							whitelist_preimage_print_len,
						)),
						preimage_for_public_referendum: Some((
							dispatch_preimage_print,
							dispatch_preimage_print_len,
						)),
						fellowship_referendum_submission: Some(fellowship_proposal.call),
						public_referendum_submission: Some(public_proposal.call),
					}
				},
				// Everything else just uses its track.
				_ => {
					let note_proposal_preimage = CallInfo::from_runtime_call(
						NetworkRuntimeCall::Kusama(KusamaRuntimeCall::Preimage(
							PreimageCall::note_preimage { bytes: proposal_bytes },
						)),
					);
					let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
						KusamaRuntimeCall::Referenda(ReferendaCall::submit {
							proposal_origin: Box::new(OriginCaller::Origins(kusama_track.clone())),
							proposal: Lookup {
								hash: sp_core::H256(proposal_call_info.hash),
								len: proposal_call_info.length,
							},
							enactment_moment: public_referendum_dispatch_time,
						}),
					));
					let (preimage_print, preimage_print_len) = note_proposal_preimage
						.create_print_output(proposal_details.output_len_limit);

					PossibleCallsToSubmit {
						preimage_for_whitelist_call: None,
						preimage_for_public_referendum: Some((preimage_print, preimage_print_len)),
						fellowship_referendum_submission: None,
						public_referendum_submission: Some(public_proposal.call),
					}
				},
			}
		},
		NetworkTrack::Polkadot(polkadot_track) => {
			use polkadot_collectives::runtime_types::{
				collectives_polkadot_runtime::OriginCaller as CollectivesOriginCaller,
				frame_support::traits::{
					preimages::Bounded::Lookup as CollectivesLookup,
					schedule::DispatchTime as CollectivesDispatchTime,
				},
				pallet_preimage::pallet::Call as CollectivesPreimageCall,
				pallet_referenda::pallet::Call as FellowshipReferendaCall,
				pallet_xcm::pallet::Call as CollectivesXcmCall,
				xcm::{
					double_encoded::DoubleEncoded,
					v2::OriginKind,
					v3::{
						junctions::Junctions::Here, multilocation::MultiLocation, Instruction,
						WeightLimit, Xcm,
					},
					VersionedMultiLocation,
					VersionedXcm::V3,
				},
			};
			use polkadot_relay::runtime_types::{
				frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
				pallet_preimage::pallet::Call as PreimageCall,
				pallet_referenda::pallet::Call as ReferendaCall,
				pallet_whitelist::pallet::Call as WhitelistCall,
				polkadot_runtime::OriginCaller,
			};

			let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Polkadot);

			let public_referendum_dispatch_time = match proposal_details.dispatch {
				DispatchTimeWrapper::At(block) => DispatchTime::At(block),
				DispatchTimeWrapper::After(block) => DispatchTime::After(block),
			};

			match polkadot_track {
				// Fellowship is on the Collectives parachain, so things are a bit different here.
				//
				// 1. Create a whitelist call on the Relay Chain:
				//
				//    let whitelist_call =
				//     	  PolkadotRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
				// 		      call_hash: sp_core::H256(proposal_hash),
				// 	      });
				//
				// 2. Create an XCM send call on the Collectives chain to Transact this on the
				//    Relay Chain:
				//
				//    let send_whitelist = CollectivesRuntimeCall::PolkadotXcm(
				//        PolkadotXcmCall::send {
				// 	          dest: MultiLocation { parents: 1, interior: Here },
				// 	          message: vec![UnpaidExecution, Transact {call: whitelist_call, ..}],
				//        }
				//    );
				//
				// 3. Make a Fellowship referendum for `send_whitelist`.
				//
				// 4. Relay Chain public referendum should be the same as on Kusama.
				PolkadotOpenGovOrigin::WhitelistedCaller => {
					// Whitelist the call on the Relay Chain.
					let whitelist_call = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
						PolkadotRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
							call_hash: sp_core::H256(proposal_call_info.hash),
						}),
					));

					let (ref_time, proof_size) =
						// The user may want to override the computed values, e.g. for deterministic
						// testing.
						if let Some(weight_override) = &proposal_details.transact_weight_override {
							(weight_override.ref_time, weight_override.proof_size)
						} else {
							// Do some weight calculation for execution of Transact on the Relay
							// Chain.
							let max_ref_time: u64 = 2_000_000_000_000 - 1;
							let max_proof_size: u64 = 5 * 1024 * 1024 - 1;
							let relay_weight_needed =
								whitelist_call.get_transact_weight_needed(Network::Polkadot).await;
							// Double the weight needed, just to be safe from a runtime upgrade that
							// could change things during the referendum period.
							(
								(2 * relay_weight_needed.ref_time).min(max_ref_time),
								(2 * relay_weight_needed.proof_size).min(max_proof_size),
							)
						};

					// This is what the Fellowship will actually vote on enacting.
					let whitelist_over_xcm =
						CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
							CollectivesRuntimeCall::PolkadotXcm(CollectivesXcmCall::send {
								dest: Box::new(VersionedMultiLocation::V3(MultiLocation {
									parents: 1,
									interior: Here,
								})),
								message: Box::new(V3(Xcm(vec![
									Instruction::UnpaidExecution {
										weight_limit: WeightLimit::Unlimited,
										check_origin: None,
									},
									Instruction::Transact {
										origin_kind: OriginKind::Xcm,
										require_weight_at_most: Weight { ref_time, proof_size },
										call: DoubleEncoded { encoded: whitelist_call.encoded },
									},
								]))),
							}),
						));

					let preimage_for_whitelist_over_xcm = CallInfo::from_runtime_call(
						NetworkRuntimeCall::PolkadotCollectives(CollectivesRuntimeCall::Preimage(
							CollectivesPreimageCall::note_preimage {
								bytes: whitelist_over_xcm.encoded,
							},
						)),
					);

					// The actual Fellowship referendum submission.
					let fellowship_proposal =
						CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
							CollectivesRuntimeCall::FellowshipReferenda(
								FellowshipReferendaCall::submit {
									proposal_origin: Box::new(
										CollectivesOriginCaller::FellowshipOrigins(
											FellowshipOrigins::Fellows,
										),
									),
									proposal: CollectivesLookup {
										hash: sp_core::H256(whitelist_over_xcm.hash),
										len: whitelist_over_xcm.length,
									},
									enactment_moment: CollectivesDispatchTime::After(10u32),
								},
							),
						));

					// Now we put together the public referendum part. This still needs separate
					// logic because the actual proposal gets wrapped in a Whitelist call.
					let dispatch_whitelisted_call = CallInfo::from_runtime_call(
						NetworkRuntimeCall::Polkadot(PolkadotRuntimeCall::Whitelist(
							WhitelistCall::dispatch_whitelisted_call_with_preimage {
								call: Box::new(
									proposal_call_info
										.get_polkadot_call()
										.expect("it is a polkadot call"),
								),
							},
						)),
					);

					let preimage_for_dispatch_whitelisted_call =
						CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
							PolkadotRuntimeCall::Preimage(PreimageCall::note_preimage {
								bytes: dispatch_whitelisted_call.encoded.clone(),
							}),
						));
					let public_proposal =
						CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
							PolkadotRuntimeCall::Referenda(ReferendaCall::submit {
								proposal_origin: Box::new(OriginCaller::Origins(
									PolkadotOpenGovOrigin::WhitelistedCaller,
								)),
								proposal: Lookup {
									hash: sp_core::H256(dispatch_whitelisted_call.hash),
									len: dispatch_whitelisted_call.length,
								},
								enactment_moment: public_referendum_dispatch_time,
							}),
						));

					// Check the lengths and prepare preimages for printing.
					let (whitelist_over_xcm_preimage_print, whitelist_over_xcm_preimage_print_len) =
						preimage_for_whitelist_over_xcm
							.create_print_output(proposal_details.output_len_limit);
					let (dispatch_preimage_print, dispatch_preimage_print_len) =
						preimage_for_dispatch_whitelisted_call
							.create_print_output(proposal_details.output_len_limit);

					// If it's a hash, let's write the data to a file you can upload.
					match dispatch_preimage_print {
						CallOrHash::Call(_) => (),
						CallOrHash::Hash(_) => {
							let mut info_to_write = "0x".to_owned();
							info_to_write
								.push_str(hex::encode(dispatch_whitelisted_call.encoded).as_str());
							fs::write(
								"polkadot_relay_public_referendum_preimage_to_note.call",
								info_to_write,
							)
							.expect("it should write");
						},
					}

					PossibleCallsToSubmit {
						preimage_for_whitelist_call: Some((
							whitelist_over_xcm_preimage_print,
							whitelist_over_xcm_preimage_print_len,
						)),
						preimage_for_public_referendum: Some((
							dispatch_preimage_print,
							dispatch_preimage_print_len,
						)),
						fellowship_referendum_submission: Some(fellowship_proposal.call),
						public_referendum_submission: Some(public_proposal.call),
					}
				},
				_ => {
					let note_proposal_preimage = CallInfo::from_runtime_call(
						NetworkRuntimeCall::Polkadot(PolkadotRuntimeCall::Preimage(
							PreimageCall::note_preimage { bytes: proposal_bytes },
						)),
					);
					let public_proposal =
						CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
							PolkadotRuntimeCall::Referenda(ReferendaCall::submit {
								proposal_origin: Box::new(OriginCaller::Origins(
									polkadot_track.clone(),
								)),
								proposal: Lookup {
									hash: sp_core::H256(proposal_call_info.hash),
									len: proposal_call_info.length,
								},
								enactment_moment: public_referendum_dispatch_time,
							}),
						));
					let (preimage_print, preimage_print_len) = note_proposal_preimage
						.create_print_output(proposal_details.output_len_limit);

					PossibleCallsToSubmit {
						preimage_for_whitelist_call: None,
						preimage_for_public_referendum: Some((preimage_print, preimage_print_len)),
						fellowship_referendum_submission: None,
						public_referendum_submission: Some(public_proposal.call),
					}
				},
			}
		},
	}
}

fn deliver_output(proposal_details: ProposalDetails, calls: PossibleCallsToSubmit) {
	let mut batch_of_calls = Vec::new();

	if let Some((call_or_hash, len)) = calls.preimage_for_whitelist_call {
		match call_or_hash {
			CallOrHash::Call(c) => {
				println!("\nSubmit the preimage for the Fellowship referendum:");
				print_output(&proposal_details.output, &c);
				batch_of_calls.push(c);
			},
			CallOrHash::Hash(h) => {
				println!(
					"\nPreimage for the public whitelist call too large ({} bytes). Not included in batch.",
					len
				);
				println!("Submission should have the hash: 0x{}", hex::encode(h));
			},
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
			},
			CallOrHash::Hash(h) => {
				println!(
					"\nPreimage for the public referendum too large ({} bytes). Not included in batch.",
					len
				);
				println!("A file was created that you can upload in `preimage.note_preimage` in Apps UI.");
				println!("Submission should have the hash: 0x{}", hex::encode(h));
			},
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
	use kusama::runtime_types::pallet_utility::pallet::Call as KusamaUtilityCall;
	use polkadot_collectives::runtime_types::pallet_utility::pallet::Call as CollectivesUtilityCall;
	use polkadot_relay::runtime_types::pallet_utility::pallet::Call as PolkadotRelayUtilityCall;

	let mut kusama_relay_batch = Vec::new();
	let mut polkadot_relay_batch = Vec::new();
	let mut polkadot_collectives_batch = Vec::new();

	for network_call in batch {
		match network_call {
			NetworkRuntimeCall::Kusama(cc) => kusama_relay_batch.push(cc),
			NetworkRuntimeCall::Polkadot(cc) => polkadot_relay_batch.push(cc),
			NetworkRuntimeCall::PolkadotCollectives(cc) => polkadot_collectives_batch.push(cc),
		}
	}
	if kusama_relay_batch.len() > 0 {
		let batch = KusamaRuntimeCall::Utility(KusamaUtilityCall::force_batch {
			calls: kusama_relay_batch,
		});
		println!("\nBatch to submit on Kusama Relay Chain:");
		print_output(output, &NetworkRuntimeCall::Kusama(batch));
	}
	if polkadot_relay_batch.len() > 0 {
		let batch = PolkadotRuntimeCall::Utility(PolkadotRelayUtilityCall::force_batch {
			calls: polkadot_relay_batch,
		});
		println!("\nBatch to submit on Polkadot Relay Chain:");
		print_output(output, &NetworkRuntimeCall::Polkadot(batch));
	}
	if polkadot_collectives_batch.len() > 0 {
		let batch = CollectivesRuntimeCall::Utility(CollectivesUtilityCall::force_batch {
			calls: polkadot_collectives_batch,
		});
		println!("\nBatch to submit on Polkadot Collectives Chain:");
		print_output(output, &NetworkRuntimeCall::PolkadotCollectives(batch));
	}
}

// Check what the user entered for the proposal. If it is just call data, return it back. Otherwise,
// we expect a path to a file that contains the call data. Read that in and return it.
fn get_proposal_bytes(proposal: &'static str) -> Vec<u8> {
	if proposal.starts_with("0x") {
		// This is just call data
		return hex::decode(proposal.trim_start_matches("0x")).expect("Valid proposal")
	} else {
		// This is a file path
		let contents = fs::read_to_string(proposal).expect("Should give a valid file path");
		return hex::decode(contents.as_str().trim_start_matches("0x")).expect("Valid proposal")
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
		},
		NetworkRuntimeCall::Polkadot(call) => {
			let rpc: &'static str = "wss%3A%2F%2Frpc.polkadot.io";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://polkadot.js.org/apps/?rpc={}#/extrinsics/decode/0x{}",
					rpc,
					hex::encode(call.encode())
				),
			}
		},
		NetworkRuntimeCall::PolkadotCollectives(call) => {
			let rpc: &'static str = "wss%3A%2F%2Fpolkadot-collectives-rpc.polkadot.io";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://polkadot.js.org/apps/?rpc={}#/extrinsics/decode/0x{}",
					rpc,
					hex::encode(call.encode())
				),
			}
		},
	}
}
