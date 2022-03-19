import {_SERVICE as IEmitterService} from 'dfx-type/emitter/emitter';
import {_SERVICE as IListenerService} from 'dfx-type/listener/listener';

import * as fs from "fs";
import fetch from "node-fetch";
import {Actor, CanisterInstallMode, getManagementCanister, HttpAgent, Identity} from "@dfinity/agent";
import {IDL} from "@dfinity/candid";
import {Principal} from "@dfinity/principal";

export interface ISetup {
    agent: HttpAgent;
    emitterService: IEmitterService;
    listenerService: IListenerService;
}

export async function setup(identity: Identity): Promise<ISetup> {
    const agent = new HttpAgent({
        host: 'http://localhost:8000/',
        // @ts-ignore
        fetch,
        identity,
    });
    await agent.fetchRootKey();

    const {
        actor: emitterService,
        canisterId: emitterCanisterId
    } = await deployCanister<IEmitterService>('emitter', [], agent);

    const {
        actor: listenerService
    } = await deployCanister<IListenerService>('listener', [...IDL.encode([IDL.Principal], [emitterCanisterId])], agent);

    return {
        agent,
        emitterService,
        listenerService,
    };
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