//! Build the calls needed to put a proposal to referendum in OpenGov.

use crate::*;

/// A proposal and how it should be put to referendum.
#[derive(Clone)]
pub struct ReferendumInput {
	/// The encoded proposal: an Asset Hub call.
	pub proposal: Vec<u8>,
	/// The network and track to submit the referendum on.
	pub track: NetworkTrack,
	/// When the proposal should enact once approved.
	pub dispatch: DispatchTimeWrapper,
	/// Use the Polkadot Fellowship (on Collectives) instead of the Kusama Fellowship to whitelist
	/// the proposal. Only valid for the Kusama `WhitelistedCaller` track.
	pub fellowship_on_polkadot: bool,
}

/// Everything that needs to be submitted to start a referendum. The calls can be submitted in any
/// order, but the preimages must be on chain before their referendum can enact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferendumCalls {
	/// `preimage.note_preimage` for the Fellowship referendum's proposal. Only `Some` when that
	/// proposal is too long to be inlined in the referendum itself.
	pub fellowship_preimage: Option<CallInfo>,
	/// `fellowshipReferenda.submit`, the Fellowship referendum to whitelist the proposal. Only
	/// `Some` on a `WhitelistedCaller` track.
	pub fellowship_referendum: Option<CallInfo>,
	/// `preimage.note_preimage` for the public referendum's proposal.
	pub public_preimage: CallInfo,
	/// `referenda.submit`, the public referendum.
	pub public_referendum: CallInfo,
	/// The bytes that `public_preimage` notes: the proposal itself, or the call to dispatch it via
	/// the whitelist.
	pub public_preimage_bytes: Vec<u8>,
	/// Hash of `public_preimage_bytes`, which the public referendum refers to.
	pub public_preimage_hash: [u8; 32],
	/// Length of `public_preimage_bytes`, which the public referendum refers to.
	pub public_preimage_len: u32,
	/// The hash the Fellowship whitelists (the hash of the proposal), when a whitelist is involved.
	pub whitelisted_call_hash: Option<[u8; 32]>,
}

impl NetworkTrack {
	/// Parse a network (`polkadot` or `kusama`) and a track name. Track names are accepted in the
	/// forms `general-admin` and `generaladmin`, in any case.
	pub fn parse(network: &str, track: &str) -> Result<Self, Error> {
		use NetworkTrack::*;
		match network.to_ascii_lowercase().as_str() {
			"polkadot" => match track.to_ascii_lowercase().as_str() {
				"root" => Ok(PolkadotRoot),
				"whitelisted-caller" | "whitelistedcaller" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::WhitelistedCaller)),
				"staking-admin" | "stakingadmin" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::StakingAdmin)),
				"treasurer" => Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::Treasurer)),
				"lease-admin" | "leaseadmin" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::LeaseAdmin)),
				"fellowship-admin" | "fellowshipadmin" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::FellowshipAdmin)),
				"general-admin" | "generaladmin" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::GeneralAdmin)),
				"auction-admin" | "auctionadmin" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::AuctionAdmin)),
				"referendum-killer" | "referendumkiller" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::ReferendumKiller)),
				"referendum-canceller" | "referendumcanceller" =>
					Ok(Polkadot(PolkadotAssetHubOpenGovOrigin::ReferendumCanceller)),
				_ => Err(Error::UnsupportedTrack),
			},
			"kusama" => match track.to_ascii_lowercase().as_str() {
				"root" => Ok(KusamaRoot),
				"whitelisted-caller" | "whitelistedcaller" =>
					Ok(Kusama(KusamaAssetHubOpenGovOrigin::WhitelistedCaller)),
				"staking-admin" | "stakingadmin" =>
					Ok(Kusama(KusamaAssetHubOpenGovOrigin::StakingAdmin)),
				"treasurer" => Ok(Kusama(KusamaAssetHubOpenGovOrigin::Treasurer)),
				"lease-admin" | "leaseadmin" => Ok(Kusama(KusamaAssetHubOpenGovOrigin::LeaseAdmin)),
				"fellowship-admin" | "fellowshipadmin" =>
					Ok(Kusama(KusamaAssetHubOpenGovOrigin::FellowshipAdmin)),
				"general-admin" | "generaladmin" =>
					Ok(Kusama(KusamaAssetHubOpenGovOrigin::GeneralAdmin)),
				"auction-admin" | "auctionadmin" =>
					Ok(Kusama(KusamaAssetHubOpenGovOrigin::AuctionAdmin)),
				"referendum-killer" | "referendumkiller" =>
					Ok(Kusama(KusamaAssetHubOpenGovOrigin::ReferendumKiller)),
				"referendum-canceller" | "referendumcanceller" =>
					Ok(Kusama(KusamaAssetHubOpenGovOrigin::ReferendumCanceller)),
				_ => Err(Error::UnsupportedTrack),
			},
			_ => Err(Error::NotARelayChain),
		}
	}

	/// Check whether the Polkadot Fellowship can be asked to whitelist a proposal for this track
	/// when `fellowship_on_polkadot` is set. It is only an option (as opposed to the default) for
	/// the Kusama `WhitelistedCaller` track.
	pub fn check_fellowship(&self, fellowship_on_polkadot: bool) -> Result<(), Error> {
		let is_kusama_whitelisted =
			matches!(self, NetworkTrack::Kusama(KusamaAssetHubOpenGovOrigin::WhitelistedCaller));
		if fellowship_on_polkadot && !is_kusama_whitelisted {
			return Err(Error::FellowshipNotSupported);
		}
		Ok(())
	}
}

impl DispatchTimeWrapper {
	/// Interpret optional `at` and `after` enactment times. Returns `None` when neither is given,
	/// and an error when both are.
	pub fn from_at_after(at: Option<u32>, after: Option<u32>) -> Result<Option<Self>, Error> {
		match (at, after) {
			(None, None) => Ok(None),
			(Some(_), Some(_)) => Err(Error::BothAtAndAfter),
			(Some(at), None) => Ok(Some(DispatchTimeWrapper::At(at))),
			(None, Some(after)) => Ok(Some(DispatchTimeWrapper::After(after))),
		}
	}
}

/// Generate all the calls needed to put `input.proposal` to referendum.
pub fn referendum_calls(input: ReferendumInput) -> Result<ReferendumCalls, Error> {
	input.track.check_fellowship(input.fellowship_on_polkadot)?;
	// A proposal is only decoded when it has to be wrapped in a whitelist call, so on other tracks
	// this is the one check on its content. No call encodes to nothing.
	if input.proposal.is_empty() {
		return Err(Error::EmptyProposal);
	}
	match &input.track {
		// Kusama Root Origin. Since the Root origin is not part of `OpenGovOrigin`, we match it
		// specially.
		NetworkTrack::KusamaRoot => {
			use kusama_asset_hub::runtime_types::frame_support::dispatch::RawOrigin;
			Ok(kusama_non_fellowship_referenda(
				&input,
				KusamaAssetHubOriginCaller::system(RawOrigin::Root),
			))
		},

		// All special Kusama origins.
		NetworkTrack::Kusama(kusama_track) => {
			match kusama_track {
				// Whitelisted calls are special.
				KusamaAssetHubOpenGovOrigin::WhitelistedCaller =>
					if input.fellowship_on_polkadot {
						polkadot_fellowship_for_kusama_referenda(&input)
					} else {
						kusama_fellowship_referenda(&input)
					},

				// All other Kusama origins.
				_ => Ok(kusama_non_fellowship_referenda(
					&input,
					KusamaAssetHubOriginCaller::Origins(kusama_track.clone()),
				)),
			}
		},

		// Same for Polkadot Root origin. It is not part of OpenGovOrigins, so it gets its own arm.
		NetworkTrack::PolkadotRoot => {
			use polkadot_asset_hub::runtime_types::frame_support::dispatch::RawOrigin;
			Ok(polkadot_non_fellowship_referenda(
				&input,
				PolkadotAssetHubOriginCaller::system(RawOrigin::Root),
			))
		},

		// All special Polkadot origins.
		NetworkTrack::Polkadot(polkadot_track) => {
			match polkadot_track {
				PolkadotAssetHubOpenGovOrigin::WhitelistedCaller =>
					polkadot_fellowship_referenda(&input),

				// All other Polkadot origins.
				_ => Ok(polkadot_non_fellowship_referenda(
					&input,
					PolkadotAssetHubOriginCaller::Origins(polkadot_track.clone()),
				)),
			}
		},
	}
}

// Generate the calls needed for a proposal to pass through the Fellowship.
fn kusama_fellowship_referenda(input: &ReferendumInput) -> Result<ReferendumCalls, Error> {
	use kusama_asset_hub::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
		pallet_whitelist::pallet::Call as WhitelistCall,
	};
	use kusama_relay::runtime_types::bounded_collections::bounded_vec::BoundedVec;
	use kusama_relay::runtime_types::{
		frame_support::traits::{
			preimages::Bounded::Inline, schedule::DispatchTime as KusamaDispatchTime,
		},
		// Since the Relay Chain and Asset Hub may be on different versions of Preimage,
		// Referenda, and XCM pallets, we need to define their `Call` enum separately.
		pallet_referenda::pallet::Call as KusamaReferendaCall,
		pallet_xcm::pallet::Call as KusamaXcmCall,
		staging_xcm::v5::{
			junction::Junction::Parachain, junctions::Junctions::X1, location::Location,
			Instruction, Xcm,
		},
		xcm::{
			double_encoded::DoubleEncoded, v3::OriginKind, v3::WeightLimit, VersionedLocation,
			VersionedXcm::V5,
		},
	};
	// Fellowship is on the Relay Chain, so things are a bit different here.
	//
	// 1. Create a whitelist call on Asset Hub:
	//
	//    let whitelist_call =
	//        KusamaAssetHubRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
	//            call_hash: H256(proposal_hash),
	//    });
	//
	// 2. Create an XCM to send this from the Relay Chain:
	//    let send_whitelist = KusamaRuntimeCall::XcmPallet(
	//        PolkadotXcmCall::send {
	//            dest: Location { parents: 0, interior: X1([Parachain(1000)]) },
	//            message: vec![UnpaidExecution, Transact {call: whitelist_call, ..}],
	//        }
	//    );
	//
	// 3. Make a Fellowship referendum for `send_whitelist`.
	//
	// 4. Make a public referendum on Asset Hub.
	let proposal_call_info = CallInfo::from_bytes(&input.proposal, Network::KusamaAssetHub);

	let public_referendum_dispatch_time = match input.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};

	let whitelist_call = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
		KusamaAssetHubRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
			call_hash: H256(proposal_call_info.hash),
		}),
	));

	// This is what the Fellowship will actually vote on enacting.
	let whitelist_over_xcm = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::XcmPallet(KusamaXcmCall::send {
			dest: Box::new(VersionedLocation::V5(Location {
				parents: 0,
				interior: X1([Parachain(1000)]),
			})),
			message: Box::new(V5(Xcm(vec![
				Instruction::UnpaidExecution {
					weight_limit: WeightLimit::Unlimited,
					check_origin: None,
				},
				Instruction::Transact {
					origin_kind: OriginKind::Xcm,
					fallback_max_weight: None,
					call: DoubleEncoded { encoded: whitelist_call.encoded },
				},
			]))),
		}),
	));

	// The Inline limit is 128 bytes. There is no longer a preimage pallet on Kusama Relay Chain to
	// fall back to.
	if whitelist_over_xcm.length > 128 {
		return Err(Error::InlineLimitExceeded(whitelist_over_xcm.length));
	}

	// The actual Fellowship referendum submission.
	let fellowship_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::Kusama(
		KusamaRuntimeCall::FellowshipReferenda(KusamaReferendaCall::submit {
			proposal_origin: Box::new(KusamaOriginCaller::Origins(KusamaOpenGovOrigin::Fellows)),
			proposal: Inline(BoundedVec(whitelist_over_xcm.encoded)),
			enactment_moment: KusamaDispatchTime::After(10u32),
		}),
	));

	// Now we put together the public referendum part. This still needs separate logic because the
	// actual proposal gets wrapped in a Whitelist call.
	let dispatch_whitelisted_call = CallInfo::from_runtime_call(
		NetworkRuntimeCall::KusamaAssetHub(KusamaAssetHubRuntimeCall::Whitelist(
			WhitelistCall::dispatch_whitelisted_call_with_preimage {
				call: Box::new(proposal_call_info.get_kusama_asset_hub_call()?),
			},
		)),
	);

	let preimage_for_dispatch_whitelisted_call = CallInfo::from_runtime_call(
		NetworkRuntimeCall::KusamaAssetHub(KusamaAssetHubRuntimeCall::Preimage(
			PreimageCall::note_preimage { bytes: dispatch_whitelisted_call.encoded.clone() },
		)),
	);
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
		KusamaAssetHubRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(KusamaAssetHubOriginCaller::Origins(
				KusamaAssetHubOpenGovOrigin::WhitelistedCaller,
			)),
			proposal: Lookup {
				hash: H256(dispatch_whitelisted_call.hash),
				len: dispatch_whitelisted_call.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));

	Ok(ReferendumCalls {
		fellowship_preimage: None,
		fellowship_referendum: Some(fellowship_proposal),
		public_preimage: preimage_for_dispatch_whitelisted_call,
		public_referendum: public_proposal,
		public_preimage_hash: dispatch_whitelisted_call.hash,
		public_preimage_len: dispatch_whitelisted_call.length,
		public_preimage_bytes: dispatch_whitelisted_call.encoded,
		whitelisted_call_hash: Some(proposal_call_info.hash),
	})
}

// Generate the calls needed for a Kusama proposal to pass through the Polkadot Fellowship.
// The Fellowship referendum is submitted on Polkadot Collectives, while the public referendum
// targets Kusama Asset Hub.
fn polkadot_fellowship_for_kusama_referenda(
	input: &ReferendumInput,
) -> Result<ReferendumCalls, Error> {
	use kusama_asset_hub::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
		pallet_whitelist::pallet::Call as WhitelistCall,
	};
	use polkadot_collectives::runtime_types::staging_xcm::v5::junction::NetworkId;
	use polkadot_collectives::runtime_types::{
		bounded_collections::bounded_vec::BoundedVec as CollectivesBoundedVec,
		pallet_xcm::pallet::Call as CollectivesXcmCall,
		staging_xcm::v5::{
			junction::Junction::{GlobalConsensus, Parachain},
			junctions::Junctions::{X1, X2},
			location::Location,
			Instruction, Xcm,
		},
		xcm::{
			double_encoded::DoubleEncoded, v3::MaybeErrorCode, v3::OriginKind, v3::WeightLimit,
			VersionedLocation, VersionedXcm::V5,
		},
	};
	// Fellowship is on the Polkadot Collectives chain.
	//
	// 1. Create a whitelist call on Kusama Asset Hub.
	// 2. Create an XCM send call on the Collectives chain to Transact this on Kusama Asset Hub
	//    via Polkadot Asset Hub and the bridge.
	// 3. Make a Fellowship referendum on Collectives for the XCM send.
	// 4. Make a public referendum on Kusama Asset Hub.
	let proposal_call_info = CallInfo::from_bytes(&input.proposal, Network::KusamaAssetHub);

	let public_referendum_dispatch_time = match input.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};

	// whitelist_call for AHK
	let whitelist_call = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
		KusamaAssetHubRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
			call_hash: H256(proposal_call_info.hash),
		}),
	));

	// XCM to execute on AHK (final destination, appended by InitiateTransfer)
	let xcm_on_ahk = Xcm(vec![
		Instruction::Transact {
			origin_kind: OriginKind::Xcm,
			call: DoubleEncoded { encoded: whitelist_call.encoded },
			fallback_max_weight: None,
		},
		Instruction::ExpectTransactStatus(MaybeErrorCode::Success),
	]);

	// AHK destination as seen from AHP
	let ahk_from_ahp = Location {
		parents: 2,
		interior: X2([GlobalConsensus(NetworkId::Kusama), Parachain(1000)]),
	};

	// XCM to execute on AHP: unpaid execution (system parachain), then InitiateTransfer
	// forwards message to AHK over bridge, preserving the Fellows origin via AliasOrigin.
	let xcm_for_ahp = Xcm(vec![
		Instruction::UnpaidExecution { weight_limit: WeightLimit::Unlimited, check_origin: None },
		Instruction::InitiateTransfer {
			destination: ahk_from_ahp,
			remote_fees: None,
			preserve_origin: true,
			assets: CollectivesBoundedVec(vec![]),
			remote_xcm: xcm_on_ahk,
		},
	]);

	// This is what the Fellowship will actually vote on enacting.
	// Send from Collectives to Polkadot Asset Hub (sibling parachain), which then bridges
	// to Kusama Asset Hub via InitiateTransfer.
	let whitelist_over_xcm = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
		CollectivesRuntimeCall::PolkadotXcm(CollectivesXcmCall::send {
			dest: Box::new(VersionedLocation::V5(Location {
				parents: 1,
				interior: X1([Parachain(1000)]),
			})),
			message: Box::new(V5(xcm_for_ahp)),
		}),
	));

	let (fellowship_proposal, preimage_for_whitelist_over_xcm) =
		polkadot_fellowship_proposal(whitelist_over_xcm);

	// Now we put together the public referendum part on Kusama Asset Hub.
	let dispatch_whitelisted_call = CallInfo::from_runtime_call(
		NetworkRuntimeCall::KusamaAssetHub(KusamaAssetHubRuntimeCall::Whitelist(
			WhitelistCall::dispatch_whitelisted_call_with_preimage {
				call: Box::new(proposal_call_info.get_kusama_asset_hub_call()?),
			},
		)),
	);

	let preimage_for_dispatch_whitelisted_call = CallInfo::from_runtime_call(
		NetworkRuntimeCall::KusamaAssetHub(KusamaAssetHubRuntimeCall::Preimage(
			PreimageCall::note_preimage { bytes: dispatch_whitelisted_call.encoded.clone() },
		)),
	);
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
		KusamaAssetHubRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(KusamaAssetHubOriginCaller::Origins(
				KusamaAssetHubOpenGovOrigin::WhitelistedCaller,
			)),
			proposal: Lookup {
				hash: H256(dispatch_whitelisted_call.hash),
				len: dispatch_whitelisted_call.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));

	Ok(ReferendumCalls {
		fellowship_preimage: preimage_for_whitelist_over_xcm,
		fellowship_referendum: Some(fellowship_proposal),
		public_preimage: preimage_for_dispatch_whitelisted_call,
		public_referendum: public_proposal,
		public_preimage_hash: dispatch_whitelisted_call.hash,
		public_preimage_len: dispatch_whitelisted_call.length,
		public_preimage_bytes: dispatch_whitelisted_call.encoded,
		whitelisted_call_hash: Some(proposal_call_info.hash),
	})
}

// Wrap a Collectives call in a Polkadot Fellowship referendum on the `Fellows` track. Returns the
// referendum submission and, if the call is too long to be inlined in it, the call to note its
// preimage.
pub(crate) fn polkadot_fellowship_proposal(
	whitelist_over_xcm: CallInfo,
) -> (CallInfo, Option<CallInfo>) {
	use polkadot_collectives::runtime_types::{
		bounded_collections::bounded_vec::BoundedVec as CollectivesBoundedVec,
		collectives_polkadot_runtime::OriginCaller as CollectivesOriginCaller,
		frame_support::traits::{
			preimages::Bounded::{Inline as CollectivesInline, Lookup as CollectivesLookup},
			schedule::DispatchTime as CollectivesDispatchTime,
		},
		pallet_preimage::pallet::Call as CollectivesPreimageCall,
		pallet_referenda::pallet::Call as CollectivesReferendaCall,
	};

	// The Inline limit is 128 bytes. Use Inline if within limit, otherwise fall back to Lookup.
	if whitelist_over_xcm.length <= 128 {
		let proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
			CollectivesRuntimeCall::FellowshipReferenda(CollectivesReferendaCall::submit {
				proposal_origin: Box::new(CollectivesOriginCaller::FellowshipOrigins(
					FellowshipOrigins::Fellows,
				)),
				proposal: CollectivesInline(CollectivesBoundedVec(whitelist_over_xcm.encoded)),
				enactment_moment: CollectivesDispatchTime::After(10u32),
			}),
		));
		(proposal, None)
	} else {
		let preimage = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
			CollectivesRuntimeCall::Preimage(CollectivesPreimageCall::note_preimage {
				bytes: whitelist_over_xcm.encoded.clone(),
			}),
		));
		let proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
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
		(proposal, Some(preimage))
	}
}

// Generate the calls needed for a proposal to pass on Kusama without the Fellowship.
fn kusama_non_fellowship_referenda(
	input: &ReferendumInput,
	origin: KusamaAssetHubOriginCaller,
) -> ReferendumCalls {
	use kusama_asset_hub::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
	};

	let proposal_call_info = CallInfo::from_bytes(&input.proposal, Network::KusamaAssetHub);

	let public_referendum_dispatch_time = match input.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};

	let note_proposal_preimage = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
		KusamaAssetHubRuntimeCall::Preimage(PreimageCall::note_preimage {
			bytes: input.proposal.clone(),
		}),
	));
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::KusamaAssetHub(
		KusamaAssetHubRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(origin),
			proposal: Lookup {
				hash: H256(proposal_call_info.hash),
				len: proposal_call_info.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));

	ReferendumCalls {
		fellowship_preimage: None,
		fellowship_referendum: None,
		public_preimage: note_proposal_preimage,
		public_referendum: public_proposal,
		public_preimage_hash: proposal_call_info.hash,
		public_preimage_len: proposal_call_info.length,
		public_preimage_bytes: proposal_call_info.encoded,
		whitelisted_call_hash: None,
	}
}

// Generate the calls needed for a proposal to pass through the Polkadot Fellowship.
fn polkadot_fellowship_referenda(input: &ReferendumInput) -> Result<ReferendumCalls, Error> {
	use polkadot_asset_hub::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
		pallet_whitelist::pallet::Call as WhitelistCall,
	};
	// Since Asset Hub and Collectives may be on different versions of the XCM pallet, we need to
	// define their `Call` enum separately.
	use polkadot_collectives::runtime_types::{
		pallet_xcm::pallet::Call as CollectivesXcmCall,
		staging_xcm::v5::{
			junction::Junction::Parachain, junctions::Junctions::X1, location::Location,
			Instruction, Xcm,
		},
		xcm::{
			double_encoded::DoubleEncoded, v3::OriginKind, v3::WeightLimit, VersionedLocation,
			VersionedXcm::V5,
		},
	};
	// Fellowship is on the Collectives chain, so things are a bit different here.
	//
	// 1. Create a whitelist call on Asset Hub:
	//
	//    let whitelist_call =
	//        PolkadotAssetHubRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
	//            call_hash: H256(proposal_hash),
	//    });
	//
	// 2. Create an XCM send call on the Collectives chain to Transact this on Asset Hub:
	//
	//    let send_whitelist = CollectivesRuntimeCall::PolkadotXcm(
	//        PolkadotXcmCall::send {
	//            dest: Location { parents: 1, interior: X1([Parachain(1000)]) },
	//            message: vec![UnpaidExecution, Transact {call: whitelist_call, ..}],
	//        }
	//    );
	//
	// 3. Make a Fellowship referendum for `send_whitelist`.
	//
	// 4. Make a public referendum on Asset Hub.
	let proposal_call_info = CallInfo::from_bytes(&input.proposal, Network::PolkadotAssetHub);

	let public_referendum_dispatch_time = match input.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};
	// Whitelist the call on Asset Hub.
	let whitelist_call = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
		PolkadotAssetHubRuntimeCall::Whitelist(WhitelistCall::whitelist_call {
			call_hash: H256(proposal_call_info.hash),
		}),
	));

	// This is what the Fellowship will actually vote on enacting.
	let whitelist_over_xcm = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotCollectives(
		CollectivesRuntimeCall::PolkadotXcm(CollectivesXcmCall::send {
			dest: Box::new(VersionedLocation::V5(Location {
				parents: 1,
				interior: X1([Parachain(1000)]),
			})),
			message: Box::new(V5(Xcm(vec![
				Instruction::UnpaidExecution {
					weight_limit: WeightLimit::Unlimited,
					check_origin: None,
				},
				Instruction::Transact {
					origin_kind: OriginKind::Xcm,
					fallback_max_weight: None,
					call: DoubleEncoded { encoded: whitelist_call.encoded },
				},
			]))),
		}),
	));

	let (fellowship_proposal, preimage_for_whitelist_over_xcm) =
		polkadot_fellowship_proposal(whitelist_over_xcm);

	// Now we put together the public referendum part. This still needs separate logic because the
	// actual proposal gets wrapped in a Whitelist call.
	let dispatch_whitelisted_call = CallInfo::from_runtime_call(
		NetworkRuntimeCall::PolkadotAssetHub(PolkadotAssetHubRuntimeCall::Whitelist(
			WhitelistCall::dispatch_whitelisted_call_with_preimage {
				call: Box::new(proposal_call_info.get_polkadot_asset_hub_call()?),
			},
		)),
	);

	let preimage_for_dispatch_whitelisted_call = CallInfo::from_runtime_call(
		NetworkRuntimeCall::PolkadotAssetHub(PolkadotAssetHubRuntimeCall::Preimage(
			PreimageCall::note_preimage { bytes: dispatch_whitelisted_call.encoded.clone() },
		)),
	);
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
		PolkadotAssetHubRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(PolkadotAssetHubOriginCaller::Origins(
				PolkadotAssetHubOpenGovOrigin::WhitelistedCaller,
			)),
			proposal: Lookup {
				hash: H256(dispatch_whitelisted_call.hash),
				len: dispatch_whitelisted_call.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));

	Ok(ReferendumCalls {
		fellowship_preimage: preimage_for_whitelist_over_xcm,
		fellowship_referendum: Some(fellowship_proposal),
		public_preimage: preimage_for_dispatch_whitelisted_call,
		public_referendum: public_proposal,
		public_preimage_hash: dispatch_whitelisted_call.hash,
		public_preimage_len: dispatch_whitelisted_call.length,
		public_preimage_bytes: dispatch_whitelisted_call.encoded,
		whitelisted_call_hash: Some(proposal_call_info.hash),
	})
}

// Generate the calls needed for a proposal to pass on Polkadot without the Fellowship.
fn polkadot_non_fellowship_referenda(
	input: &ReferendumInput,
	origin: PolkadotAssetHubOriginCaller,
) -> ReferendumCalls {
	use polkadot_asset_hub::runtime_types::{
		frame_support::traits::{preimages::Bounded::Lookup, schedule::DispatchTime},
		pallet_preimage::pallet::Call as PreimageCall,
		pallet_referenda::pallet::Call as ReferendaCall,
	};

	let proposal_call_info = CallInfo::from_bytes(&input.proposal, Network::PolkadotAssetHub);

	let public_referendum_dispatch_time = match input.dispatch {
		DispatchTimeWrapper::At(block) => DispatchTime::At(block),
		DispatchTimeWrapper::After(block) => DispatchTime::After(block),
	};

	let note_proposal_preimage = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
		PolkadotAssetHubRuntimeCall::Preimage(PreimageCall::note_preimage {
			bytes: input.proposal.clone(),
		}),
	));
	let public_proposal = CallInfo::from_runtime_call(NetworkRuntimeCall::PolkadotAssetHub(
		PolkadotAssetHubRuntimeCall::Referenda(ReferendaCall::submit {
			proposal_origin: Box::new(origin),
			proposal: Lookup {
				hash: H256(proposal_call_info.hash),
				len: proposal_call_info.length,
			},
			enactment_moment: public_referendum_dispatch_time,
		}),
	));

	ReferendumCalls {
		fellowship_preimage: None,
		fellowship_referendum: None,
		public_preimage: note_proposal_preimage,
		public_referendum: public_proposal,
		public_preimage_hash: proposal_call_info.hash,
		public_preimage_len: proposal_call_info.length,
		public_preimage_bytes: proposal_call_info.encoded,
		whitelisted_call_hash: None,
	}
}
