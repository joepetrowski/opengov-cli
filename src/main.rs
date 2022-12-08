#[subxt::subxt(runtime_metadata_url = "wss://kusama-rpc.polkadot.io:443")]
pub mod kusama {
	#[subxt(substitute_type = "sp_runtime::multiaddress::MultiAddress")]
	use ::subxt::ext::sp_runtime::MultiAddress;
}

// use subxt::{ext::sp_runtime::{AccountId32, MultiAddress}, };
// use parity_scale_codec::Encode as _;
use std::str::FromStr as _;
use kusama::runtime_types::{
	kusama_runtime::{
        governance::origins::pallet_custom_origins::Origin as OpenGovOrigin,
        RuntimeCall
    },
    pallet_whitelist::pallet::Call as WhitelistCall,
    pallet_referenda::pallet::Call as ReferendaCall,
    // pallet_fellowship_referenda::pallet::Call as FellowshipReferendaCall, // why doesn't this work?
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
    preimage_for_whitelist_call: Option<PreimageCall>,
    // ```
    // // Without Fellowship
    // preimage.note(proposal);
    //
    // // With Fellowship
    // preimage.note(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
    // ```
    preimage_for_public_referendum: Option<PreimageCall>,
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
    fellowship_referendum_submission: Option<ReferendaCall>, // need fellowship referenda call
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
    public_referendum_submission: Option<ReferendaCall>,
}

fn main() {
    let proposal_details = get_the_actual_proposed_action();

    let calls: PossibleCallsToSubmit = match proposal_details.track {
        OpenGovOrigin::WhitelistedCaller => {
            PossibleCallsToSubmit {
                // preimage.note(whitelist.whitelist_call(hash(proposal)));
                preimage_for_whitelist_call: None,
                // preimage.note(whitelist.dispatch_whitelisted_call_with_preimage(proposal));
                preimage_for_public_referendum: None,
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
                fellowship_referendum_submission: None,
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
                public_referendum_submission: None,
            }
        },
        _ => {
            PossibleCallsToSubmit {
                // None
                preimage_for_whitelist_call: None,
                // preimage.note(proposal);
                preimage_for_public_referendum: None,
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
                public_referendum_submission: None,
            }
        },
    };

    // Todo: Encode and log all necessary calls to console.
}
