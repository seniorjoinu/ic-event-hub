import {_SERVICE as IEmitterService} from 'dfx-type/emitter-counter/emitter-counter';
import {_SERVICE as IListener1Service} from 'dfx-type/listener-counter-1/listener-counter-1';
import {_SERVICE as IListener2Service} from 'dfx-type/listener-counter-2/listener-counter-2';

import * as fs from "fs";
import fetch from "node-fetch";
import {Actor, CanisterInstallMode, getManagementCanister, HttpAgent, Identity} from "@dfinity/agent";
import {IDL} from "@dfinity/candid";
import {Principal} from "@dfinity/principal";

export interface ISetup {
    user1: HttpAgent;
    emitterClientUser1: IEmitterService;
    listener1ClientUser1: IListener1Service;
    listener2ClientUser1: IListener2Service;

    user2: HttpAgent;
    emitterClientUser2: IEmitterService;
    listener1ClientUser2: IListener1Service;
    listener2ClientUser2: IListener2Service;
}

export async function setup(identity1: Identity, identity2: Identity): Promise<ISetup> {
    const agent1 = new HttpAgent({
        host: 'http://localhost:8000/',
        // @ts-ignore
        fetch,
        identity: identity1
    });
    await agent1.fetchRootKey();

    const agent2 = new HttpAgent({
        host: 'http://localhost:8000/',
        // @ts-ignore
        fetch,
        identity: identity2
    });
    await agent2.fetchRootKey();

    const {
        actor: emitterClientUser1,
        canisterId: canisterId1
    } = await deployCanister<IEmitterService>('emitter-counter', [], agent1);
    const emitterClientUser2 = await connectCanister<IEmitterService>('emitter-counter', canisterId1, agent2);

    const {
        actor: listener1ClientUser1,
        canisterId: canisterId2
    } = await deployCanister<IListener1Service>('listener-counter-1', [...IDL.encode([IDL.Principal], [canisterId1])], agent1);
    const listener1ClientUser2 = await connectCanister<IListener1Service>('listener-counter-1', canisterId2, agent2);

    const {
        actor: listener2ClientUser1,
        canisterId: canisterId3
    } = await deployCanister<IListener2Service>('listener-counter-2', [...IDL.encode([IDL.Principal], [canisterId1])], agent1);
    const listener2ClientUser2 = await connectCanister<IListener2Service>('listener-counter-2', canisterId3, agent2);

    return {
        user1: agent1,
        emitterClientUser1,
        listener1ClientUser1,
        listener2ClientUser1,

        user2: agent2,
        emitterClientUser2,
        listener1ClientUser2,
        listener2ClientUser2,
    };
}

export async function connectCanister<T>(name: string, canisterId: Principal, agent: HttpAgent): Promise<T> {
    const {idlFactory} = await import(`dfx/${name}/${name}`)

    return Actor.createActor(idlFactory, {
        agent,
        canisterId
    });
}

export async function deployCanister<T>(name: string, arg: number[], agent: HttpAgent): Promise<{ actor: T, canisterId: Principal }> {
    const managementCanister = getManagementCanister({agent});
    const {canister_id} = await managementCanister.provisional_create_canister_with_cycles({amount: [], settings: []});
    const wasm = fs.readFileSync(`.dfx/local/canisters/${name}/${name}.wasm`);
    const {idlFactory} = await import(`dfx/${name}/${name}`)

    await managementCanister.install_code({
        canister_id,
        mode: {[CanisterInstallMode.Install]: null},
        wasm_module: [...wasm],
        arg
    });

    console.log(`Canister ${name} ${canister_id} deployed`);

    return {
        actor: Actor.createActor(idlFactory, {
            agent,
            canisterId: canister_id
        }),
        canisterId: canister_id
    };
}