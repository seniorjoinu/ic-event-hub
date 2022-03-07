### E2E test for ic-event-hub

This directory contains `ts-mocha` tests for canisters located at [example directory](../canisters)

#### Requirements

* `rust`
* `wasm32-unknown-unknown` target
* `dfx 0.9.0`
* `ic-cdk-optimizer` (`cargo install --locked ic-cdk-optimizer`)

#### Local development

* `yarn install` - install dependencies
* `yarn start` - start a replica in a separate terminal
* `yarn test` - start the test
* observe replicas logs