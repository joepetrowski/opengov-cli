use crate::*;
use clap::Parser as ClapParser;
use std::fs;

/// Generate all the calls needed to submit a proposal as a referendum in OpenGov.
#[derive(Debug, ClapParser)]
pub(crate) struct ReferendumArgs {
	/// The encoded proposal that we want to submit. This can either be the call data itself,
	/// e.g. "0x0102...", or a file path that contains the data, e.g. "./my_proposal.call".
	#[clap(long = "proposal", short)]
	proposal: String,

	/// Network on which to submit the referendum. `polkadot` or `kusama`.
	#[clap(long = "network", short)]
	network: String,

	/// Track on which to submit the referendum.
	#[clap(long = "track", short)]
	track: String,

	/// Optional: Enact at a particular block number.
	#[clap(long = "at")]
	at: Option<u32>,

	/// Optional: Enact after a given number of blocks.
	#[clap(long = "after")]
	after: Option<u32>,

	/// Output length limit. Defaults to 1,000.
	#[clap(long = "output-len-limit")]
	output_len_limit: Option<u32>,

	/// Do not print batch calls. Defaults to false.
	#[clap(long = "no-batch")]
	no_batch: bool,

	/// Form of output. `AppsUiLink` or `CallData`. Defaults to Apps UI.
	#[clap(long = "output")]
	output: Option<String>,
}

// The sub-command's "main" function.
pub(crate) async fn submit_referendum(prefs: ReferendumArgs) {
	// Find out what the user wants to do.
	let proposal_details = parse_inputs(prefs);
	// Generate the calls necessary.
	let calls = generate_calls(&proposal_details).await;
	// Tell the user what to do.
	deliver_output(proposal_details, calls);
}

// Parse the CLI inputs and return a typed struct with all the details needed.
fn parse_inputs(prefs: ReferendumArgs) -> ProposalDetails {
	use DispatchTimeWrapper::*;
	use NetworkTrack::*;
	use Output::*;

	let proposal = prefs.proposal;

	let track = match prefs.network.to_ascii_lowercase().as_str() {
		"polkadot" => match prefs.track.to_ascii_lowercase().as_str() {
			"root" => PolkadotRoot,
			"whitelisted-caller" | "whitelistedcaller" =>
				Polkadot(PolkadotOpenGovOrigin::WhitelistedCaller),
			"staking-admin" | "stakingadmin" => Polkadot(PolkadotOpenGovOrigin::StakingAdmin),
			"treasurer" => Polkadot(PolkadotOpenGovOrigin::Treasurer),
			"lease-admin" | "leaseadmin" => Polkadot(PolkadotOpenGovOrigin::LeaseAdmin),
			"fellowship-admin" | "fellowshipadmin" =>
				Polkadot(PolkadotOpenGovOrigin::FellowshipAdmin),
			"general-admin" | "generaladmin" => Polkadot(PolkadotOpenGovOrigin::GeneralAdmin),
			"auction-admin" | "auctionadmin" => Polkadot(PolkadotOpenGovOrigin::AuctionAdmin),
			"referendum-killer" | "referendumkiller" =>
				Polkadot(PolkadotOpenGovOrigin::ReferendumKiller),
			"referendum-canceller" | "referendumcanceller" =>
				Polkadot(PolkadotOpenGovOrigin::ReferendumCanceller),
			_ => panic!("Unsupported track! Tracks should be in the form `general-admin` or `generaladmin`."),
		},
		"kusama" => match prefs.track.to_ascii_lowercase().as_str() {
			"root" => KusamaRoot,
			"whitelisted-caller" | "whitelistedcaller" =>
				Kusama(KusamaOpenGovOrigin::WhitelistedCaller),
			"staking-admin" | "stakingadmin" => Kusama(KusamaOpenGovOrigin::StakingAdmin),
			"treasurer" => Kusama(KusamaOpenGovOrigin::Treasurer),
			"lease-admin" | "leaseadmin" => Kusama(KusamaOpenGovOrigin::LeaseAdmin),
			"fellowship-admin" | "fellowshipadmin" => Kusama(KusamaOpenGovOrigin::FellowshipAdmin),
			"general-admin" | "generaladmin" => Kusama(KusamaOpenGovOrigin::GeneralAdmin),
			"auction-admin" | "auctionadmin" => Kusama(KusamaOpenGovOrigin::AuctionAdmin),
			"referendum-killer" | "referendumkiller" =>
				Kusama(KusamaOpenGovOrigin::ReferendumKiller),
			"referendum-canceller" | "referendumcanceller" =>
				Kusama(KusamaOpenGovOrigin::ReferendumCanceller),
			_ => panic!("Unsupported track! Tracks should be in the form `general-admin` or `generaladmin`."),
		},
		_ => panic!("`network` must be `polkadot` or `kusama`"),
	};

	let dispatch = match (prefs.at, prefs.after) {
		(None, None) => {
			println!("\nNo enactment time specified. Defaulting to `After(10)`.");
			println!("Specify an enactment time with `--at <block>` or `--after <blocks>`.\n");
			After(10)
		},
		(Some(_), Some(_)) => {
			panic!("\nBoth `At` and `After` dispatch times provided. You can only use one.\n");
		},
		(Some(at), None) => At(at),
		(None, Some(after)) => After(after),
	};

	let output_len_limit = if let Some(input) = prefs.output_len_limit { input } else { 1_000 };

	let print_batch = !prefs.no_batch;

	let output = if let Some(input) = prefs.output {
		match input.to_ascii_lowercase().as_str() {
			"calldata" | "call-data" => CallData,
			"appsuilink" | "apps-ui-link" => AppsUiLink,
			_ => panic!("`output` must be `calldata` or `appsuilink`. If not specified, the default is `appsuilink`."),
		}
	} else {
		AppsUiLink
	};

	ProposalDetails {
		proposal,
		track,
		dispatch,
		output,
		output_len_limit,
		print_batch,
		transact_weight_override: None,
	}
}

// Generate all the calls needed.
pub(crate) async fn generate_calls(proposal_details: &ProposalDetails) -> PossibleCallsToSubmit {
	match &proposal_details.track {
		// Kusama Root Origin. Since the Root origin is not part of `OpenGovOrigin`, we match it
		// specially.
		NetworkTrack::KusamaRoot => {
			use kusama_relay::runtime_types::frame_support::dispatch::RawOrigin;
			kusama_non_fellowship_referenda(
				proposal_details,
				KusamaOriginCaller::system(RawOrigin::Root),
			)
		},

		// All special Kusama origins.
		NetworkTrack::Kusama(kusama_track) => {
			match kusama_track {
				// Whitelisted calls are special.
				KusamaOpenGovOrigin::WhitelistedCaller =>
					kusama_fellowship_referenda(proposal_details),

				// All other Kusama origins.
				_ => kusama_non_fellowship_referenda(
					proposal_details,
					KusamaOriginCaller::Origins(kusama_track.clone()),
				),
			}
		},

		// Same for Polkadot Root origin. It is not part of OpenGovOrigins, so it gets its own arm.
		NetworkTrack::PolkadotRoot => {
			use polkadot_relay::runtime_types::frame_support::dispatch::RawOrigin;
			polkadot_non_fellowship_referenda(
				proposal_details,
				PolkadotOriginCaller::system(RawOrigin::Root),
			)
		},

		// All special Polkadot origins.
		NetworkTrack::Polkadot(polkadot_track) => {
			match polkadot_track {
				PolkadotOpenGovOrigin::WhitelistedCaller =>
					polkadot_fellowship_referenda(proposal_details).await,

				// All other Polkadot origins.
				_ => polkadot_non_fellowship_referenda(
					proposal_details,
					PolkadotOriginCaller::Origins(polkadot_track.clone()),
				),
			}
		},
	}
}

// Generate the calls needed for a proposal to pass through the Kusama Fellowship.
fn kusama_fellowship_referenda(proposal_details: &ProposalDetails) -> PossibleCallsToSubmit {
	use kusama_relay::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
		pallet_whitelist::pallet::Call as WhitelistCall,
	};
	// First we need to whitelist this proposal. We will need:
	//   1. To wrap the proposal hash in `whitelist.whitelist_call()` and submit this as a preimage.
	//   2. To submit a referendum to the Fellowship Referenda pallet to dispatch this preimage.
	let proposal_bytes = get_proposal_bytes(proposal_details.proposal.clone());
	let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Kusama);

	let public_referendum_dispatch_time = match proposal_details.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};

	let whitelist_call =
		CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(KusamaRuntimeCall::Whitelist(
			WhitelistCall::whitelist_call { call_hash: H256(proposal_call_info.hash) },
		)));
	let preimage_for_whitelist_call = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::Preimage(PreimageCall::note_preimage { bytes: whitelist_call.encoded }),
	));

	let fellowship_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::FellowshipReferenda(ReferendaCall::submit {
			proposal_origin: Box::new(KusamaOriginCaller::Origins(KusamaOpenGovOrigin::Fellows)),
			proposal: Lookup { hash: H256(whitelist_call.hash), len: whitelist_call.length },
			enactment_moment: DispatchTime::After(10),
		}),
	));

	// Now we put together the public referendum part. This still needs separate logic because the
	// actual proposal gets wrapped in a Whitelist call.
	let dispatch_whitelisted_call = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::Whitelist(WhitelistCall::dispatch_whitelisted_call_with_preimage {
			call: Box::new(proposal_call_info.get_kusama_call().expect("kusama")),
		}),
	));

	let preimage_for_dispatch_whitelisted_call =
		CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(KusamaRuntimeCall::Preimage(
			PreimageCall::note_preimage { bytes: dispatch_whitelisted_call.encoded.clone() },
		)));
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(KusamaOriginCaller::Origins(
				KusamaOpenGovOrigin::WhitelistedCaller,
			)),
			proposal: Lookup {
				hash: H256(dispatch_whitelisted_call.hash),
				len: dispatch_whitelisted_call.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));

	// Check the lengths and prepare preimages for printing.
	let (whitelist_preimage_print, whitelist_preimage_print_len) =
		preimage_for_whitelist_call.create_print_output(proposal_details.output_len_limit);
	let (dispatch_preimage_print, dispatch_preimage_print_len) =
		preimage_for_dispatch_whitelisted_call
			.create_print_output(proposal_details.output_len_limit);

	// If it's a hash, let's write the data to a file you can upload.
	match dispatch_preimage_print {
		CallOrHash::Call(_) => (),
		CallOrHash::Hash(_) => {
			let mut info_to_write = "0x".to_owned();
			info_to_write.push_str(hex::encode(dispatch_whitelisted_call.encoded).as_str());
			fs::write("kusama_relay_public_referendum_preimage_to_note.call", info_to_write)
				.expect("it should write");
		},
	}

	PossibleCallsToSubmit {
		preimage_for_whitelist_call: Some((whitelist_preimage_print, whitelist_preimage_print_len)),
		preimage_for_public_referendum: Some((
			dispatch_preimage_print,
			dispatch_preimage_print_len,
		)),
		fellowship_referendum_submission: Some(NetworkRuntimeCall::Kusama(
			fellowship_proposal.get_kusama_call().expect("kusama"),
		)),
		public_referendum_submission: Some(NetworkRuntimeCall::Kusama(
			public_proposal.get_kusama_call().expect("kusama"),
		)),
	}
}

// Generate the calls needed for a proposal to pass on Kusama without the Fellowship.
fn kusama_non_fellowship_referenda(
	proposal_details: &ProposalDetails,
	origin: KusamaOriginCaller,
) -> PossibleCallsToSubmit {
	use kusama_relay::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
	};

	let proposal_bytes = get_proposal_bytes(proposal_details.proposal.clone());
	let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Kusama);

	let public_referendum_dispatch_time = match proposal_details.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};

	let note_proposal_preimage = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::Preimage(PreimageCall::note_preimage { bytes: proposal_bytes }),
	));
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(origin),
			proposal: Lookup {
				hash: H256(proposal_call_info.hash),
				len: proposal_call_info.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));
	let (preimage_print, preimage_print_len) =
		note_proposal_preimage.create_print_output(proposal_details.output_len_limit);

	PossibleCallsToSubmit {
		preimage_for_whitelist_call: None,
		preimage_for_public_referendum: Some((preimage_print, preimage_print_len)),
		fellowship_referendum_submission: None,
		public_referendum_submission: Some(NetworkRuntimeCall::Kusama(
			public_proposal.get_kusama_call().expect("kusama"),
		)),
	}
}

// Generate the calls needed for a proposal to pass through the Polkadot Fellowship.
async fn polkadot_fellowship_referenda(
	proposal_details: &ProposalDetails,
) -> PossibleCallsToSubmit {
	use polkadot_collectives::runtime_types::{
		collectives_polkadot_runtime::OriginCaller as CollectivesOriginCaller,
		frame_support::traits::{
			preimages::Bounded::Lookup as CollectivesLookup,
			schedule::DispatchTime as CollectivesDispatchTime,
		},
		// Since the Relay Chain and Collectives chains may be on different versions of Preimage,
		// Referenda, and XCM pallets, we need to define their `Call` enum separately.
		pallet_preimage::pallet::Call as CollectivesPreimageCall,
		pallet_referenda::pallet::Call as CollectivesReferendaCall,
		pallet_xcm::pallet::Call as CollectivesXcmCall,
		staging_xcm::v4::{junctions::Junctions::Here, location::Location, Instruction, Xcm},
		xcm::{
			double_encoded::DoubleEncoded, v3::OriginKind, v3::WeightLimit, VersionedLocation,
			VersionedXcm::V4,
		},
	};
	use polkadot_relay::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
		pallet_whitelist::pallet::Call as WhitelistCall,
	};
	// Fellowship is on the Collectives parachain, so things are a bit different here.
	//
	// 1. Create a whitelist call on the Relay Chain:
	//
	//    let whitelist_call =
	//     	  PolkadotRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
	// 		      call_hash: H256(proposal_hash),
	// 	      });
	//
	// 2. Create an XCM send call on the Collectives chain to Transact this on the Relay Chain:
	//
	//    let send_whitelist = CollectivesRuntimeCall::PolkadotXcm(
	//        PolkadotXcmCall::send {
	// 	          dest: Location { parents: 1, interior: Here },
	// 	          message: vec![UnpaidExecution, Transact {call: whitelist_call, ..}],
	//        }
	//    );
	//
	// 3. Make a Fellowship referendum for `send_whitelist`.
	//
	// 4. Relay Chain public referendum should be the same as on Kusama.
	let proposal_bytes = get_proposal_bytes(proposal_details.proposal.clone());
	let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Polkadot);

	let public_referendum_dispatch_time = match proposal_details.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};
	// Whitelist the call on the Relay Chain.
	let whitelist_call =
		CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(PolkadotRuntimeCall::Whitelist(
			WhitelistCall::whitelist_call { call_hash: H256(proposal_call_info.hash) },
		)));

	let (ref_time, proof_size) =
		// The user may want to override the computed values, e.g. for deterministic
		// testing.
		if let Some(weight_override) = &proposal_details.transact_weight_override {
			(weight_override.ref_time, weight_override.proof_size)
		} else {
			// Do some weight calculation for execution of Transact on the Relay Chain.
			let max_ref_time: u64 = 2_000_000_000_000 - 1;
			let max_proof_size: u64 = 5 * 1024 * 1024 - 1;
			let relay_weight_needed = whitelist_call.get_transact_weight_needed(
				&Network::Polkadot,
				Weight { ref_time: 1_000_000_000, proof_size: 10_000 }
			).await;
			// Double the weight needed, just to be safe from a runtime upgrade that could change
			// things during the referendum period.
			(
				(2 * relay_weight_needed.ref_time).min(max_ref_time),
				(2 * relay_weight_needed.proof_size).min(max_proof_size),
			)
		};

	// This is what the Fellowship will actually vote on enacting.
	let whitelist_over_xcm = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
		CollectivesRuntimeCall::PolkadotXcm(CollectivesXcmCall::send {
			dest: Box::new(VersionedLocation::V4(Location { parents: 1, interior: Here })),
			message: Box::new(V4(Xcm(vec![
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
			CollectivesPreimageCall::note_preimage { bytes: whitelist_over_xcm.encoded },
		)),
	);

	// The actual Fellowship referendum submission.
	let fellowship_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
		CollectivesRuntimeCall::FellowshipReferenda(CollectivesReferendaCall::submit {
			proposal_origin: Box::new(CollectivesOriginCaller::FellowshipOrigins(
				FellowshipOrigins::Fellows,
			)),
			proposal: CollectivesLookup {
				hash: H256(whitelist_over_xcm.hash),
				len: whitelist_over_xcm.length,
			},
			enactment_moment: CollectivesDispatchTime::After(10u32),
		}),
	));

	// Now we put together the public referendum part. This still needs separate logic because the
	// actual proposal gets wrapped in a Whitelist call.
	let dispatch_whitelisted_call = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
		PolkadotRuntimeCall::Whitelist(WhitelistCall::dispatch_whitelisted_call_with_preimage {
			call: Box::new(proposal_call_info.get_polkadot_call().expect("it is a polkadot call")),
		}),
	));

	let preimage_for_dispatch_whitelisted_call =
		CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(PolkadotRuntimeCall::Preimage(
			PreimageCall::note_preimage { bytes: dispatch_whitelisted_call.encoded.clone() },
		)));
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
		PolkadotRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(PolkadotOriginCaller::Origins(
				PolkadotOpenGovOrigin::WhitelistedCaller,
			)),
			proposal: Lookup {
				hash: H256(dispatch_whitelisted_call.hash),
				len: dispatch_whitelisted_call.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));

	// Check the lengths and prepare preimages for printing.
	let (whitelist_over_xcm_preimage_print, whitelist_over_xcm_preimage_print_len) =
		preimage_for_whitelist_over_xcm.create_print_output(proposal_details.output_len_limit);
	let (dispatch_preimage_print, dispatch_preimage_print_len) =
		preimage_for_dispatch_whitelisted_call
			.create_print_output(proposal_details.output_len_limit);

	// If it's a hash, let's write the data to a file you can upload.
	match dispatch_preimage_print {
		CallOrHash::Call(_) => (),
		CallOrHash::Hash(_) => {
			let mut info_to_write = "0x".to_owned();
			info_to_write.push_str(hex::encode(dispatch_whitelisted_call.encoded).as_str());
			fs::write("polkadot_relay_public_referendum_preimage_to_note.call", info_to_write)
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
		fellowship_referendum_submission: Some(NetworkRuntimeCall::PolkadotCollectives(
			fellowship_proposal.get_polkadot_collectives_call().expect("polkadot collectives"),
		)),
		public_referendum_submission: Some(NetworkRuntimeCall::Polkadot(
			public_proposal.get_polkadot_call().expect("polkadot"),
		)),
	}
}

// Generate the calls needed for a proposal to pass on Polkadot without the Fellowship.
fn polkadot_non_fellowship_referenda(
	proposal_details: &ProposalDetails,
	origin: PolkadotOriginCaller,
) -> PossibleCallsToSubmit {
	use polkadot_relay::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
	};

	let proposal_bytes = get_proposal_bytes(proposal_details.proposal.clone());
	let proposal_call_info = CallInfo::from_bytes(&proposal_bytes, Network::Polkadot);

	let public_referendum_dispatch_time = match proposal_details.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};

	let note_proposal_preimage = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
		PolkadotRuntimeCall::Preimage(PreimageCall::note_preimage { bytes: proposal_bytes }),
	));
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Polkadot(
		PolkadotRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(origin),
			proposal: Lookup {
				hash: H256(proposal_call_info.hash),
				len: proposal_call_info.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));
	let (preimage_print, preimage_print_len) =
		note_proposal_preimage.create_print_output(proposal_details.output_len_limit);

	PossibleCallsToSubmit {
		preimage_for_whitelist_call: None,
		preimage_for_public_referendum: Some((preimage_print, preimage_print_len)),
		fellowship_referendum_submission: None,
		public_referendum_submission: Some(NetworkRuntimeCall::Polkadot(
			public_proposal.get_polkadot_call().expect("polkadot"),
		)),
	}
}

// Takes all the `calls` needed to submit and logs them according to the user's preferences.
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

// Takes a vec of calls, which could be intended for use on different networks, sorts them into the
// appropriate network, and provides a single batch call for each network.
fn handle_batch_of_calls(output: &Output, batch: Vec<NetworkRuntimeCall>) {
	use kusama_relay::runtime_types::pallet_utility::pallet::Call as KusamaUtilityCall;
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
			_ => panic!("no other chains are needed for this"),
		}
	}
	if !kusama_relay_batch.is_empty() {
		let batch = KusamaRuntimeCall::Utility(KusamaUtilityCall::force_batch {
			calls: kusama_relay_batch,
		});
		println!("\nBatch to submit on Kusama Relay Chain:");
		print_output(output, &NetworkRuntimeCall::Kusama(batch));
	}
	if !polkadot_relay_batch.is_empty() {
		let batch = PolkadotRuntimeCall::Utility(PolkadotRelayUtilityCall::force_batch {
			calls: polkadot_relay_batch,
		});
		println!("\nBatch to submit on Polkadot Relay Chain:");
		print_output(output, &NetworkRuntimeCall::Polkadot(batch));
	}
	if !polkadot_collectives_batch.is_empty() {
		let batch = CollectivesRuntimeCall::Utility(CollectivesUtilityCall::force_batch {
			calls: polkadot_collectives_batch,
		});
		println!("\nBatch to submit on Polkadot Collectives Chain:");
		print_output(output, &NetworkRuntimeCall::PolkadotCollectives(batch));
	}
}

// Format the data to print to console.
fn print_output(output: &Output, network_call: &NetworkRuntimeCall) {
	match network_call {
		NetworkRuntimeCall::Kusama(call) => {
			let rpc: &'static str = "wss%3A%2F%2Fkusama-rpc.dwellir.com";
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
			let rpc: &'static str = "wss%3A%2F%2Fpolkadot-rpc.dwellir.com";
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
		_ => panic!("no other chains are needed for this"),
	}
}
