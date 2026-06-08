use crate::{kusama_asset_hub, polkadot_asset_hub};
use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use std::fs;
use subxt::{OnlineClient, PolkadotConfig};

const BLOCK_TIME_MILLIS: u64 = 6_000; // Polkadot/Kusama block time in milliseconds

#[derive(Debug)]
pub struct TimeConversionError(String);

impl std::fmt::Display for TimeConversionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::error::Error for TimeConversionError {}

// Check what the user entered for the proposal. If it is just call data, return it back. Otherwise,
// we expect a path to a file that contains the call data. Read that in and return it.
pub(crate) fn get_proposal_bytes(proposal: String) -> Vec<u8> {
	let proposal = proposal.as_str();
	if proposal.starts_with("0x") {
		// This is just call data
		hex::decode(proposal.trim_start_matches("0x")).expect("Valid proposal")
	} else {
		// This is a file path
		let contents = fs::read_to_string(proposal).expect("Should give a valid file path");
		hex::decode(contents.as_str().trim_start_matches("0x").trim()).expect("Valid proposal")
	}
}

/// Convert a wall clock time string to a block number on Asset Hub.
/// Format: YY-MM-DDThhmm (e.g. 25-05-21T0800 for 21st May 2025 at 08:00)
pub(crate) async fn wall_clock_to_block_number(
	time_str: &str,
	network: &str,
) -> Result<u32, TimeConversionError> {
	// Parse the input time string
	let naive_dt = NaiveDateTime::parse_from_str(time_str, "%y-%m-%dT%H%M")
		.map_err(|e| TimeConversionError(format!("Invalid time format: {}", e)))?;

	// Convert to UTC
	let local_dt = Local
		.from_local_datetime(&naive_dt)
		.single()
		.ok_or_else(|| TimeConversionError("Invalid local time".to_string()))?;
	let utc_dt = local_dt.with_timezone(&Utc);

	// Connect to Asset Hub (where referenda are submitted post-AHM)
	let url = match network.to_lowercase().as_str() {
		"polkadot" => "wss://asset-hub-polkadot-rpc.dwellir.com:443",
		"kusama" => "wss://asset-hub-kusama-rpc.dwellir.com:443",
		_ => return Err(TimeConversionError("Invalid network".to_string())),
	};

	let api = OnlineClient::<PolkadotConfig>::from_url(url)
		.await
		.map_err(|e| TimeConversionError(format!("Failed to connect to node: {}", e)))?;

	let current_block = api
		.blocks()
		.at_latest()
		.await
		.map_err(|e| TimeConversionError(format!("Failed to get latest block: {}", e)))?;

	let current_block_number = current_block.number();
	let current_timestamp = api
		.storage()
		.at(current_block.hash())
		.fetch(&match network.to_lowercase().as_str() {
			"polkadot" => polkadot_asset_hub::storage().timestamp().now(),
			"kusama" => kusama_asset_hub::storage().timestamp().now(),
			_ => return Err(TimeConversionError("Invalid network".to_string())),
		})
		.await
		.map_err(|e| TimeConversionError(format!("Failed to get block timestamp: {}", e)))?
		.ok_or_else(|| TimeConversionError("Failed to get timestamp".to_string()))?;

	// Calculate time difference in milliseconds
	let target_timestamp = utc_dt.timestamp_millis() as u64;
	let time_diff_ms = if target_timestamp > current_timestamp {
		target_timestamp - current_timestamp
	} else {
		return Err(TimeConversionError("Target time is in the past".to_string()));
	};

	// Calculate block difference (rounding up)
	let block_diff = (time_diff_ms + BLOCK_TIME_MILLIS - 1) / BLOCK_TIME_MILLIS;

	Ok(current_block_number + block_diff as u32)
}
