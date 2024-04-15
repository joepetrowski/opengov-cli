pub(super) use parity_scale_codec::Encode as _;
pub(super) use sp_core::{blake2_256, H256};

// Kusama Chains -----------------------------------------------------------------------------------

#[subxt::subxt(
	runtime_metadata_insecure_url = "wss://kusama-rpc.dwellir.com:443",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod kusama_relay {}
pub(super) use kusama_relay::runtime_types::staging_kusama_runtime::{
	governance::origins::pallet_custom_origins::Origin as KusamaOpenGovOrigin,
	OriginCaller as KusamaOriginCaller, RuntimeCall as KusamaRuntimeCall,
};

#[subxt::subxt(runtime_metadata_insecure_url = "wss://kusama-asset-hub-rpc.polkadot.io:443")]
pub mod kusama_asset_hub {}
pub(super) use kusama_asset_hub::runtime_types::asset_hub_kusama_runtime::RuntimeCall as KusamaAssetHubRuntimeCall;

#[subxt::subxt(runtime_metadata_insecure_url = "wss://kusama-bridge-hub-rpc.polkadot.io:443")]
pub mod kusama_bridge_hub {}
pub(super) use kusama_bridge_hub::runtime_types::bridge_hub_kusama_runtime::RuntimeCall as KusamaBridgeHubRuntimeCall;

#[subxt::subxt(runtime_metadata_insecure_url = "wss://kusama.api.encointer.org:443")]
pub mod kusama_encointer {}
pub(super) use kusama_encointer::runtime_types::encointer_runtime::RuntimeCall as KusamaEncointerRuntimeCall;

#[subxt::subxt(runtime_metadata_insecure_url = "wss://kusama-coretime-rpc.polkadot.io:443")]
pub mod kusama_coretime {}
pub(super) use kusama_coretime::runtime_types::coretime_kusama_runtime::RuntimeCall as KusamaCoretimeRuntimeCall;

// Polkadot Chains ---------------------------------------------------------------------------------

#[subxt::subxt(
	runtime_metadata_insecure_url = "wss://polkadot-rpc.dwellir.com:443",
	derive_for_all_types = "PartialEq, Clone"
)]
pub mod polkadot_relay {}
pub(super) use polkadot_relay::runtime_types::polkadot_runtime::{
	governance::origins::pallet_custom_origins::Origin as PolkadotOpenGovOrigin,
	OriginCaller as PolkadotOriginCaller, RuntimeCall as PolkadotRuntimeCall,
};

#[subxt::subxt(runtime_metadata_insecure_url = "wss://polkadot-asset-hub-rpc.polkadot.io:443")]
pub mod polkadot_asset_hub {}
pub(super) use polkadot_asset_hub::runtime_types::asset_hub_polkadot_runtime::RuntimeCall as PolkadotAssetHubRuntimeCall;

#[subxt::subxt(runtime_metadata_insecure_url = "wss://polkadot-collectives-rpc.polkadot.io:443")]
pub mod polkadot_collectives {}
pub(super) use polkadot_collectives::runtime_types::{
	collectives_polkadot_runtime::{
		fellowship::origins::pallet_origins::Origin as FellowshipOrigins,
		RuntimeCall as CollectivesRuntimeCall,
	},
	sp_weights::weight_v2::Weight,
};

#[subxt::subxt(runtime_metadata_insecure_url = "wss://polkadot-bridge-hub-rpc.polkadot.io:443")]
pub mod polkadot_bridge_hub {}
pub(super) use polkadot_bridge_hub::runtime_types::bridge_hub_polkadot_runtime::RuntimeCall as PolkadotBridgeHubRuntimeCall;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Network {
	Kusama,
	KusamaAssetHub,
	KusamaBridgeHub,
	KusamaCoretime,
	KusamaEncointer,
	Polkadot,
	PolkadotAssetHub,
	PolkadotCollectives,
	PolkadotBridgeHub,
}

impl Network {
	pub(super) fn get_para_id(&self) -> Result<u32, &'static str> {
		use Network::*;
		match &self {
			Kusama => Err("relay chain"),
			KusamaAssetHub => Ok(1_000),
			KusamaBridgeHub => Ok(1_002),
			KusamaCoretime => Ok(1_005),
			KusamaEncointer => Ok(1_001),
			Polkadot => Err("relay chain"),
			PolkadotAssetHub => Ok(1_000),
			PolkadotCollectives => Ok(1_001),
			PolkadotBridgeHub => Ok(1_002),
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
	// `Some` if you want to manually set the `require_weight_at_most` parameter used in any
	// `Transact` instruction. If `None`, then the program will fetch the required weight (plus a 2x
	// factor of safety) and construct the instruction with that.
	pub(super) transact_weight_override: Option<Weight>,
}

// Info and preferences provided by the user for runtime upgrade construction.
pub(super) struct UpgradeDetails {
	// The Relay Network for this upgrade, Polkadot or Kusama.
	pub(super) relay: Network,
	// The version of the Relay Chain to which to upgrade. Typically, but not always, the default.
	pub(super) relay_version: Option<String>,
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
	Kusama(KusamaOpenGovOrigin),
	PolkadotRoot,
	Polkadot(PolkadotOpenGovOrigin),
}

// A runtime call wrapped in the network it should execute on.
pub(super) enum NetworkRuntimeCall {
	Kusama(KusamaRuntimeCall),
	KusamaAssetHub(KusamaAssetHubRuntimeCall),
	KusamaBridgeHub(KusamaBridgeHubRuntimeCall),
	KusamaCoretime(KusamaCoretimeRuntimeCall),
	KusamaEncointer(KusamaEncointerRuntimeCall),
	Polkadot(PolkadotRuntimeCall),
	PolkadotAssetHub(PolkadotAssetHubRuntimeCall),
	PolkadotCollectives(CollectivesRuntimeCall),
	PolkadotBridgeHub(PolkadotBridgeHubRuntimeCall),
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
			NetworkRuntimeCall::KusamaCoretime(cc) => (Network::KusamaCoretime, cc.encode()),
			NetworkRuntimeCall::KusamaEncointer(cc) => (Network::KusamaEncointer, cc.encode()),
			NetworkRuntimeCall::Polkadot(cc) => (Network::Polkadot, cc.encode()),
			NetworkRuntimeCall::PolkadotAssetHub(cc) => (Network::PolkadotAssetHub, cc.encode()),
			NetworkRuntimeCall::PolkadotCollectives(cc) =>
				(Network::PolkadotCollectives, cc.encode()),
			NetworkRuntimeCall::PolkadotBridgeHub(cc) => (Network::PolkadotBridgeHub, cc.encode()),
		};
		let hash = blake2_256(&encoded);
		let length: u32 = (encoded.len()).try_into().unwrap();
		Self { network, encoded: encoded.to_vec(), hash, length }
	}

	// Construct `Self` for some `network` given some `encoded` bytes.
	pub(super) fn from_bytes(encoded: &Vec<u8>, network: Network) -> Self {
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

	pub(super) async fn get_transact_weight_needed(
		&self,
		network: &Network,
		fallback_weight: Weight,
	) -> Weight {
		// `PolkadotConfig` is a bit confusing. It should work across everything. It contains
		// basic types like `Nonce`, etc.
		use subxt::{OnlineClient, PolkadotConfig};

		let url = match network {
			Network::Kusama => "wss://kusama-rpc.dwellir.com:443",
			Network::KusamaAssetHub => "wss://kusama-asset-hub-rpc.polkadot.io:443",
			Network::KusamaBridgeHub => "wss://kusama-bridge-hub-rpc.polkadot.io:443",
			Network::KusamaCoretime => "wss://kusama-coretime-rpc.polkadot.io:443",
			Network::KusamaEncointer => "wss://kusama.api.encointer.org:443",
			Network::Polkadot => "wss://polkadot-rpc.dwellir.com:443",
			Network::PolkadotAssetHub => "wss://polkadot-asset-hub-rpc.polkadot.io:443",
			Network::PolkadotCollectives => "wss://polkadot-collectives-rpc.polkadot.io:443",
			Network::PolkadotBridgeHub => "wss://polkadot-bridge-hub-rpc.polkadot.io:443",
		};

		let mut args = self.encoded.clone();
		self.length.encode_to(&mut args);

		let relay_api = OnlineClient::<PolkadotConfig>::from_url(url).await.expect("an api");
		let runtime_apis = relay_api.runtime_api().at_latest().await.expect("latest block");
		let (weight_needed, _, _): (Weight, u8, u128) = runtime_apis
			.call_raw("TransactionPaymentCallApi_query_call_info", Some(&args))
			.await
			.unwrap_or((fallback_weight, 0u8, 0u128));
		weight_needed
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
				Network::Polkadot => {
					let polkadot_call = self.get_polkadot_call().expect("polkadot");
					CallOrHash::Call(NetworkRuntimeCall::Polkadot(polkadot_call))
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
