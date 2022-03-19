import {Ed25519KeyIdentity} from "@dfinity/identity";
import {assert} from 'chai';
import {ISetup, setup} from "./deploy";

function delay(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

describe('event batching', () => {
    let s: ISetup;

    before(async () => {
        s = await setup(Ed25519KeyIdentity.generate());
    });

    it("flow works fine", async () => {
        // this listener should catch all events
        await s.listenerService.start_listening();

        // checking before
        const emitterRequestsBefore = await s.emitterService.get_requests_count();
        assert.equal(emitterRequestsBefore, 0n, "Emitter state should be clean before everything");

        const listenerEventsBefore = await s.listenerService.get_events_received();
        const listenerBatchesBefore = await s.listenerService.get_batches_received();
        assert.equal(listenerEventsBefore, 0n, "Listener events state should be clean before everything");
        assert.equal(listenerBatchesBefore, 0n, "Listener batches state should be clean before everything");

        // sending 10 events each of 100 bytes of data
        for (let i = 0; i < 10; i++) {
            await s.emitterService.mirror(Array(100).fill(1));
        }

        // it should send all events in one batch
        const emitterRequestsAfter = await s.emitterService.get_requests_count();
        assert.equal(emitterRequestsAfter, 10n, "Emitter requests count should equal 10");

        // waiting for at least 10 seconds
        await delay(10_000);

        const listenerEventsAfter = await s.listenerService.get_events_received();
        assert.equal(listenerEventsAfter, 10n, "Listener events count should be equal to 10");

        const listenerBatchesAfter = await s.listenerService.get_batches_received();
        assert.equal(listenerBatchesAfter, 1n, "Listener batches count should be equal to 1");
    });
});