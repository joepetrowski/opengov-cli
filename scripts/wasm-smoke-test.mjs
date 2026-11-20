// Check that the wasm package built by `wasm-pack` loads in a JavaScript host and returns the
// same bytes as the Rust tests assert (see src/tests.rs).
//
// Usage: node scripts/wasm-smoke-test.mjs [pkg directory]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const pkg = resolve(process.argv[2] ?? 'pkg');
const wasm = await import(pathToFileURL(join(pkg, 'opengov.js')));
await wasm.default({ module_or_path: readFileSync(join(pkg, 'opengov_bg.wasm')) });

// build-upgrade: a Bulletin-only upgrade with a stand-in runtime.
assert.equal(
	wasm.runtime_file_name('bulletin-polkadot', 'v2.5.3'),
	'bulletin-polkadot_runtime-v2005003.compact.compressed.wasm',
);
const runtime = new TextEncoder().encode('fake runtime bulletin-polkadot');
assert.throws(() => wasm.check_and_hash_runtime(runtime, false), /not a zstd-compressed/);
const code_hash = wasm.check_and_hash_runtime(runtime, true);
const upgrade = wasm.build_upgrade({
	relay: 'polkadot',
	runtimes: [{ network: 'bulletin-polkadot', code_hash }],
});
assert.equal(upgrade.network, 'asset-hub-polkadot');
assert.equal(
	upgrade.encoded,
	'0x1f0005010100c90f05082f0000060200880009d71d8efe5e48431864aa41d98b6d40bc5f2bcda4253b33dc4913810bac323c80',
);
assert.throws(() => wasm.build_upgrade({ relay: 'westend', runtimes: [] }), /must be `polkadot` or `kusama`/);

// submit-referendum: `system.remark("opengov-submit test")` on the Whitelisted Caller track.
const calls = wasm.referendum_calls({
	proposal: '0x00004c6f70656e676f762d7375626d69742074657374',
	network: 'polkadot',
	track: 'whitelistedcaller',
	dispatch: { after: 10 },
});
assert.deepEqual(
	{
		fellowship_preimage: calls.fellowship_preimage,
		fellowship_referendum: [calls.fellowship_referendum.network, calls.fellowship_referendum.encoded],
		public_preimage: [calls.public_preimage.network, calls.public_preimage.encoded],
		public_referendum: [calls.public_referendum.network, calls.public_referendum.encoded],
		public_preimage_bytes: calls.public_preimage_bytes,
		public_preimage_hash: calls.public_preimage_hash,
		public_preimage_len: calls.public_preimage_len,
		whitelisted_call_hash: calls.whitelisted_call_hash,
	},
	{
		fellowship_preimage: null,
		fellowship_referendum: [
			'collectives-polkadot',
			'0x3d003e0201cc1f0005010100a10f05082f00000603008840008821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534010a000000',
		],
		public_preimage: [
			'asset-hub-polkadot',
			'0x050060400300004c6f70656e676f762d7375626d69742074657374',
		],
		public_referendum: [
			'asset-hub-polkadot',
			'0x3e003f0d02a322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e18000000010a000000',
		],
		public_preimage_bytes: '0x400300004c6f70656e676f762d7375626d69742074657374',
		public_preimage_hash: '0xa322f65fd03ba368587f997b14e306211f6fb3c30b06a5be472f2f96b3b27e1e',
		public_preimage_len: 24,
		whitelisted_call_hash: '0x8821e8db19b8e34b62ee8bc618a5ed3eecb9761d7d81349b00aa5ce5dfca2534',
	},
);
assert.throws(
	() => wasm.referendum_calls({ proposal: '0x0000', network: 'polkadot', track: 'nonsense', dispatch: { after: 10 } }),
	/Unsupported track/,
);

console.log('wasm package OK');
