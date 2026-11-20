//! JavaScript bindings for the library.
//!
//! Inputs and outputs are plain objects (see the TypeScript definitions below). Bytes are
//! `0x`-prefixed hex strings, networks are named as in Fellowship releases (e.g.
//! `asset-hub-polkadot`), and errors are thrown as JavaScript `Error`s carrying the message.

use crate::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
/** A call and the chain it must be submitted to or executed on. */
export interface NetworkCall {
	/** Runtime name of the chain, e.g. "asset-hub-polkadot". */
	network: string;
	/** SCALE-encoded `RuntimeCall`, as 0x-prefixed hex. */
	encoded: string;
	/** blake2_256 of `encoded`, as 0x-prefixed hex. */
	hash: string;
}

export interface UpgradeInput {
	/** "polkadot" or "kusama". */
	relay: string;
	/** Each chain to upgrade and the hash of its new runtime, in batch order. */
	runtimes: { network: string; code_hash: string }[];
	/** An extra, encoded Asset Hub call to execute in the same batch. */
	additional?: string;
}

export interface ReferendumInput {
	/** The encoded proposal: an Asset Hub call. */
	proposal: string;
	/** "polkadot" or "kusama". */
	network: string;
	/** Track name, e.g. "root" or "whitelistedcaller". */
	track: string;
	/** Enact after a number of blocks, or at a block number. */
	dispatch: { after: number } | { at: number };
	/** Kusama `whitelistedcaller` only: have the Polkadot Fellowship whitelist the call. */
	fellowship_on_polkadot?: boolean;
}

export interface ReferendumCalls {
	/** `preimage.note_preimage` for the Fellowship proposal, when it is too long to inline. */
	fellowship_preimage: NetworkCall | null;
	/** `fellowshipReferenda.submit`, on a whitelisted track. */
	fellowship_referendum: NetworkCall | null;
	/** `preimage.note_preimage` for the public referendum's proposal. */
	public_preimage: NetworkCall;
	/** `referenda.submit`. */
	public_referendum: NetworkCall;
	/** The bytes that `public_preimage` notes. */
	public_preimage_bytes: string;
	/** Hash of `public_preimage_bytes`, which the public referendum refers to. */
	public_preimage_hash: string;
	/** Length of `public_preimage_bytes` in bytes. */
	public_preimage_len: number;
	/** The hash the Fellowship whitelists, when a whitelist is involved. */
	whitelisted_call_hash: string | null;
}
"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpgradeRequest {
	relay: String,
	runtimes: Vec<RuntimeRequest>,
	#[serde(default)]
	additional: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeRequest {
	network: String,
	code_hash: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReferendumRequest {
	proposal: String,
	network: String,
	track: String,
	dispatch: DispatchRequest,
	#[serde(default)]
	fellowship_on_polkadot: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum DispatchRequest {
	At(u32),
	After(u32),
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct CallResponse {
	network: &'static str,
	encoded: String,
	hash: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ReferendumResponse {
	fellowship_preimage: Option<CallResponse>,
	fellowship_referendum: Option<CallResponse>,
	public_preimage: CallResponse,
	public_referendum: CallResponse,
	public_preimage_bytes: String,
	public_preimage_hash: String,
	public_preimage_len: u32,
	whitelisted_call_hash: Option<String>,
}

fn to_hex(bytes: &[u8]) -> String {
	format!("0x{}", hex::encode(bytes))
}

fn from_hex(input: &str) -> Result<Vec<u8>, Error> {
	hex::decode(input.trim().trim_start_matches("0x"))
		.map_err(|_| Error::InvalidHex(input.to_string()))
}

fn hash_from_hex(input: &str) -> Result<[u8; 32], Error> {
	let bytes = from_hex(input)?;
	bytes.as_slice().try_into().map_err(|_| Error::InvalidHashLength(bytes.len()))
}

impl From<CallInfo> for CallResponse {
	fn from(call: CallInfo) -> Self {
		Self {
			network: call.network.runtime_name(),
			encoded: to_hex(&call.encoded),
			hash: to_hex(&call.hash),
		}
	}
}

// The relay chain is given by its runtime name, which for relay chains is also the name the CLI
// takes as `--network`.
fn relay_from_name(name: &str) -> Result<Network, Error> {
	match Network::from_runtime_name(name) {
		Ok(relay @ (Network::Polkadot | Network::Kusama)) => Ok(relay),
		_ => Err(Error::NotARelayChain),
	}
}

pub(crate) fn build_upgrade_request(request: UpgradeRequest) -> Result<CallResponse, Error> {
	let runtimes = request
		.runtimes
		.iter()
		.map(|runtime| {
			Ok((Network::from_runtime_name(&runtime.network)?, hash_from_hex(&runtime.code_hash)?))
		})
		.collect::<Result<Vec<_>, Error>>()?;
	let input = UpgradeInput {
		relay: relay_from_name(&request.relay)?,
		runtimes,
		additional: request.additional.as_deref().map(from_hex).transpose()?,
	};
	crate::build_upgrade(input).map(Into::into)
}

pub(crate) fn referendum_calls_request(
	request: ReferendumRequest,
) -> Result<ReferendumResponse, Error> {
	let input = ReferendumInput {
		proposal: from_hex(&request.proposal)?,
		track: NetworkTrack::parse(&request.network, &request.track)?,
		dispatch: match request.dispatch {
			DispatchRequest::At(block) => DispatchTimeWrapper::At(block),
			DispatchRequest::After(blocks) => DispatchTimeWrapper::After(blocks),
		},
		fellowship_on_polkadot: request.fellowship_on_polkadot,
	};
	let calls = crate::referendum_calls(input)?;
	Ok(ReferendumResponse {
		fellowship_preimage: calls.fellowship_preimage.map(Into::into),
		fellowship_referendum: calls.fellowship_referendum.map(Into::into),
		public_preimage: calls.public_preimage.into(),
		public_referendum: calls.public_referendum.into(),
		public_preimage_bytes: to_hex(&calls.public_preimage_bytes),
		public_preimage_hash: to_hex(&calls.public_preimage_hash),
		public_preimage_len: calls.public_preimage_len,
		whitelisted_call_hash: calls.whitelisted_call_hash.map(|hash| to_hex(&hash)),
	})
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
	// Absent values are `null`, not `undefined`, as in the TypeScript definitions.
	let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);
	Ok(value.serialize(&serializer)?)
}

fn js_error(error: Error) -> JsError {
	JsError::new(&error.to_string())
}

/// Build the proposal that upgrades a relay chain and its system parachains: an
/// `authorize_upgrade` per chain, sent by XCM where needed, in a `force_batch`.
#[wasm_bindgen(js_name = build_upgrade, unchecked_return_type = "NetworkCall")]
pub fn build_upgrade_js(
	#[wasm_bindgen(unchecked_param_type = "UpgradeInput")] input: JsValue,
) -> Result<JsValue, JsError> {
	let request: UpgradeRequest = serde_wasm_bindgen::from_value(input)?;
	to_js(&build_upgrade_request(request).map_err(js_error)?)
}

/// Build every call needed to put a proposal to referendum.
#[wasm_bindgen(js_name = referendum_calls, unchecked_return_type = "ReferendumCalls")]
pub fn referendum_calls_js(
	#[wasm_bindgen(unchecked_param_type = "ReferendumInput")] input: JsValue,
) -> Result<JsValue, JsError> {
	let request: ReferendumRequest = serde_wasm_bindgen::from_value(input)?;
	to_js(&referendum_calls_request(request).map_err(js_error)?)
}

/// File name of a chain's runtime in a Fellowship release. `network` is a runtime name such as
/// `asset-hub-polkadot`, `version` the release's semver.
#[wasm_bindgen(js_name = runtime_file_name)]
pub fn runtime_file_name_js(network: &str, version: &str) -> Result<String, JsError> {
	let network = Network::from_runtime_name(network).map_err(js_error)?;
	crate::runtime_file_name(&network, version).map_err(js_error)
}

/// Check that `bytes` looks like a compressed Substrate runtime (unless `skip_checks`) and return
/// its code hash as 0x-prefixed hex.
#[wasm_bindgen(js_name = check_and_hash_runtime)]
pub fn check_and_hash_runtime_js(bytes: &[u8], skip_checks: bool) -> Result<String, JsError> {
	crate::check_and_hash_runtime(bytes, skip_checks)
		.map(|hash| to_hex(&hash))
		.map_err(|error| JsError::new(&format!("Runtime is {error}")))
}

// The bindings themselves only run in a JavaScript host, but everything between the parsed
// request and the response to serialize is plain Rust and is tested here.
#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::{json, Value};

	fn upgrade(request: Value) -> Result<Value, String> {
		let request: UpgradeRequest = serde_json::from_value(request).map_err(|e| e.to_string())?;
		let response = build_upgrade_request(request).map_err(|e| e.to_string())?;
		Ok(serde_json::to_value(response).expect("serializable"))
	}

	fn referendum(request: Value) -> Result<Value, String> {
		let request: ReferendumRequest =
			serde_json::from_value(request).map_err(|e| e.to_string())?;
		let response = referendum_calls_request(request).map_err(|e| e.to_string())?;
		Ok(serde_json::to_value(response).expect("serializable"))
	}

	#[test]
	fn upgrade_request_gives_the_library_result() {
		let relay_hash = [1u8; 32];
		let asset_hub_hash = [2u8; 32];
		let expected = crate::build_upgrade(UpgradeInput {
			relay: Network::Polkadot,
			runtimes: vec![
				(Network::Polkadot, relay_hash),
				(Network::PolkadotAssetHub, asset_hub_hash),
			],
			// `system.remark("test")`
			additional: Some(hex::decode("00001074657374").expect("hex")),
		})
		.expect("valid upgrade");

		let response = upgrade(json!({
			"relay": "polkadot",
			"runtimes": [
				{ "network": "polkadot", "code_hash": to_hex(&relay_hash) },
				{ "network": "asset-hub-polkadot", "code_hash": to_hex(&asset_hub_hash) },
			],
			"additional": "0x00001074657374",
		}));
		assert_eq!(
			response,
			Ok(json!({
				"network": "asset-hub-polkadot",
				"encoded": to_hex(&expected.encoded),
				"hash": to_hex(&expected.hash),
			}))
		);
	}

	#[test]
	fn upgrade_request_reports_errors() {
		let request = |relay: &str, network: &str, code_hash: &str| {
			upgrade(json!({
				"relay": relay,
				"runtimes": [{ "network": network, "code_hash": code_hash }],
			}))
			.expect_err("invalid request")
		};
		let hash = to_hex(&[1u8; 32]);
		assert_eq!(
			request("asset-hub-polkadot", "polkadot", &hash),
			"`network` must be `polkadot` or `kusama`"
		);
		assert_eq!(request("polkadot", "westend", &hash), "`westend` is not a supported network");
		assert_eq!(
			request("polkadot", "polkadot", "0x1234"),
			"expected a 32 byte hash, got 2 bytes"
		);
		assert_eq!(request("polkadot", "polkadot", "0xzz"), "`0xzz` is not valid hex");
		assert_eq!(
			request("polkadot", "kusama", &hash),
			"`kusama` is not upgraded by `polkadot` governance"
		);
		assert!(upgrade(json!({ "relay": "polkadot", "runtimes": [], "extra": 1 })).is_err());
	}

	#[test]
	fn referendum_request_gives_the_library_result() {
		// `system.remark("opengov-submit test")`
		let proposal = "0x00004c6f70656e676f762d7375626d69742074657374";
		let response = referendum(json!({
			"proposal": proposal,
			"network": "polkadot",
			"track": "whitelistedcaller",
			"dispatch": { "after": 10 },
		}));
		// The encodings are those asserted for the library and the command line tool.
		assert_eq!(
			response,
			Ok(json!({
				"fellowship_preimage": null,
				"fellowship_referendum": {
					"network": "collectives-polkadot",
					"encoded": "0x3d003e0201cc1f0005010100a10f05082f00000603008840008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534010a000000",
					"hash": to_hex(&blake2_256(&hex::decode("3d003e0201cc1f0005010100a10f05082f00000603008840008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534010a000000").expect("hex"))),
				},
				"public_preimage": {
					"network": "asset-hub-polkadot",
					"encoded": "0x050060400300004c6f70656e676f762d7375626d69742074657374",
					"hash": to_hex(&blake2_256(&hex::decode("050060400300004c6f70656e676f762d7375626d69742074657374").expect("hex"))),
				},
				"public_referendum": {
					"network": "asset-hub-polkadot",
					"encoded": "0x3e003f0d02a322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e18000000010a000000",
					"hash": to_hex(&blake2_256(&hex::decode("3e003f0d02a322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e18000000010a000000").expect("hex"))),
				},
				"public_preimage_bytes": "0x400300004c6f70656e676f762d7375626d69742074657374",
				"public_preimage_hash": "0xa322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e",
				"public_preimage_len": 24,
				"whitelisted_call_hash": "0x8821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534",
			}))
		);

		// `at` is the other way to give the enactment time.
		let at = referendum(json!({
			"proposal": proposal,
			"network": "polkadot",
			"track": "root",
			"dispatch": { "at": 34030500 },
		}))
		.expect("valid request");
		let expected = crate::referendum_calls(ReferendumInput {
			proposal: from_hex(proposal).expect("hex"),
			track: NetworkTrack::PolkadotRoot,
			dispatch: DispatchTimeWrapper::At(34030500),
			fellowship_on_polkadot: false,
		})
		.expect("valid referendum");
		assert_eq!(at["public_referendum"]["encoded"], to_hex(&expected.public_referendum.encoded));
		assert_eq!(at["fellowship_referendum"], Value::Null);
		assert_eq!(at["whitelisted_call_hash"], Value::Null);
	}

	#[test]
	fn referendum_request_reports_errors() {
		let request = |proposal: &str, network: &str, track: &str, dispatch: Value| {
			referendum(json!({
				"proposal": proposal,
				"network": network,
				"track": track,
				"dispatch": dispatch,
			}))
			.expect_err("invalid request")
		};
		let after = json!({ "after": 10 });
		assert_eq!(
			request("0x0000", "polkadot", "nonsense", after.clone()),
			"Unsupported track! Tracks should be in the form `general-admin` or `generaladmin`."
		);
		assert_eq!(
			request("0x0000", "westend", "root", after.clone()),
			"`network` must be `polkadot` or `kusama`"
		);
		assert_eq!(
			request("0xffffff", "polkadot", "whitelistedcaller", after.clone()),
			"the bytes do not decode as a `asset-hub-polkadot` call"
		);
		assert_eq!(request("nope", "polkadot", "root", after), "`nope` is not valid hex");
		// Exactly one of `at` and `after` must be given.
		request("0x0000", "polkadot", "root", json!({ "at": 1, "after": 2 }));
		request("0x0000", "polkadot", "root", json!({}));
	}
}
