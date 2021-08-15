import {execAsync} from "./utils";

export async function deployExample() {
    const command1 = `dfx deploy emitter-counter`;
    console.log(command1);
    console.log(await execAsync(command1));

    const {canisterId} = await import('dfx/emitter-counter/emitter-counter');

    const command2 = `dfx deploy listener-counter-1 --argument '(principal "${canisterId}")'`;
    console.log(command2);
    console.log(await execAsync(command2));

    const command3 = `dfx deploy listener-counter-2 --argument '(principal "${canisterId}")'`;
    console.log(command3);
    console.log(await execAsync(command3));
}