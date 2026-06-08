# OpenGov CLI

This program's primary purpose is to construct all the needed calls to submit a proposal as an OpenGov referendum on Kusama or Polkadot. It assumes that you construct the proposal (i.e., the privileged call you want to execute) elsewhere (e.g. Polkadot JS Apps UI Extrinsics tab). It will return all the calls that you will need to sign and submit (also using, e.g., the Apps UI Extrinsics tab). Note that you may need to submit calls on multiple chains.

It also provides a utility to construct a runtime upgrade call that will batch the upgrades of the Kusama or Polkadot Relay Chain with the upgrades of all their respective system parachains.

## CLI

This is a CLI program. To get started:

```
$ git clone https://github.com/joepetrowski/opengov-cli.git
$ cd opengov-cli
$ cargo build
$ ./target/debug/opengov-cli --help
Utilities for submitting OpenGov referenda and constructing tedious calls

Usage: opengov-cli <COMMAND>

Commands:
  build-upgrade      Generate a single call that will upgrade a Relay Chain and all of its system parachains
  submit-referendum  Generate all the calls needed to submit a proposal as a referendum in OpenGov
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Submit Referendum

The `submit-referendum` subcommand will take a proposal and some parameters and create all the necessary calls. Note that they can actually be submitted in any order. The preimages do not need to be submitted in order to start the referenda, but they will eventually in order to enact.

```
$ ./target/debug/opengov-cli submit-referendum --help
Generate all the calls needed to submit a proposal as a referendum in OpenGov

Usage: opengov-cli submit-referendum [OPTIONS] --proposal <PROPOSAL> --network <NETWORK> --track <TRACK>

Options:
  -p, --proposal <PROPOSAL>
          The encoded proposal that we want to submit. This can either be the call data itself, e.g. "0x0102...", or a file path that contains the data, e.g. "./my_proposal.call"
  -n, --network <NETWORK>
          Network on which to submit the referendum. `polkadot` or `kusama`
  -t, --track <TRACK>
          Track on which to submit the referendum
      --at <AT>
          Optional: Enact at a particular block number
      --at-date <AT_DATE>
          Optional: Enact at a particular wall clock time (format: DD-MM-YYThhmm, e.g. 25-05-21T0800)
          Note: The date/time is interpreted in your local machine's time zone.
      --after <AFTER>
          Optional: Enact after a given number of blocks
      --output-len-limit <OUTPUT_LEN_LIMIT>
          Output length limit. Defaults to 1,000
      --no-batch
          Do not print batch calls. Defaults to false
      --output <OUTPUT>
          Form of output. `AppsUiLink` or `CallData`. Defaults to Apps UI
      --light-client
          Use light client endpoints instead of RPC for PAPI links
  -h, --help
          Print help
```

### Build Upgrade

The `build-upgrade` subcommand will take a Relay Chain name and version and construct a single call to upgrade the Relay Chain and all of its system parachains.

```
$ ./target/debug/opengov-cli build-upgrade --help
Generate a single call that will upgrade a Relay Chain and all of its system parachains

Usage: opengov-cli build-upgrade [OPTIONS] --network <NETWORK> --relay-version <RELAY_VERSION>

Options:
  -n, --network <NETWORK>              Network on which to submit the referendum. `polkadot` or `kusama`
      --relay-version <RELAY_VERSION>  The Fellowship release version. Should be semver and correspond to the release published
      --asset-hub <ASSET_HUB>          Optional. The runtime version of Asset Hub to which to upgrade. If not provided, it will use the Relay Chain's version
      --bridge-hub <BRIDGE_HUB>        Optional. The runtime version of Bridge Hub to which to upgrade. If not provided, it will use the Relay Chain's version
      --collectives <COLLECTIVES>      Optional. The runtime version of Collectives to which to upgrade. If not provided, it will use the Relay Chain's version
      --filename <FILENAME>            Name of the file to which to write the output. If not provided, a default will be constructed
      --additional <ADDITIONAL>        Some additional call that you want executed on the Relay Chain along with the upgrade
  -h, --help                           Print help
```

## Examples

### Build Upgrade

```
$ ./target/release/opengov-cli build-upgrade --network polkadot --relay-version "1.0.0"

Downloading runtimes.

Downloading... polkadot_runtime-v1000000.compact.compressed.wasm
Downloading... asset_hub_polkadot_runtime-v1000000.compact.compressed.wasm
Downloading... collectives_polkadot_runtime-v1000000.compact.compressed.wasm
Downloading... bridge_hub_polkadot_runtime-v1000000.compact.compressed.wasm

Generating parachain authorization calls. The runtime hashes are logged if you would like to verify them with srtool.

Polkadot Asset Hub Runtime Hash:   0x52c2f520914514a196059fc8cc74f516a004f2463ba11d7385b5241bb5d50ee4
Polkadot Collectives Runtime Hash: 0xb061815642328374a62b3282c78fa8bef5a27cd313d4ac79cbd49e43e0a4b879
Polkadot Bridge Hub Runtime Hash:  0xf43e890a5eca0230a7eaaf88a60b8cc5ccb0ef157986f628650b071e47f7d323

Generating Relay Chain upgrade call. The runtime hash is logged if you would like to verify it with srtool.

Polkadot Relay Chain Runtime Hash: 0x17e1c7023134c196678f202daf8071a25f63f3e7b3937d8632a7474c618dc9a4

Batching calls.

Success! The call data was written to ./upgrade-polkadot-1.0.0/polkadot-1.0.0.call
To submit this as a referendum in OpenGov, run:

opengov-cli submit-referendum \
    --proposal "./upgrade-polkadot-1.0.0/polkadot-1.0.0.call" \
    --network "polkadot" --track <"root" or "whitelistedcaller">
```

### Submit a Referendum on Kusama

As a proposal, send an [XCM to Kusama Asset Hub](https://dev.papi.how/extrinsics#data=0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a&networkId=kusama&endpoint=wss%3A%2F%2Fkusama-rpc.dwellir.com) to authorize an upgrade.

Call data:
```
0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a
```

This has a call hash of `0x4149bf15976cd3c0c244ca0cd43d59fed76f4bb936b186cc18bd88dee6edd986`.

```
$ ./target/debug/opengov-cli submit-referendum \
	--proposal "0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a" \
	--network "kusama" --track "whitelistedcaller" \
	--after "10"

Submit the preimage for the Fellowship referendum:
https://dev.papi.how/extrinsics#data=0x2000882c004149bf15976cd3c0c244ca0cd43d59fed76f4bb936b186cc18bd88dee6edd986&networkId=kusama&endpoint=wss%3A%2F%2Fkusama-rpc.dwellir.com

Open a Fellowship referendum to whitelist the call:
https://dev.papi.how/extrinsics#data=0x17002b0f024c02d09f7b5e4b71e357780baf8cb2d625dca6efaba2ee777516eaf72e5a14a022000000010a000000&networkId=kusama&endpoint=wss%3A%2F%2Fkusama-rpc.dwellir.com

Submit the preimage for the public referendum:
https://dev.papi.how/extrinsics#data=0x2000d42c03630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a&networkId=kusama&endpoint=wss%3A%2F%2Fkusama-rpc.dwellir.com

Open a public referendum to dispatch the call:
https://dev.papi.how/extrinsics#data=0x15002b0d02022022c662d88f6b0f84c1771134e69b4412aff9e08a99e2a2da2794b5725fbe35000000010a000000&networkId=kusama&endpoint=wss%3A%2F%2Fkusama-rpc.dwellir.com

Batch to submit on Kusama Relay Chain:
https://dev.papi.how/extrinsics#data=0x1804102000882c004149bf15976cd3c0c244ca0cd43d59fed76f4bb936b186cc18bd88dee6edd98617002b0f024c02d09f7b5e4b71e357780baf8cb2d625dca6efaba2ee777516eaf72e5a14a022000000010a0000002000d42c03630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a15002b0d02022022c662d88f6b0f84c1771134e69b4412aff9e08a99e2a2da2794b5725fbe35000000010a000000&networkId=kusama&endpoint=wss%3A%2F%2Fkusama-rpc.dwellir.com
```

This will return either two or four calls, the latter if the origin is `WhitelistedCaller`, which will require a preimage and referendum for the Fellowship. It also returns a batch call if you want to submit them all at once (you can hide this with `--no-batch "true"`).

### Submit a Referendum on Polkadot

For Polkadot, we will use a proposal of `0x0000645468652046656c6c6f777368697020736179732068656c6c6f`, which is a `system.remark` call. We will use the Fellowship to whitelist it.

The Fellowship is on the Collectives parachain, so this will require a referendum on the Collectives chain for the Fellowship to whitelist a call, and a referendum on the Relay Chain for it to pass public vote. Notice the WSS nodes pointing to different chains in the output.

```
$ ./target/debug/opengov-cli submit-referendum \
	--proposal "0x0000645468652046656c6c6f777368697020736179732068656c6c6f" \
	--network "polkadot" --track "whitelistedcaller" \
	--after "10"

Submit the preimage for the Fellowship referendum:
https://dev.papi.how/extrinsics#data=0x2b00d41f0003010003082f00000603c2695e6d216f8817000363631d09c4ac33f2960d5d26b02f8ec89ac7a986c0bdab2a3a9f354acb6167&networkId=polkadot_collectives&endpoint=wss%3A%2F%2Fpolkadot-collectives-rpc.polkadot.io

Open a Fellowship referendum to whitelist the call:
https://dev.papi.how/extrinsics#data=0x3d003e0102664da7c8fb74a75e641b8aca751297fff57c5aee8014b3570e08f1454c06a88b35000000010a000000&networkId=polkadot_collectives&endpoint=wss%3A%2F%2Fpolkadot-collectives-rpc.polkadot.io

Submit the preimage for the public referendum:
https://dev.papi.how/extrinsics#data=0x0a007817030000645468652046656c6c6f777368697020736179732068656c6c6f&networkId=polkadot&endpoint=wss%3A%2F%2Fpolkadot-rpc.dwellir.com

Open a public referendum to dispatch the call:
https://dev.papi.how/extrinsics#data=0x1500160d022d1d8846a18770fc07a5b03383045d965aad65abb1077d0306142e60551813141e000000010a000000&networkId=polkadot&endpoint=wss%3A%2F%2Fpolkadot-rpc.dwellir.com

Batch to submit on Polkadot Relay Chain:
https://dev.papi.how/extrinsics#data=0x1a04080a007817030000645468652046656c6c6f777368697020736179732068656c6c6f1500160d022d1d8846a18770fc07a5b03383045d965aad65abb1077d0306142e60551813141e000000010a000000&networkId=polkadot&endpoint=wss%3A%2F%2Fpolkadot-rpc.dwellir.com

Batch to submit on Polkadot Collectives Chain:
https://dev.papi.how/extrinsics#data=0x2804082b00d41f0003010003082f00000603c2695e6d216f8817000363631d09c4ac33f2960d5d26b02f8ec89ac7a986c0bdab2a3a9f354acb61673d003e0102664da7c8fb74a75e641b8aca751297fff57c5aee8014b3570e08f1454c06a88b35000000010a000000&networkId=polkadot_collectives&endpoint=wss%3A%2F%2Fpolkadot-collectives-rpc.polkadot.io
```

### Checking the Results

We will use the Kusama referendum example to check the output. Let's check each of these calls and ensure they match our expectations.

The **first one** just gives us some bytes wrapped in a `note_preimage` call:

![note-preimage](https://i.imgur.com/vfMq3MS.png)

Let's of course look at those bytes:

![whitelist-call](https://i.imgur.com/VUEZcQk.png)

This whitelists a call with the hash of our proposal, `0x4149bf15976cd3c0c244ca0cd43d59fed76f4bb936b186cc18bd88dee6edd986`, good.

The **second call** starts a Fellowship referendum:

![fellowship-referendum](https://i.imgur.com/g1msmrV.png)

Note that the hash is for the `whitelist_call` instruction, _not_ our actual proposal.

The **third call** submits another preimage, this time a `dispatch_whitelisted_call_with_preimage` wrapping our actual proposal. The Fellowship referendum will have had to have passed before this executes.

![note-second-preimage](https://i.imgur.com/ECFTdDS.png)

Let's again inspect the noted bytes. We should find that they contain our original proposal!

![dispatch-whitelisted](https://i.imgur.com/WvAeHLZ.png)

It does, wrapped in the dispatch instruction.

Finally, the **fourth call** submits a referendum to the public:

![start-referendum](https://i.imgur.com/hGN9YHG.png)

Again, note that the hash of the referendum is the hash of the _dispatch_ instructions that contain our actual proposal.
