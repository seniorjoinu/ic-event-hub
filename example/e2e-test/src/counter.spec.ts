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
        await set.listener1ClientUser2.start_listening();

        // notice, user2 is the caller, so this listener will only catch events when user2 triggers the increment
        await set.listener2ClientUser2.start_listening();

        // checking before
        const listener1Before1 = await set.listener1ClientUser1.get_counter_value();
        assert.equal(listener1Before1, 0n, "Listener 1 state should be clean before everything");

        const listener2Before1 = await set.listener2ClientUser1.get_counter_value();
        assert.equal(listener2Before1, 0n, "Listener 2 state should be clean before everything");

        // incrementing
        const emitterAfter1 = await set.emitterClientUser1.inc();
        assert.equal(emitterAfter1, 1n, "Emitter state should equal 1");

        // waiting for event propagation
        await delay(10000);

        // checking after first increment
        const listener1After1 = await set.listener1ClientUser1.get_counter_value();
        assert.equal(listener1After1, 1n, "Listener 1 state should equal 1");

        const listener2After1 = await set.listener2ClientUser1.get_counter_value();
        assert.equal(listener2After1, 0n, "Listener 2 state should still equal 0");

        // incrementing by user2
        const emitterAfter2 = await set.emitterClientUser2.inc();
        assert.equal(emitterAfter2, 2n, "Emitter state should equal 2");

        // waiting for event propagation
        await delay(10000);

        // checking after second increment
        const listener1After2 = await set.listener1ClientUser1.get_counter_value();
        assert.equal(listener1After2, 2n, "Listener 1 state should equal 2");

        const listener2After2 = await set.listener2ClientUser1.get_counter_value();
        assert.equal(listener2After2, 2n, "Listener 2 state should now also equal 2");
    });
});