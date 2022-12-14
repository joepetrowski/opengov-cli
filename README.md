# OpenGov Call Constructor

This script will construct the calls needed to submit a proposal to OpenGov on Kusama. It assumes that you construct the call elsewhere (e.g. Polkadot JS Apps UI Extrinsics tab) and then paste in the raw call data. It will return all the calls that you will need to sign and submit (also using, e.g., the Apps UI Extrinsics tab).

## Notes

1. This hardcodes the dispatch time to be `After(10)`. Will update to make it configurable.
2. This returns four calls, but they can actually be submitted in any order. But if dispatching a whitelisted call, the Fellowship referendum will have to enact (whitelisting the call) before the public one does. The preimages do not need to be submitted in order to start the referenda, but they will eventually in order to enact.

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
		// The encoded proposal that we want to submit.
		proposal: "0x630001000100a10f0204060202286bee880102957f0c9b47bc84d11116aef273e61565cf893801e7db0223aeea112e53922a4a",
		// The OpenGov track that it will use.
		track: OpenGovOrigin::WhitelistedCaller,
		// Choose if you just want to see the hex-encoded `CallData`, or get a link to Polkadot JS
		// Apps UI (`AppsUiLink`).
		output: Output::AppsUiLink,
		// Limit the length of calls printed to console. Prevents massive hex dumps for proposals
		// like runtime upgrades.
		output_len_limit: 1_000,
	}
}
```

### Run the Script

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
