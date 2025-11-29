pub(super) use parity_scale_codec::Encode as _;
pub(super) use sp_core::blake2_256;
pub(super) use subxt::utils::H256;

// Kusama Chains -----------------------------------------------------------------------------------

#[subxt::subxt(
	runtime_metadata_path = "metadata/kusama.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod kusama_relay {}
pub(super) use kusama_relay::runtime_types::staging_kusama_runtime::{
	governance::origins::pallet_custom_origins::Origin as KusamaOpenGovOrigin,
	OriginCaller as KusamaOriginCaller, RuntimeCall as KusamaRuntimeCall,
};

#[subxt::subxt(
	runtime_metadata_path = "metadata/kusama_asset_hub.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod kusama_asset_hub {}
pub(super) use kusama_asset_hub::runtime_types::asset_hub_kusama_runtime::{
	governance::origins::pallet_custom_origins::Origin as KusamaAssetHubOpenGovOrigin,
	OriginCaller as KusamaAssetHubOriginCaller, RuntimeCall as KusamaAssetHubRuntimeCall,
};

#[subxt::subxt(runtime_metadata_path = "metadata/kusama_bridge_hub.scale")]
pub mod kusama_bridge_hub {}
pub(super) use kusama_bridge_hub::runtime_types::bridge_hub_kusama_runtime::RuntimeCall as KusamaBridgeHubRuntimeCall;

#[subxt::subxt(runtime_metadata_path = "metadata/kusama_encointer.scale")]
pub mod kusama_encointer {}
pub(super) use kusama_encointer::runtime_types::encointer_kusama_runtime::RuntimeCall as KusamaEncointerRuntimeCall;

#[subxt::subxt(runtime_metadata_path = "metadata/kusama_people.scale")]
pub mod kusama_people {}
pub(super) use kusama_people::runtime_types::people_kusama_runtime::RuntimeCall as KusamaPeopleRuntimeCall;

#[subxt::subxt(runtime_metadata_path = "metadata/kusama_coretime.scale")]
pub mod kusama_coretime {}
pub(super) use kusama_coretime::runtime_types::coretime_kusama_runtime::RuntimeCall as KusamaCoretimeRuntimeCall;

// Polkadot Chains ---------------------------------------------------------------------------------

#[subxt::subxt(
	runtime_metadata_path = "metadata/polkadot.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod polkadot_relay {}
pub(super) use polkadot_relay::runtime_types::polkadot_runtime::RuntimeCall as PolkadotRuntimeCall;

#[subxt::subxt(
	runtime_metadata_path = "metadata/polkadot_asset_hub.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod polkadot_asset_hub {}
pub(super) use polkadot_asset_hub::runtime_types::asset_hub_polkadot_runtime::{
	governance::origins::pallet_custom_origins::Origin as PolkadotAssetHubOpenGovOrigin,
	OriginCaller as PolkadotAssetHubOriginCaller, RuntimeCall as PolkadotAssetHubRuntimeCall,
};

#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_collectives.scale")]
pub mod polkadot_collectives {}
pub(super) use polkadot_collectives::runtime_types::collectives_polkadot_runtime::{
	fellowship::origins::pallet_origins::Origin as FellowshipOrigins,
	RuntimeCall as CollectivesRuntimeCall,
};

#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_bridge_hub.scale")]
pub mod polkadot_bridge_hub {}
pub(super) use polkadot_bridge_hub::runtime_types::bridge_hub_polkadot_runtime::RuntimeCall as PolkadotBridgeHubRuntimeCall;

#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_people.scale")]
pub mod polkadot_people {}
pub(super) use polkadot_people::runtime_types::people_polkadot_runtime::RuntimeCall as PolkadotPeopleRuntimeCall;

#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_coretime.scale")]
pub mod polkadot_coretime {}
pub(super) use polkadot_coretime::runtime_types::coretime_polkadot_runtime::RuntimeCall as PolkadotCoretimeRuntimeCall;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Network {
	Kusama,
	KusamaAssetHub,
	KusamaEncointer,
	KusamaBridgeHub,
	KusamaPeople,
	KusamaCoretime,
	Polkadot,
	PolkadotAssetHub,
	PolkadotCollectives,
	PolkadotBridgeHub,
	PolkadotPeople,
	PolkadotCoretime,
}

impl Network {
	/// Return the `ParaId` of a given network. Returns an error if the network is not a parachain.
	pub(super) fn get_para_id(&self) -> Result<u32, &'static str> {
		use Network::*;
		match &self {
			// Kusama
			Kusama => Err("relay chain"),
			KusamaAssetHub => Ok(1_000),
			KusamaBridgeHub => Ok(1_002),
			KusamaPeople => Ok(1_004),
			KusamaCoretime => Ok(1_005),
			KusamaEncointer => Ok(1_001),
			// Polkadot
			Polkadot => Err("relay chain"),
			PolkadotAssetHub => Ok(1_000),
			PolkadotBridgeHub => Ok(1_002),
			PolkadotCollectives => Ok(1_001),
			PolkadotPeople => Ok(1_004),
			PolkadotCoretime => Ok(1_005),
		}
	}
}

// Info and preferences provided by the user for proposal submission.
pub(super) struct ProposalDetails {
	// The proposal, generated elsewhere and pasted here.
	pub(super) proposal: String,
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
	// Whether to use light client endpoints in PAPI links (default true).
	pub(super) use_light_client: bool,
}

// Info and preferences provided by the user for runtime upgrade construction.
pub(super) struct UpgradeDetails {
	// The Relay Network for this upgrade, Polkadot or Kusama.
	pub(super) relay: Network,
	// All networks to upgrade.
	pub(super) networks: Vec<VersionedNetwork>,
	// The directory into which to write information needed.
	pub(super) directory: String,
	// The filename of the output.
	pub(super) output_file: String,
	// An additional call to be enacted in the same batch as the system upgrade.
	pub(super) additional: Option<CallInfo>,
}

// A network and the version to which it will upgrade.
#[derive(Debug, PartialEq)]
pub(super) struct VersionedNetwork {
	// A network identifier.
	pub(super) network: Network,
	// A runtime version number (i.e. "9430", not "0.9.43").
	pub(super) version: String,
}

// The network and OpenGov track this proposal should be voted on.
pub(super) enum NetworkTrack {
	KusamaRoot,
	Kusama(KusamaAssetHubOpenGovOrigin),
	PolkadotRoot,
	Polkadot(PolkadotAssetHubOpenGovOrigin),
}

// A runtime call wrapped in the network it should execute on.
pub(super) enum NetworkRuntimeCall {
	Kusama(KusamaRuntimeCall),
	KusamaAssetHub(KusamaAssetHubRuntimeCall),
	KusamaBridgeHub(KusamaBridgeHubRuntimeCall),
	KusamaPeople(KusamaPeopleRuntimeCall),
	KusamaCoretime(KusamaCoretimeRuntimeCall),
	KusamaEncointer(KusamaEncointerRuntimeCall),
	Polkadot(PolkadotRuntimeCall),
	PolkadotAssetHub(PolkadotAssetHubRuntimeCall),
	PolkadotCollectives(CollectivesRuntimeCall),
	PolkadotBridgeHub(PolkadotBridgeHubRuntimeCall),
	PolkadotPeople(PolkadotPeopleRuntimeCall),
	PolkadotCoretime(PolkadotCoretimeRuntimeCall),
}

// How the user would like to see the output of the program.
pub(super) enum Output {
	// Print just the call data (e.g. 0x1234).
	CallData,
	// Print a clickable link to view the decoded call on Polkadot JS Apps UI.
	AppsUiLink,
}

// Local concrete type to use in each runtime's `DispatchTime`
pub(super) enum DispatchTimeWrapper {
	At(u32),
	After(u32),
}

// A call or a hash. Used for printing (or rather, to avoid printing large calls).
// The Hash variant is only used when calls exceed the output length limit, which is rare.
#[allow(clippy::large_enum_variant)]
pub(super) enum CallOrHash {
	Call(NetworkRuntimeCall),
	Hash([u8; 32]),
}

// All the info associated with a call in the forms you may need it in.
#[derive(Clone)]
pub(super) struct CallInfo {
	pub(super) network: Network,
	pub(super) encoded: Vec<u8>,
	pub(super) hash: [u8; 32],
	pub(super) length: u32,
}

impl CallInfo {
	// Construct `Self` from a `NetworkRuntimeCall`.
	pub(super) fn from_runtime_call(call: NetworkRuntimeCall) -> Self {
		let (network, encoded) = match &call {
			NetworkRuntimeCall::Kusama(cc) => (Network::Kusama, cc.encode()),
			NetworkRuntimeCall::KusamaAssetHub(cc) => (Network::KusamaAssetHub, cc.encode()),
			NetworkRuntimeCall::KusamaBridgeHub(cc) => (Network::KusamaBridgeHub, cc.encode()),
			NetworkRuntimeCall::KusamaPeople(cc) => (Network::KusamaPeople, cc.encode()),
			NetworkRuntimeCall::KusamaCoretime(cc) => (Network::KusamaCoretime, cc.encode()),
			NetworkRuntimeCall::KusamaEncointer(cc) => (Network::KusamaEncointer, cc.encode()),
			NetworkRuntimeCall::Polkadot(cc) => (Network::Polkadot, cc.encode()),
			NetworkRuntimeCall::PolkadotAssetHub(cc) => (Network::PolkadotAssetHub, cc.encode()),
			NetworkRuntimeCall::PolkadotCollectives(cc) =>
				(Network::PolkadotCollectives, cc.encode()),
			NetworkRuntimeCall::PolkadotBridgeHub(cc) => (Network::PolkadotBridgeHub, cc.encode()),
			NetworkRuntimeCall::PolkadotPeople(cc) => (Network::PolkadotPeople, cc.encode()),
			NetworkRuntimeCall::PolkadotCoretime(cc) => (Network::PolkadotCoretime, cc.encode()),
		};
		let hash = blake2_256(&encoded);
		let length: u32 = (encoded.len()).try_into().unwrap();
		Self { network, encoded: encoded.to_vec(), hash, length }
	}

	// Construct `Self` for some `network` given some `encoded` bytes.
	pub(super) fn from_bytes(encoded: &[u8], network: Network) -> Self {
		let hash = blake2_256(encoded);
		let length = (encoded.len()).try_into().unwrap();
		Self { network, encoded: encoded.to_vec(), hash, length }
	}

	// Strip the outer enum and return a Kusama Relay `RuntimeCall`.
	pub(super) fn get_kusama_call(&self) -> Result<KusamaRuntimeCall, &'static str> {
		match &self.network {
			Network::Kusama => {
				let bytes = &self.encoded;
				Ok(<KusamaRuntimeCall as parity_scale_codec::Decode>::decode(&mut &bytes[..])
					.unwrap())
			},
			_ => Err("not a kusama call"),
		}
	}

	// Strip the outer enum and return a Kusama Asset Hub `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_kusama_asset_hub_call(
		&self,
	) -> Result<KusamaAssetHubRuntimeCall, &'static str> {
		match &self.network {
			Network::KusamaAssetHub => {
				let bytes = &self.encoded;
				Ok(<KusamaAssetHubRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a kusama asset hub call"),
		}
	}

	// Strip the outer enum and return a Kusama Bridge Hub `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_kusama_bridge_hub_call(
		&self,
	) -> Result<KusamaBridgeHubRuntimeCall, &'static str> {
		match &self.network {
			Network::KusamaBridgeHub => {
				let bytes = &self.encoded;
				Ok(<KusamaBridgeHubRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a kusama bridge hub call"),
		}
	}

	// Strip the outer enum and return a Kusama Encointer `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_kusama_encointer_call(
		&self,
	) -> Result<KusamaEncointerRuntimeCall, &'static str> {
		match &self.network {
			Network::KusamaEncointer => {
				let bytes = &self.encoded;
				Ok(<KusamaEncointerRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a kusama encointer call"),
		}
	}

	// Strip the outer enum and return a Kusama People `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_kusama_people_call(&self) -> Result<KusamaPeopleRuntimeCall, &'static str> {
		match &self.network {
			Network::KusamaPeople => {
				let bytes = &self.encoded;
				Ok(<KusamaPeopleRuntimeCall as parity_scale_codec::Decode>::decode(&mut &bytes[..])
					.unwrap())
			},
			_ => Err("not a kusama people call"),
		}
	}

	// Strip the outer enum and return a Kusama Coretime `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_kusama_coretime_call(
		&self,
	) -> Result<KusamaCoretimeRuntimeCall, &'static str> {
		match &self.network {
			Network::KusamaCoretime => {
				let bytes = &self.encoded;
				Ok(<KusamaCoretimeRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a kusama coretime call"),
		}
	}

	// Strip the outer enum and return a Polkadot Relay `RuntimeCall`.
	pub(super) fn get_polkadot_call(&self) -> Result<PolkadotRuntimeCall, &'static str> {
		match &self.network {
			Network::Polkadot => {
				let bytes = &self.encoded;
				Ok(<PolkadotRuntimeCall as parity_scale_codec::Decode>::decode(&mut &bytes[..])
					.unwrap())
			},
			_ => Err("not a polkadot call"),
		}
	}

	// Strip the outer enum and return a Polkadot Asset Hub `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_polkadot_asset_hub_call(
		&self,
	) -> Result<PolkadotAssetHubRuntimeCall, &'static str> {
		match &self.network {
			Network::PolkadotAssetHub => {
				let bytes = &self.encoded;
				Ok(<PolkadotAssetHubRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a polkadot asset hub call"),
		}
	}

	// Strip the outer enum and return a Polkadot Collectives `RuntimeCall`.
	pub(super) fn get_polkadot_collectives_call(
		&self,
	) -> Result<CollectivesRuntimeCall, &'static str> {
		match &self.network {
			Network::PolkadotCollectives => {
				let bytes = &self.encoded;
				Ok(<CollectivesRuntimeCall as parity_scale_codec::Decode>::decode(&mut &bytes[..])
					.unwrap())
			},
			_ => Err("not a polkadot collectives call"),
		}
	}

	// Strip the outer enum and return a Polkadot Bridge Hub `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_polkadot_bridge_hub_call(
		&self,
	) -> Result<PolkadotBridgeHubRuntimeCall, &'static str> {
		match &self.network {
			Network::PolkadotBridgeHub => {
				let bytes = &self.encoded;
				Ok(<PolkadotBridgeHubRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a polkadot bridge hub call"),
		}
	}

	// Strip the outer enum and return a Polkadot People `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_polkadot_people_call(
		&self,
	) -> Result<PolkadotPeopleRuntimeCall, &'static str> {
		match &self.network {
			Network::PolkadotPeople => {
				let bytes = &self.encoded;
				Ok(<PolkadotPeopleRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a polkadot people call"),
		}
	}

	// Strip the outer enum and return a Polkadot Coretime `RuntimeCall`.
	#[allow(dead_code)]
	pub(super) fn get_polkadot_coretime_call(
		&self,
	) -> Result<PolkadotCoretimeRuntimeCall, &'static str> {
		match &self.network {
			Network::PolkadotCoretime => {
				let bytes = &self.encoded;
				Ok(<PolkadotCoretimeRuntimeCall as parity_scale_codec::Decode>::decode(
					&mut &bytes[..],
				)
				.unwrap())
			},
			_ => Err("not a polkadot coretime call"),
		}
	}

	// Take `Self` and a length limit as input. If the call length exceeds the limit, just return
	// its hash. Call length is recomputed and will be 2 bytes longer than the actual preimage
	// length. This is because the call is `preimage.note_preimage(call)`, so the outer pallet/call
	// indices have a length of 2 bytes.
	pub(super) fn create_print_output(&self, length_limit: u32) -> (CallOrHash, u32) {
		let print_output = if self.length > length_limit {
			CallOrHash::Hash(self.hash)
		} else {
			match &self.network {
				Network::Kusama => {
					let kusama_call = self.get_kusama_call().expect("kusama");
					CallOrHash::Call(NetworkRuntimeCall::Kusama(kusama_call))
				},
				Network::KusamaAssetHub => {
					let kusama_asset_hub_call =
						self.get_kusama_asset_hub_call().expect("kusama asset hub");
					CallOrHash::Call(NetworkRuntimeCall::KusamaAssetHub(kusama_asset_hub_call))
				},
				Network::Polkadot => {
					let polkadot_call = self.get_polkadot_call().expect("polkadot");
					CallOrHash::Call(NetworkRuntimeCall::Polkadot(polkadot_call))
				},
				Network::PolkadotAssetHub => {
					let polkadot_asset_hub_call =
						self.get_polkadot_asset_hub_call().expect("polkadot asset hub");
					CallOrHash::Call(NetworkRuntimeCall::PolkadotAssetHub(polkadot_asset_hub_call))
				},
				Network::PolkadotCollectives => {
					let collectives_call =
						self.get_polkadot_collectives_call().expect("collectives");
					CallOrHash::Call(NetworkRuntimeCall::PolkadotCollectives(collectives_call))
				},
				_ => panic!("to do"),
			}
		};
		(print_output, self.length)
	}
}

// The set of calls that some user will need to sign and submit to initiate a referendum.
pub(super) struct PossibleCallsToSubmit {
	// `Some` if using the Fellowship to Whitelist a call. The second value is the length of the
	// call, which may be relevant to the print output.
	//
	// ```
	// preimage.note(whitelist.whitelist_call(hash(proposal)));
	// ```
	pub(super) preimage_for_whitelist_call: Option<(CallOrHash, u32)>,
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
	pub(super) preimage_for_public_referendum: Option<(CallOrHash, u32)>,
	// The actual submission of the Fellowship referendum to Whitelist a call. `None` when not using
	// Whitelist.
	pub(super) fellowship_referendum_submission: Option<NetworkRuntimeCall>,
	// The actual submission of the public referendum. The `proposal` is the proposal itself when
	// not using the Whitelist, or the dispatch call with nested proposal when using the Whitelist.
	pub(super) public_referendum_submission: Option<NetworkRuntimeCall>,
}
