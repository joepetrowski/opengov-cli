# OpenGov Call Constructor

This script will construct the calls needed to submit a proposal to OpenGov on Kusama or Polkadot. It assumes that you construct the call elsewhere (e.g. Polkadot JS Apps UI Extrinsics tab) and then paste in the raw call data. It will return all the calls that you will need to sign and submit (also using, e.g., the Apps UI Extrinsics tab). Note that you may need to submit calls on multiple chains.

## Notes

1. This returns up to four calls, but they can actually be submitted in any order. However, if dispatching a whitelisted call, the Fellowship referendum will have to _enact_ (whitelisting the call) before the public one does. The preimages do not need to be submitted in order to start the referenda, but they will eventually in order to enact.

## Example

### Proposal

Send an [XCM to Statemine](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fkusama-rpc.polkadot.io#/extrinsics/decode/0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a) to authorize an upgrade.

Call data:
```
0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a
```

This has a call hash of `0x4149bf15976cd3c0c244ca0cd43d59fed76f4bb936b186cc18bd88dee6edd986`.

In the script, we will paste the _call data_ into:

```rust
fn get_the_actual_proposed_action() -> ProposalDetails {
	return ProposalDetails {
		// The encoded proposal that we want to submit. This can either be the call data itself,
		// e.g. "0x0102...", or a file path that contains the data, e.g. "./my_proposal.call".
		proposal: "0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a",
		// The OpenGov track that it will use. You need to wrap this in either Kusama or Polkadot.
		track: Kusama(OpenGovOrigin::WhitelistedCaller),
		// When do you want this to enact. `At(block)` a specific block number or `After(blocks)`,
		// some delay from passing.
		dispatch: After(10),
		// Choose if you just want to see the hex-encoded `CallData`, or get a link to Polkadot JS
		// Apps UI (`AppsUiLink`).
		output: Output::AppsUiLink,
		// Limit the length of calls printed to console. Prevents massive hex dumps for proposals
		// like runtime upgrades.
		output_len_limit: 1_000,
		// Whether or not to print a single `force_batch` call.
		print_batch: true,
	}
}
```

### Run the Script

#### Kusama

```
$ cargo run
   Compiling opengov-submit v0.1.0 (/home/joe/parity/sideprojects/opengov-submit)
    Finished dev [unoptimized + debuginfo] target(s) in 10.95s
     Running `target/debug/opengov-submit`

Submit the preimage for the Fellowship referendum:
0x2000882c004149bf15976cd3c0c244ca0cd43d59fed76f4bb936b186cc18bd88dee6edd986

Open a Fellowship referendum to whitelist the call:
0x17002b0f024c02d09f7b5e4b71e357780baf8cb2d625dca6efaba2ee777516eaf72e5a14a022000000010a000000

Submit the preimage for the public referendum:
0x2000d42c03630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a

Open a public referendum to dispatch the whitelisted call:
0x15002b0d02022022c662d88f6b0f84c1771134e69b4412aff9e08a99e2a2da2794b5725fbe35000000010a000000
```

This will return either two or four calls, the latter if the origin is `WhitelistedCaller`, which will require a preimage and referendum for the Fellowship.

#### Polkadot

The Fellowship is on the Collectives parachain, so this will require a referendum on the Collectives chain for the Fellowship to whitelist a call, and then a referendum on the Relay Chain for it to pass public vote. Notice the WSS nodes pointing to different chains in the output.

```
$ cargo run
   Compiling opengov-submit v0.1.0 (/home/joe/parity/sideprojects/opengov-submit)
    Finished dev [unoptimized + debuginfo] target(s) in 30.32s
     Running `target/debug/opengov-submit`

Submit the preimage for the Fellowship referendum:
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpolkadot-collectives-rpc.polkadot.io#/extrinsics/decode/0x2b00dc1f0003010003082f0000060302286bee02093d008817000363631d09c4ac33f2960d5d26b02f8ec89ac7a986c0bdab2a3a9f354acb6167

Open a Fellowship referendum to whitelist the call:
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpolkadot-collectives-rpc.polkadot.io#/extrinsics/decode/0x3d003e010259c8fcb1f6af54c364eb074abd824193044e1b3dac38dbc6f45307e7959e939037000000010a000000

Submit the preimage for the public referendum:
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frpc.polkadot.io#/extrinsics/decode/0x0a007817030000645468652046656c6c6f777368697020736179732068656c6c6f

Open a public referendum to dispatch the call:
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frpc.polkadot.io#/extrinsics/decode/0x1500160d022d1d8846a18770fc07a5b03383045d965aad65abb1077d0306142e60551813141e000000010a000000
```

### Checking the Results

Let's check each of these calls and ensure they match our expectations.

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
