import {delay} from "./utils";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {assert} from 'chai';
import {ISetup, setup} from "./deploy";

describe('counter', () => {
    let set: ISetup;

    before(async () => {
        set = await setup(Ed25519KeyIdentity.generate(), Ed25519KeyIdentity.generate());
    });

    it("flow works fine", async () => {
        // this listener should catch all events
        await set.listenerClientUser2.start_listening();

        // checking before
        const emitterCounterBefore = await set.emitterClientUser1.get_counter_value();
        assert.equal(emitterCounterBefore, 0n, "Emitter counter state should be clean before everything");

        const listenerCounterBefore = await set.listenerClientUser1.get_counter_value();
        const listenerMirrorTriggersBefore = await set.listenerClientUser1.get_times_events_callback_triggered();
        assert.equal(listenerCounterBefore, 0n, "Listener counter state should be clean before everything");
        assert.equal(listenerMirrorTriggersBefore, 0n, "Listener triggers state should be clean before everything");

        // 7 events is enough for 1024 bytes threshold
        for (let i = 0; i < 7; i++) {
            await set.emitterClientUser1.mirror();
        }

        // it should send all events in one batch
        const emitterCounterAfter = await set.emitterClientUser1.get_counter_value();
        assert.equal(emitterCounterAfter, 7n, "Emitter counter state should be equal to 7");

        await delay(10_000);

        const listenerCounterAfter = await set.listenerClientUser1.get_counter_value();
        assert.equal(listenerCounterAfter, 7n, "Listener counter state should be equal to 7");

        const listenerTriggersAfter = await set.listenerClientUser1.get_times_events_callback_triggered();
        assert.equal(listenerTriggersAfter, 1n, "Listener trigger state should be equal to 1");
    });
});