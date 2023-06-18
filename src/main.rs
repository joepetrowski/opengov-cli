use parity_scale_codec::Encode as _;
use std::fs;
use subxt::ext::sp_core;

#[subxt::subxt(
	runtime_metadata_url = "wss://kusama-rpc.polkadot.io:443",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod kusama {}
use kusama::runtime_types::kusama_runtime::{
	governance::origins::pallet_custom_origins::Origin as KusamaOpenGovOrigin,
	RuntimeCall as KusamaRuntimeCall,
};

#[subxt::subxt(
	runtime_metadata_url = "wss://rpc.polkadot.io:443",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod polkadot_relay {}
use polkadot_relay::runtime_types::polkadot_runtime::{
	governance::origins::pallet_custom_origins::Origin as PolkadotOpenGovOrigin,
	RuntimeCall as PolkadotRuntimeCall,
};

#[subxt::subxt(runtime_metadata_url = "wss://polkadot-collectives-rpc.polkadot.io:443")]
pub mod polkadot_collectives {}
use polkadot_collectives::runtime_types::collectives_polkadot_runtime::{
	fellowship::origins::pallet_origins::Origin as FellowshipOrigins,
	RuntimeCall as CollectivesRuntimeCall,
};

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
	}
}

// Info and preferences provided by the user.
struct ProposalDetails {
	// The proposal, generated elsewhere and pasted here.
	proposal: &'static str,
	// The track to submit on.
	track: NetworkTrack,
	// When do you want this to enact. `At(block)` or `After(blocks)`.
	dispatch: DispatchTimeWrapper,
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
enum Network {
	Kusama,
	Polkadot,
	PolkadotCollectives,
}

#[allow(dead_code)]
enum NetworkTrack {
	Kusama(KusamaOpenGovOrigin),
	Polkadot(PolkadotOpenGovOrigin),
}

#[allow(dead_code)]
enum NetworkRuntimeCall {
	Kusama(KusamaRuntimeCall),
	Polkadot(PolkadotRuntimeCall),
	PolkadotCollectives(CollectivesRuntimeCall),
}

#[allow(dead_code)]
enum Output {
	// Print just the call data (e.g. 0x1234).
	CallData,
	// Print a clickable link to view the decoded call on Polkadot JS Apps UI.
	AppsUiLink,
}

// Local concrete type to use in each runtime's `DispatchTime`
#[allow(dead_code)]
enum DispatchTimeWrapper {
	At(u32),
	After(u32),
}

enum CallOrHash {
	Call(NetworkRuntimeCall),
	Hash([u8; 32]),
}

// All the info associated with a call in the forms you may need it in.
struct CallInfo {
	call: NetworkRuntimeCall,
	encoded: Vec<u8>,
	hash: [u8; 32],
	length: u32,
}

impl CallInfo {
	// Construct `Self` from a `NetworkRuntimeCall`.
	fn from_runtime_call(call: NetworkRuntimeCall) -> Self {
		let encoded = match &call {
			NetworkRuntimeCall::Kusama(cc) => cc.encode(),
			NetworkRuntimeCall::Polkadot(cc) => cc.encode(),
			NetworkRuntimeCall::PolkadotCollectives(cc) => cc.encode(),
		};
		let hash = sp_core::blake2_256(&encoded);
		let length: u32 = (*&encoded.len()).try_into().unwrap();
		Self { call, encoded: encoded.to_vec(), hash, length }
	}

	// Construct `Self` for some `network` given some `encoded` bytes.
	fn from_bytes(encoded: &Vec<u8>, network: Network) -> Self {
		let call = match network {
			Network::Kusama => {
				let runtime_call =
					<KusamaRuntimeCall as parity_scale_codec::Decode>::decode(&mut &encoded[..])
						.unwrap();
				NetworkRuntimeCall::Kusama(runtime_call)
			},
			Network::Polkadot => {
				let runtime_call =
					<PolkadotRuntimeCall as parity_scale_codec::Decode>::decode(&mut &encoded[..])
						.unwrap();
				NetworkRuntimeCall::Polkadot(runtime_call)
			},
			Network::PolkadotCollectives => {
				let runtime_call = <CollectivesRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &encoded[..],
				)
				.unwrap();
				NetworkRuntimeCall::PolkadotCollectives(runtime_call)
			},
		};
		let hash = sp_core::blake2_256(&encoded);
		let length = (*&encoded.len()).try_into().unwrap();
		Self { call, encoded: encoded.to_vec(), hash, length }
	}

	// Strip the outer enum and return a Kusama Relay `RuntimeCall`.
	fn get_kusama_call(&self) -> Result<KusamaRuntimeCall, &'static str> {
		match &self.call {
			NetworkRuntimeCall::Kusama(_) => {
				let bytes = &self.encoded;
				Ok(<KusamaRuntimeCall as parity_scale_codec::Decode>::decode(&mut &bytes[..])
					.unwrap())
			},
			_ => return Err("not a kusama call"),
		}
	}

	// Strip the outer enum and return a Polkadot Relay `RuntimeCall`.
	fn get_polkadot_call(&self) -> Result<PolkadotRuntimeCall, &'static str> {
		match &self.call {
			NetworkRuntimeCall::Polkadot(_) => {
				let bytes = &self.encoded;
				Ok(<PolkadotRuntimeCall as parity_scale_codec::Decode>::decode(&mut &bytes[..])
					.unwrap())
			},
			_ => return Err("not a polkadot call"),
		}
	}

	// Strip the outer enum and return a Polkadot Collectives `RuntimeCall`.
	fn get_polkadot_collectives_call(&self) -> Result<CollectivesRuntimeCall, &'static str> {
		match &self.call {
			NetworkRuntimeCall::PolkadotCollectives(_) => {
				let bytes = &self.encoded;
				Ok(<CollectivesRuntimeCall as parity_scale_codec::Decode>::decode(&mut &bytes[..])
					.unwrap())
			},
			_ => return Err("not a polkadot collectives call"),
		}
	}

	// Take `Self` and a length limit as input. If the call length exceeds the limit, just return
	// its hash. Call length is recomputed and will be 2 bytes longer than the actual preimage
	// length. This is because the call is `preimage.note_preimage(call)`, so the outer pallet/call
	// indices have a length of 2 bytes.
	fn create_print_output(&self, length_limit: u32) -> (CallOrHash, u32) {
		let print_output: CallOrHash;
		if self.length > length_limit {
			print_output = CallOrHash::Hash(self.hash);
		} else {
			print_output = match &self.call {
				NetworkRuntimeCall::Kusama(_) => {
					let kusama_call = self.get_kusama_call().expect("kusama");
					CallOrHash::Call(NetworkRuntimeCall::Kusama(kusama_call))
				},
				NetworkRuntimeCall::Polkadot(_) => {
					let polkadot_call = self.get_polkadot_call().expect("polkadot");
					CallOrHash::Call(NetworkRuntimeCall::Polkadot(polkadot_call))
				},
				NetworkRuntimeCall::PolkadotCollectives(_) => {
					let collectives_call =
						self.get_polkadot_collectives_call().expect("collectives");
					CallOrHash::Call(NetworkRuntimeCall::PolkadotCollectives(collectives_call))
				},
			};
		}
		(print_output, self.length)
	}
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
	let calls = generate_calls(&proposal_details);

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

fn generate_calls(proposal_details: &ProposalDetails) -> PossibleCallsToSubmit {
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
				sp_weights::weight_v2::Weight,
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
										require_weight_at_most: Weight {
											// todo
											ref_time: 1_000_000_000,
											// We don't really care about proof size on the Relay Chain.
											// Make it big so that it will definitely work.
											proof_size: 1_000_000,
										},
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
