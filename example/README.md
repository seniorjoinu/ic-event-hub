### IC Event Hub usage example

The example shows how a single emitter could emit the same event, but in reality send them only to those listeners,
which are subscribed to it. Emitter is a counter canister, which gets incremented by a user. Listeners are also counter
canisters, but they mirror the emitter's canister state only when they need to.

It contains:

* [canisters](./canisters)
* [ts-mocha e2e test](./e2e-test)