## IC Event Hub example

The example consists of three following canisters:

* [emitter-counter](emitter-counter) - counter canister that emits an event each time it increments a value
* [listener-counter-1](listener-counter-1) - canister that completely mirrors the emitter by listening for all the
  emitted events
* [listener-counter-2](listener-counter-2) - canister that only mirrors the emitter, when the increment was triggered by
  some specified controller

Execute the [e2e-test](../../example-e2e-test) locally to see how it works.