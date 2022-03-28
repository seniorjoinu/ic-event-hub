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

* [Introduction to ic-event-hub library](https://dev.to/seniorjoinu/introduction-to-ic-event-hub-library-5366)
* [Tutorial: Connecting A Token With Multiple Ledgers Using ic-event-hub](https://dev.to/seniorjoinu/tutorial-connecting-a-token-with-multiple-ledgers-using-ic-event-hub-1d4)
* [Tutorial: Efficient Canister Networking With ic-event-hub](https://dev.to/seniorjoinu/tutorial-efficient-canister-networking-with-ic-event-hub-4idb)

### Installation

```toml
# Cargo.toml

[dependencies]
ic-event-hub = "0.3"
ic-event-hub-macros = "0.3"
```

### Contribution

You can reach me out here on Github opening an issue, or you could start a thread on Dfinity's developer forum.

You're also welcome to suggest new features and open PR's.
