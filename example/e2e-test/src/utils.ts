import {exec} from 'child_process';

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