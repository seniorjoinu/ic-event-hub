import {Actor, HttpAgent, Identity} from "@dfinity/agent";
import fetch from 'node-fetch';
import {exec} from 'child_process';

import IEmitterCounterClient from 'dfx-type/emitter-counter/emitter-counter';
import IListenerCounter1Client from 'dfx-type/listener-counter-1/listener-counter-1';
import IListenerCounter2Client from 'dfx-type/listener-counter-2/listener-counter-2';

export interface ISetup {
    agent: HttpAgent;
    emitterCounterClient: IEmitterCounterClient;
    listenerCounter1Client: IListenerCounter1Client;
    listenerCounter2Client: IListenerCounter2Client;
}

export async function setup(identity: Identity): Promise<ISetup> {
    const agent = new HttpAgent({
        host: 'http://localhost:8000/',
        // @ts-ignore
        fetch,
        identity
    });

    const {
        canisterId: emitterCanisterId,
        idlFactory: emitterIdlFactory,
    } = await import('dfx/emitter-counter/emitter-counter');

    const emitterCounterClient: IEmitterCounterClient = Actor.createActor(emitterIdlFactory, {
        agent,
        canisterId: emitterCanisterId
    });


    const {
        canisterId: listener1CanisterId,
        idlFactory: listener1IdlFactory,
    } = await import('dfx/listener-counter-1/listener-counter-1');

    const listenerCounter1Client: IListenerCounter1Client = Actor.createActor(listener1IdlFactory, {
        agent,
        canisterId: listener1CanisterId
    });


    const {
        canisterId: listener2CanisterId,
        idlFactory: listener2IdlFactory,
    } = await import('dfx/listener-counter-2/listener-counter-2');


    const listenerCounter2Client: IListenerCounter2Client = Actor.createActor(listener2IdlFactory, {
        agent,
        canisterId: listener2CanisterId
    });


    return {
        agent,
        emitterCounterClient,
        listenerCounter1Client,
        listenerCounter2Client
    };
}

export async function execAsync(command: string) {
    return new Promise((res, rej) => {
        exec(command, (err, stderr, stdout) => {
            if (err) {
                rej(err);
            } else if (stderr) {
                rej(stderr);
            } else if (stdout) {
                res(stdout);
            } else {
                res("No error");
            }
        })
    })
}

export async function delay(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}