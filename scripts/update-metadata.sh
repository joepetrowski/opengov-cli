#!/usr/bin/env bash
# Regenerate the runtime metadata snapshots in ./metadata from public RPC endpoints.
#
# The .scale files are point-in-time snapshots used by subxt codegen at build time. Re-run this
# script after runtime upgrades that change the calls this tool encodes (pallet/call indices or
# argument types), then rebuild and run the tests.
#
# Requires subxt-cli: `cargo install subxt-cli`

set -euo pipefail
cd "$(dirname "$0")/.."

# "<metadata file name>|<rpc endpoint>"
CHAINS=(
	"polkadot|wss://polkadot-rpc.n.dwellir.com:443"
	"polkadot_asset_hub|wss://polkadot-asset-hub-rpc.polkadot.io:443"
	"polkadot_collectives|wss://polkadot-collectives-rpc.polkadot.io:443"
	"polkadot_bridge_hub|wss://polkadot-bridge-hub-rpc.polkadot.io:443"
	"polkadot_people|wss://polkadot-people-rpc.polkadot.io:443"
	"polkadot_coretime|wss://polkadot-coretime-rpc.polkadot.io:443"
	"polkadot_bulletin|wss://bulletin-rpc.polkadot.io"
	"kusama|wss://kusama-rpc.n.dwellir.com:443"
	"kusama_asset_hub|wss://kusama-asset-hub-rpc.polkadot.io:443"
	"kusama_bridge_hub|wss://kusama-bridge-hub-rpc.polkadot.io:443"
	"kusama_encointer|wss://encointer-kusama-rpc.n.dwellir.com:443"
	"kusama_people|wss://kusama-people-rpc.polkadot.io:443"
	"kusama_coretime|wss://kusama-coretime-rpc.polkadot.io:443"
)

for entry in "${CHAINS[@]}"; do
	name="${entry%%|*}"
	url="${entry##*|}"
	echo "Updating metadata/${name}.scale from ${url}"
	subxt metadata --url "${url}" --version 16 --format bytes > "metadata/${name}.scale"
done

echo "Done. Rebuild and run 'cargo test' to check that the tool's calls still encode correctly."
