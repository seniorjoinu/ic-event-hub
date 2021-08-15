import {delay, ISetup, setup} from "./utils";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {assert} from 'chai';
import {deployExample} from "./deploy";

describe('counter', () => {
    let user1: ISetup, user2: ISetup;

    before(async () => {
        await deployExample();

        user1 = await setup(Ed25519KeyIdentity.generate());
        user2 = await setup(Ed25519KeyIdentity.generate());

        await user1.agent.fetchRootKey();
        await user2.agent.fetchRootKey();
    });

    it("flow works fine", async () => {
        // this listener should catch all events
        await user2.listenerCounter1Client.start_listening();

        // notice, user2 is the caller, so this listener will only catch events when user2 triggers the increment
        await user2.listenerCounter2Client.start_listening();

        // checking before
        const listener1Before1 = await user1.listenerCounter1Client.get_counter_value();
        assert.equal(listener1Before1, 0n, "Listener 1 state should be clean before everything");

        const listener2Before1 = await user1.listenerCounter2Client.get_counter_value();
        assert.equal(listener2Before1, 0n, "Listener 2 state should be clean before everything");

        // incrementing
        const emitterAfter1 = await user1.emitterCounterClient.inc();
        assert.equal(emitterAfter1, 1n, "Emitter state should equal 1");

        // waiting for event propagation
        await delay(3000);

        // checking after first increment
        const listener1After1 = await user1.listenerCounter1Client.get_counter_value();
        assert.equal(listener1After1, 1n, "Listener 1 state should equal 1");

        const listener2After1 = await user1.listenerCounter2Client.get_counter_value();
        assert.equal(listener2After1, 0n, "Listener 2 state should still equal 0");

        // incrementing by user2
        const emitterAfter2 = await user2.emitterCounterClient.inc();
        assert.equal(emitterAfter2, 2n, "Emitter state should equal 2");

        // waiting for event propagation
        await delay(3000);

        // checking after second increment
        const listener1After2 = await user1.listenerCounter1Client.get_counter_value();
        assert.equal(listener1After2, 2n, "Listener 1 state should equal 2");

        const listener2After2 = await user1.listenerCounter2Client.get_counter_value();
        assert.equal(listener2After2, 2n, "Listener 2 state should now also equal 2");
    });
});