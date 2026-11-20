use crate::Error;
use parity_scale_codec::Decode;
pub(crate) use parity_scale_codec::Encode as _;
pub use sp_crypto_hashing::blake2_256;
pub(crate) use subxt_core::utils::H256;

// Kusama Chains -----------------------------------------------------------------------------------

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/kusama.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub(crate) mod kusama_relay {}
pub(crate) use kusama_relay::runtime_types::staging_kusama_runtime::{
	governance::origins::pallet_custom_origins::Origin as KusamaOpenGovOrigin,
	OriginCaller as KusamaOriginCaller, RuntimeCall as KusamaRuntimeCall,
};

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/kusama_asset_hub.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub(crate) mod kusama_asset_hub {}
// The generated types are internal, except for the OpenGov origins (tracks) of the Asset Hubs, which
// `NetworkTrack` is made of.
pub use kusama_asset_hub::runtime_types::asset_hub_kusama_runtime::governance::origins::pallet_custom_origins::Origin as KusamaAssetHubOpenGovOrigin;
pub(crate) use kusama_asset_hub::runtime_types::asset_hub_kusama_runtime::{
	OriginCaller as KusamaAssetHubOriginCaller, RuntimeCall as KusamaAssetHubRuntimeCall,
};

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/kusama_bridge_hub.scale"
)]
pub(crate) mod kusama_bridge_hub {}
pub(crate) use kusama_bridge_hub::runtime_types::bridge_hub_kusama_runtime::RuntimeCall as KusamaBridgeHubRuntimeCall;

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/kusama_encointer.scale"
)]
pub(crate) mod kusama_encointer {}
pub(crate) use kusama_encointer::runtime_types::encointer_kusama_runtime::RuntimeCall as KusamaEncointerRuntimeCall;

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/kusama_people.scale"
)]
pub(crate) mod kusama_people {}
pub(crate) use kusama_people::runtime_types::people_kusama_runtime::RuntimeCall as KusamaPeopleRuntimeCall;

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/kusama_coretime.scale"
)]
pub(crate) mod kusama_coretime {}
pub(crate) use kusama_coretime::runtime_types::coretime_kusama_runtime::RuntimeCall as KusamaCoretimeRuntimeCall;

// Polkadot Chains ---------------------------------------------------------------------------------

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/polkadot.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub(crate) mod polkadot_relay {}
pub(crate) use polkadot_relay::runtime_types::polkadot_runtime::RuntimeCall as PolkadotRuntimeCall;

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/polkadot_asset_hub.scale",
	derive_for_all_types = "PartialEq, Clone"
)]
pub(crate) mod polkadot_asset_hub {}
pub use polkadot_asset_hub::runtime_types::asset_hub_polkadot_runtime::governance::origins::pallet_custom_origins::Origin as PolkadotAssetHubOpenGovOrigin;
pub(crate) use polkadot_asset_hub::runtime_types::asset_hub_polkadot_runtime::{
	OriginCaller as PolkadotAssetHubOriginCaller, RuntimeCall as PolkadotAssetHubRuntimeCall,
};

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/polkadot_collectives.scale"
)]
pub(crate) mod polkadot_collectives {}
pub(crate) use polkadot_collectives::runtime_types::collectives_polkadot_runtime::{
	fellowship::origins::pallet_origins::Origin as FellowshipOrigins,
	RuntimeCall as CollectivesRuntimeCall,
};

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/polkadot_bridge_hub.scale"
)]
pub(crate) mod polkadot_bridge_hub {}
pub(crate) use polkadot_bridge_hub::runtime_types::bridge_hub_polkadot_runtime::RuntimeCall as PolkadotBridgeHubRuntimeCall;

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/polkadot_people.scale"
)]
pub(crate) mod polkadot_people {}
pub(crate) use polkadot_people::runtime_types::people_polkadot_runtime::RuntimeCall as PolkadotPeopleRuntimeCall;

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/polkadot_coretime.scale"
)]
pub(crate) mod polkadot_coretime {}
pub(crate) use polkadot_coretime::runtime_types::coretime_polkadot_runtime::RuntimeCall as PolkadotCoretimeRuntimeCall;

#[subxt_macro::subxt(
	crate = "::subxt_core",
	runtime_metadata_path = "metadata/polkadot_bulletin.scale"
)]
pub(crate) mod polkadot_bulletin {}
pub(crate) use polkadot_bulletin::runtime_types::bulletin_polkadot_runtime::RuntimeCall as PolkadotBulletinRuntimeCall;

/// The chains this crate can build calls for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Network {
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
	PolkadotBulletin,
}

impl Network {
	/// The name a Fellowship release gives this chain's runtime, e.g. `asset-hub-polkadot`. Also
	/// the stable string id of the network.
	pub fn runtime_name(&self) -> &'static str {
		match self {
			Network::Kusama => "kusama",
			Network::KusamaAssetHub => "asset-hub-kusama",
			Network::KusamaBridgeHub => "bridge-hub-kusama",
			Network::KusamaPeople => "people-kusama",
			Network::KusamaCoretime => "coretime-kusama",
			Network::KusamaEncointer => "encointer-kusama",
			Network::Polkadot => "polkadot",
			Network::PolkadotAssetHub => "asset-hub-polkadot",
			Network::PolkadotCollectives => "collectives-polkadot",
			Network::PolkadotBridgeHub => "bridge-hub-polkadot",
			Network::PolkadotPeople => "people-polkadot",
			Network::PolkadotCoretime => "coretime-polkadot",
			Network::PolkadotBulletin => "bulletin-polkadot",
		}
	}

	/// The inverse of [`Network::runtime_name`].
	pub fn from_runtime_name(name: &str) -> Result<Self, Error> {
		Self::ALL
			.iter()
			.find(|network| network.runtime_name() == name)
			.cloned()
			.ok_or_else(|| Error::UnknownNetwork(name.to_string()))
	}

	/// Every supported network.
	pub const ALL: [Network; 13] = [
		Network::Kusama,
		Network::KusamaAssetHub,
		Network::KusamaEncointer,
		Network::KusamaBridgeHub,
		Network::KusamaPeople,
		Network::KusamaCoretime,
		Network::Polkadot,
		Network::PolkadotAssetHub,
		Network::PolkadotCollectives,
		Network::PolkadotBridgeHub,
		Network::PolkadotPeople,
		Network::PolkadotCoretime,
		Network::PolkadotBulletin,
	];

	/// Return the `ParaId` of a given network. Returns an error if the network is not a parachain.
	pub fn get_para_id(&self) -> Result<u32, &'static str> {
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
			PolkadotBulletin => Ok(1_010),
		}
	}

	/// Whether the network belongs to the Kusama family (as opposed to Polkadot).
	pub fn is_kusama(&self) -> bool {
		use Network::*;
		matches!(
			self,
			Kusama
				| KusamaAssetHub
				| KusamaEncointer
				| KusamaBridgeHub
				| KusamaPeople
				| KusamaCoretime
		)
	}

	/// The SS58 prefix used to display accounts of this network.
	pub fn ss58_prefix(&self) -> u16 {
		if self.is_kusama() {
			2
		} else {
			0
		}
	}
}

/// The network and OpenGov track a proposal should be voted on.
#[derive(Clone)]
pub enum NetworkTrack {
	KusamaRoot,
	Kusama(KusamaAssetHubOpenGovOrigin),
	PolkadotRoot,
	Polkadot(PolkadotAssetHubOpenGovOrigin),
}

// A runtime call wrapped in the network it should execute on.
pub(crate) enum NetworkRuntimeCall {
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
	PolkadotBulletin(PolkadotBulletinRuntimeCall),
}

/// When a referendum should enact once approved: at a block number or after a number of blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchTimeWrapper {
	At(u32),
	After(u32),
}

/// A call together with the chain it must be submitted to or executed on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallInfo {
	/// The chain whose `RuntimeCall` this is.
	pub network: Network,
	/// The SCALE-encoded `RuntimeCall`.
	pub encoded: Vec<u8>,
	/// `blake2_256(encoded)`.
	pub hash: [u8; 32],
	/// The length of `encoded` in bytes.
	pub length: u32,
}

// Define a method that decodes a `CallInfo` as the `RuntimeCall` of the network it is for.
macro_rules! call_getter {
	($name:ident, $network:ident, $call:ty) => {
		#[allow(dead_code)]
		pub(crate) fn $name(&self) -> Result<$call, Error> {
			match &self.network {
				Network::$network => <$call as Decode>::decode(&mut &self.encoded[..])
					.map_err(|_| Error::UndecodableCall(Network::$network)),
				other =>
					Err(Error::WrongNetwork { expected: Network::$network, found: other.clone() }),
			}
		}
	};
}

impl CallInfo {
	// Construct `Self` from a `NetworkRuntimeCall`.
	pub(crate) fn from_runtime_call(call: NetworkRuntimeCall) -> Self {
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
			NetworkRuntimeCall::PolkadotBulletin(cc) => (Network::PolkadotBulletin, cc.encode()),
		};
		let hash = blake2_256(&encoded);
		let length: u32 = (encoded.len()).try_into().unwrap();
		Self { network, encoded: encoded.to_vec(), hash, length }
	}

	/// Construct `Self` for some `network` given some `encoded` bytes.
	pub fn from_bytes(encoded: &[u8], network: Network) -> Self {
		let hash = blake2_256(encoded);
		let length = (encoded.len()).try_into().unwrap();
		Self { network, encoded: encoded.to_vec(), hash, length }
	}

	call_getter!(
		// Decode as a Kusama Relay Chain `RuntimeCall`.
		get_kusama_call,
		Kusama,
		KusamaRuntimeCall
	);
	call_getter!(
		// Decode as a Kusama Asset Hub `RuntimeCall`.
		get_kusama_asset_hub_call,
		KusamaAssetHub,
		KusamaAssetHubRuntimeCall
	);
	call_getter!(
		// Decode as a Kusama Bridge Hub `RuntimeCall`.
		get_kusama_bridge_hub_call,
		KusamaBridgeHub,
		KusamaBridgeHubRuntimeCall
	);
	call_getter!(
		// Decode as a Kusama Encointer `RuntimeCall`.
		get_kusama_encointer_call,
		KusamaEncointer,
		KusamaEncointerRuntimeCall
	);
	call_getter!(
		// Decode as a Kusama People `RuntimeCall`.
		get_kusama_people_call,
		KusamaPeople,
		KusamaPeopleRuntimeCall
	);
	call_getter!(
		// Decode as a Kusama Coretime `RuntimeCall`.
		get_kusama_coretime_call,
		KusamaCoretime,
		KusamaCoretimeRuntimeCall
	);
	call_getter!(
		// Decode as a Polkadot Relay Chain `RuntimeCall`.
		get_polkadot_call,
		Polkadot,
		PolkadotRuntimeCall
	);
	call_getter!(
		// Decode as a Polkadot Asset Hub `RuntimeCall`.
		get_polkadot_asset_hub_call,
		PolkadotAssetHub,
		PolkadotAssetHubRuntimeCall
	);
	call_getter!(
		// Decode as a Polkadot Collectives `RuntimeCall`.
		get_polkadot_collectives_call,
		PolkadotCollectives,
		CollectivesRuntimeCall
	);
	call_getter!(
		// Decode as a Polkadot Bridge Hub `RuntimeCall`.
		get_polkadot_bridge_hub_call,
		PolkadotBridgeHub,
		PolkadotBridgeHubRuntimeCall
	);
	call_getter!(
		// Decode as a Polkadot People `RuntimeCall`.
		get_polkadot_people_call,
		PolkadotPeople,
		PolkadotPeopleRuntimeCall
	);
	call_getter!(
		// Decode as a Polkadot Coretime `RuntimeCall`.
		get_polkadot_coretime_call,
		PolkadotCoretime,
		PolkadotCoretimeRuntimeCall
	);
	call_getter!(
		// Decode as a Polkadot Bulletin `RuntimeCall`.
		get_polkadot_bulletin_call,
		PolkadotBulletin,
		PolkadotBulletinRuntimeCall
	);
}
