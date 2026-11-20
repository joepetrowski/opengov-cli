use crate::cli::*;
use crate::referendum::{referendum_calls, ReferendumInput};
use crate::*;
use clap::Parser as ClapParser;

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

	/// Use light client endpoints instead of RPC for PAPI links.
	#[clap(long = "light-client")]
	light_client: bool,

	/// Which network's Fellowship to use for whitelisting. Only supported with
	/// `--network kusama --track whitelistedcaller`. Options: `kusama` (default) or `polkadot`.
	#[clap(long = "fellowship")]
	fellowship: Option<String>,
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
	use Output::*;

	let proposal = prefs.proposal;

	let track = NetworkTrack::parse(&prefs.network, &prefs.track).unwrap_or_else(|e| panic!("{e}"));

	let dispatch = match DispatchTimeWrapper::from_at_after(prefs.at, prefs.after) {
		Ok(Some(dispatch)) => dispatch,
		Ok(None) => {
			println!("\nNo enactment time specified. Defaulting to `After(10)`.");
			println!("Specify an enactment time with `--at <block>` or `--after <blocks>`.\n");
			DispatchTimeWrapper::After(10)
		},
		Err(e) => panic!("{e}"),
	};

	let output_len_limit = prefs.output_len_limit.unwrap_or(1_000);

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

	let use_light_client = prefs.light_client;

	let fellowship_on_polkadot = if let Some(ref f) = prefs.fellowship {
		match f.to_ascii_lowercase().as_str() {
			"kusama" => false,
			"polkadot" => true,
			_ => panic!("`--fellowship` must be `kusama` or `polkadot`."),
		}
	} else {
		false
	};

	track.check_fellowship(fellowship_on_polkadot).unwrap_or_else(|e| panic!("{e}"));

	ProposalDetails {
		proposal,
		track,
		dispatch,
		output,
		output_len_limit,
		print_batch,
		use_light_client,
		fellowship_on_polkadot,
	}
}

// Generate all the calls needed, in the form that they will be printed.
pub(crate) async fn generate_calls(proposal_details: &ProposalDetails) -> PossibleCallsToSubmit {
	let calls = referendum_calls(ReferendumInput {
		proposal: get_proposal_bytes(proposal_details.proposal.clone()),
		track: proposal_details.track.clone(),
		dispatch: proposal_details.dispatch,
		fellowship_on_polkadot: proposal_details.fellowship_on_polkadot,
	})
	.unwrap_or_else(|e| panic!("{e}"));

	// Check the lengths and prepare preimages for printing.
	let whitelist_preimage_print =
		calls.fellowship_preimage.map(|p| p.create_print_output(proposal_details.output_len_limit));
	let (public_preimage_print, public_preimage_print_len) =
		calls.public_preimage.create_print_output(proposal_details.output_len_limit);

	// If it's a hash, let's write the data to a file you can upload.
	if matches!(public_preimage_print, CallOrHash::Hash(_)) {
		let fname = match calls.public_preimage.network {
			Network::KusamaAssetHub => "kusama_asset_hub_public_referendum_preimage_to_note.call",
			Network::PolkadotAssetHub =>
				"polkadot_asset_hub_public_referendum_preimage_to_note.call",
			_ => panic!("public referenda are on Asset Hub"),
		};
		write_call_data(fname, &calls.public_preimage_bytes);
	}

	PossibleCallsToSubmit {
		preimage_for_whitelist_call: whitelist_preimage_print,
		preimage_for_public_referendum: Some((public_preimage_print, public_preimage_print_len)),
		fellowship_referendum_submission: calls
			.fellowship_referendum
			.map(|call| call.to_runtime_call()),
		public_referendum_submission: Some(calls.public_referendum.to_runtime_call()),
	}
}

// Takes all the `calls` needed to submit and logs them according to the user's preferences.
fn deliver_output(proposal_details: ProposalDetails, calls: PossibleCallsToSubmit) {
	let mut batch_of_calls = Vec::new();

	if let Some((call_or_hash, len)) = calls.preimage_for_whitelist_call {
		match call_or_hash {
			CallOrHash::Call(c) => {
				println!("\nSubmit the preimage for the Fellowship referendum:");
				print_output(&proposal_details.output, &c, proposal_details.use_light_client);
				batch_of_calls.push(c);
			},
			CallOrHash::Hash(h) => {
				println!(
					"\nPreimage for the public whitelist call too large ({len} bytes). Not included in batch."
				);
				println!("Submission should have the hash: 0x{}", hex::encode(h));
			},
		}
	}
	if let Some(c) = calls.fellowship_referendum_submission {
		println!("\nOpen a Fellowship referendum to whitelist the call:");
		print_output(&proposal_details.output, &c, proposal_details.use_light_client);
		batch_of_calls.push(c);
	}
	if let Some((call_or_hash, len)) = calls.preimage_for_public_referendum {
		match call_or_hash {
			CallOrHash::Call(c) => {
				println!("\nSubmit the preimage for the public referendum:");
				print_output(&proposal_details.output, &c, proposal_details.use_light_client);
				batch_of_calls.push(c);
			},
			CallOrHash::Hash(h) => {
				println!(
					"\nPreimage for the public referendum too large ({len} bytes). Not included in batch."
				);
				println!("A file was created that you can upload in `preimage.note_preimage` in Apps UI.");
				println!("Submission should have the hash: 0x{}", hex::encode(h));
			},
		}
	}
	if let Some(c) = calls.public_referendum_submission {
		println!("\nOpen a public referendum to dispatch the call:");
		print_output(&proposal_details.output, &c, proposal_details.use_light_client);
		batch_of_calls.push(c);
	}

	if proposal_details.print_batch {
		handle_batch_of_calls(
			&proposal_details.output,
			batch_of_calls,
			proposal_details.use_light_client,
		);
	}
}

// Takes a vec of calls, which could be intended for use on different networks, sorts them into the
// appropriate network, and provides a single batch call for each network.
fn handle_batch_of_calls(output: &Output, batch: Vec<NetworkRuntimeCall>, use_light_client: bool) {
	use kusama_asset_hub::runtime_types::pallet_utility::pallet::Call as KusamaAssetHubUtilityCall;
	use kusama_relay::runtime_types::pallet_utility::pallet::Call as KusamaUtilityCall;
	use polkadot_asset_hub::runtime_types::pallet_utility::pallet::Call as PolkadotAssetHubUtilityCall;
	use polkadot_collectives::runtime_types::pallet_utility::pallet::Call as CollectivesUtilityCall;
	use polkadot_relay::runtime_types::pallet_utility::pallet::Call as PolkadotRelayUtilityCall;

	let mut kusama_relay_batch = Vec::new();
	let mut kusama_asset_hub_batch = Vec::new();
	let mut polkadot_asset_hub_batch = Vec::new();
	let mut polkadot_relay_batch = Vec::new();
	let mut polkadot_collectives_batch = Vec::new();

	for network_call in batch {
		match network_call {
			NetworkRuntimeCall::Kusama(cc) => kusama_relay_batch.push(cc),
			NetworkRuntimeCall::KusamaAssetHub(cc) => kusama_asset_hub_batch.push(cc),
			NetworkRuntimeCall::Polkadot(cc) => polkadot_relay_batch.push(cc),
			NetworkRuntimeCall::PolkadotAssetHub(cc) => polkadot_asset_hub_batch.push(cc),
			NetworkRuntimeCall::PolkadotCollectives(cc) => polkadot_collectives_batch.push(cc),
			_ => panic!("no other chains are needed for this"),
		}
	}
	if !kusama_relay_batch.is_empty() {
		let batch = KusamaRuntimeCall::Utility(KusamaUtilityCall::force_batch {
			calls: kusama_relay_batch,
		});
		println!("\nBatch to submit on Kusama Relay Chain:");
		print_output(output, &NetworkRuntimeCall::Kusama(batch), use_light_client);
	}
	if !kusama_asset_hub_batch.is_empty() {
		let batch = KusamaAssetHubRuntimeCall::Utility(KusamaAssetHubUtilityCall::force_batch {
			calls: kusama_asset_hub_batch,
		});
		println!("\nBatch to submit on Kusama Asset Hub:");
		print_output(output, &NetworkRuntimeCall::KusamaAssetHub(batch), use_light_client);
	}
	if !polkadot_relay_batch.is_empty() {
		let batch = PolkadotRuntimeCall::Utility(PolkadotRelayUtilityCall::force_batch {
			calls: polkadot_relay_batch,
		});
		println!("\nBatch to submit on Polkadot Relay Chain:");
		print_output(output, &NetworkRuntimeCall::Polkadot(batch), use_light_client);
	}
	if !polkadot_asset_hub_batch.is_empty() {
		let batch =
			PolkadotAssetHubRuntimeCall::Utility(PolkadotAssetHubUtilityCall::force_batch {
				calls: polkadot_asset_hub_batch,
			});
		println!("\nBatch to submit on Polkadot Asset Hub:");
		print_output(output, &NetworkRuntimeCall::PolkadotAssetHub(batch), use_light_client);
	}
	if !polkadot_collectives_batch.is_empty() {
		let batch = CollectivesRuntimeCall::Utility(CollectivesUtilityCall::force_batch {
			calls: polkadot_collectives_batch,
		});
		println!("\nBatch to submit on Polkadot Collectives Chain:");
		print_output(output, &NetworkRuntimeCall::PolkadotCollectives(batch), use_light_client);
	}
}

// Format the data to print to console.
fn print_output(output: &Output, network_call: &NetworkRuntimeCall, use_light_client: bool) {
	match network_call {
		NetworkRuntimeCall::Kusama(call) => {
			let endpoint = if use_light_client {
				"light-client"
			} else {
				"wss%3A%2F%2Fkusama-rpc.dwellir.com"
			};
			let network_id = "kusama";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://dev.papi.how/extrinsics#data=0x{}&networkId={network_id}&endpoint={endpoint}",
					hex::encode(call.encode())
				),
			}
		},
		NetworkRuntimeCall::KusamaAssetHub(call) => {
			let endpoint = if use_light_client {
				"light-client"
			} else {
				"wss%3A%2F%2Fasset-hub-kusama-rpc.dwellir.com"
			};
			let network_id = "kusama_asset_hub";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://dev.papi.how/extrinsics#data=0x{}&networkId={network_id}&endpoint={endpoint}",
					hex::encode(call.encode())
				),
			}
		},
		NetworkRuntimeCall::Polkadot(call) => {
			let endpoint = if use_light_client {
				"light-client"
			} else {
				"wss%3A%2F%2Fpolkadot-rpc.dwellir.com"
			};
			let network_id = "polkadot";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://dev.papi.how/extrinsics#data=0x{}&networkId={network_id}&endpoint={endpoint}",
					hex::encode(call.encode())
				),
			}
		},
		NetworkRuntimeCall::PolkadotAssetHub(call) => {
			let endpoint = if use_light_client {
				"light-client"
			} else {
				"wss%3A%2F%2Fasset-hub-polkadot-rpc.dwellir.com"
			};
			let network_id = "polkadot_asset_hub";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://dev.papi.how/extrinsics#data=0x{}&networkId={network_id}&endpoint={endpoint}",
					hex::encode(call.encode())
				),
			}
		},
		NetworkRuntimeCall::PolkadotCollectives(call) => {
			let endpoint = if use_light_client {
				"light-client"
			} else {
				"wss%3A%2F%2Fpolkadot-collectives-rpc.polkadot.io"
			};
			let network_id = "polkadot_collectives";
			match output {
				Output::CallData => println!("0x{}", hex::encode(call.encode())),
				Output::AppsUiLink => println!(
					"https://dev.papi.how/extrinsics#data=0x{}&networkId={network_id}&endpoint={endpoint}",
					hex::encode(call.encode())
				),
			}
		},
		_ => panic!("no other chains are needed for this"),
	}
}
