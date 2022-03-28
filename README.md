## IC Event Hub

A rust library that enables efficient event-based pub/sub for IC canisters

### Motivation

The main idea behind the Open Internet Services concept is ___collaboration___. The easier it is for us to integrate
canisters, the more great software we can build together, the closer we are to the Open Internet.

This library greatly simplifies canister integration by flipping integration surface upside down. It enables your
canister to express its interfaces in terms of emitted events. Sending a message, your canister does not need to know a
signature of the remote canister anymore. Instead, the remote canister should know what events it wants to receive from
your canister. This enables us to create emitter canisters - ones which emit events, when something important happens
inside it, and which does not care who are the exact recipients of these events. On the other side, listener-canisters
(recipients) are able to make precise decisions which event topics they want to receive updates on, which is good for
both: performance/price and overall application logic.

### Tutorials

* [Introduction to ic-event-hub library]()
* [Tutorial: Efficient canister networking with ic-event-hub]()
* [Tutorial: Connecting a token with multiple ledgers using ic-event-hub]()

### Installation

```toml
# Cargo.toml

[dependencies]
ic-event-hub = "0.3"
ic-event-hub-macros = "0.3"
```

### Limitations

Right now `ic-event-hub` doesn't support canister upgrades, so all your queued tasks will be lost. This is due to a
limitation in `ic-cdk`, which doesn't support multiple stable variables at this moment. Once they do, I'll update this
library, so it will handle canister upgrades gracefully.

If you really want this functionality right now, you may try to serialize the state manually using `get_event_hub()`
function.

### Contribution

You can reach me out here on Github opening an issue, or you could start a thread on Dfinity's developer forum.

You're also welcome to suggest new features and open PR's.
