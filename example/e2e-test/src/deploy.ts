import {_SERVICE as IEmitterService} from 'dfx-type/emitter/emitter';
import {_SERVICE as IListenerService} from 'dfx-type/listener/listener';

import * as fs from "fs";
import fetch from "node-fetch";
import {Actor, CanisterInstallMode, getManagementCanister, HttpAgent, Identity} from "@dfinity/agent";
import {IDL} from "@dfinity/candid";
import {Principal} from "@dfinity/principal";

export interface ISetup {
    user1: HttpAgent;
    emitterClientUser1: IEmitterService;
    listenerClientUser1: IListenerService;

    user2: HttpAgent;
    emitterClientUser2: IEmitterService;
    listenerClientUser2: IListenerService;
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
    } = await deployCanister<IEmitterService>('emitter', [], agent1);
    const emitterClientUser2 = await connectCanister<IEmitterService>('emitter', canisterId1, agent2);

    const {
        actor: listenerClientUser1,
        canisterId: canisterId2
    } = await deployCanister<IListenerService>('listener', [...IDL.encode([IDL.Principal], [canisterId1])], agent1);
    const listenerClientUser2 = await connectCanister<IListenerService>('listener', canisterId2, agent2);

    return {
        user1: agent1,
        emitterClientUser1,
        listenerClientUser1,

        user2: agent2,
        emitterClientUser2,
        listenerClientUser2,
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