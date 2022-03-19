## IC Event Hub example

The example consists of the two following canisters:

* [emitter](emitter) - a canister that receives messages from users, make batches of them and sends them to
  the `listener` canister as events
* [listener](listener) - a canister that receives batches of events from the `emitter`

Execute the [e2e-test](../e2e-test) locally to see how it works.