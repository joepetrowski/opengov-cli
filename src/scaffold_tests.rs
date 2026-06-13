use crate::chopsticks::generate_test_scaffold;
use clap::Parser as ClapParser;
use std::fs;

/// Generate test scaffolding for chopsticks testing.
#[derive(Debug, ClapParser)]
pub(crate) struct GenerateTestScaffoldArgs {
	/// Network to generate tests for (`polkadot` or `kusama`).
	#[clap(long = "network", short)]
	network: String,

	/// Output file name. Defaults to `testfile.js`.
	#[clap(long = "output", short)]
	output: Option<String>,
}

// The sub-command's "main" function.
pub(crate) async fn run_generate_test_scaffold(prefs: GenerateTestScaffoldArgs) {
	// Validate network
	let network = match prefs.network.to_lowercase().as_str() {
		"polkadot" => "polkadot",
		"kusama" => "kusama",
		_ => {
			eprintln!("Error: Network must be 'polkadot' or 'kusama'");
			return;
		},
	};

	let test_content = generate_test_scaffold(network);
	let output_file = prefs.output.unwrap_or_else(|| "testfile.js".to_string());

	match fs::write(&output_file, test_content) {
		Ok(_) => {
			println!("Test scaffold generated: {}", output_file);
			println!("\nUsage:");
			println!("  opengov-cli submit-referendum \\");
			println!("    --proposal \"./your-proposal.call\" \\");
			println!("    --network \"{}\" \\", network);
			println!("    --track \"whitelistedcaller\" \\");
			println!("    --test \"{}\"", output_file);
			println!("\nPrerequisites:");
			println!("  npm install -g @acala-network/chopsticks");
			println!("  npm install @polkadot/api @polkadot/util-crypto");
		},
		Err(e) => {
			eprintln!("Error writing test file: {}", e);
		},
	}
}
