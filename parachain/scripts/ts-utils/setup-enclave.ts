//run: pnpm exec ts-node setup-enclave.ts $enclaveAccount $mrenclave $accountPassword
//example: pnpm exec ts-node setup-enclave.ts 2KWd4sEmYj2VW42L2WUDDRKA4JwnKg76uoQ2keUBUwFHU9Dx a552654d1733c4054a3c7e5e86adf26b5d65c072b57b2550fe763821ebac54c6 123456
const { Keyring } = require('@polkadot/api');
import { initApi } from './initApis';
const { hexToU8a } = require('@polkadot/util');
import colors from 'colors';

//100 token
const transferAmount = '100000000000000';
const enclaveAccount = process.argv[2];
const mrenclave = process.argv[3];
const block = process.argv[4];

async function transfer(api: any, Alice: any) {
    console.log(colors.green('transfer start...'));
    return new Promise(async (resolve, reject) => {
        await api.tx.balances
            .transferKeepAlive(enclaveAccount, transferAmount)
            .signAndSend(Alice, ({ status, events, dispatchError }) => {
                if (status.isInBlock || status.isFinalized) {
                    events.forEach(({ phase, event: { data, method, section } }) => {
                        if (method === 'Transfer' && section === 'balances') {
                            console.log(colors.green('transfer completed'));
                            resolve('transfer done');
                            return;
                        }
                    });
                }
            });
    });
}
async function teebagSetAdmin(api: any, Alice: any) {
    return new Promise(async (resolve, reject) => {
        // Note: The hardcoded address is that of Alice on dev chain
        await api.tx.sudo
            .sudo(api.tx.teebag.setAdmin('esqZdrqhgH8zy1wqYh1aLKoRyoRWLFbX9M62eKfaTAoK67pJ5'))
            .signAndSend(Alice, ({ status, events, dispatchError }) => {
                if (status.isInBlock || status.isFinalized) {
                    if (dispatchError) {
                        if (dispatchError.isModule) {
                            // for module errors, we have the section indexed, lookup
                            const decoded = api.registry.findMetaError(dispatchError.asModule);
                            const { docs, name, section } = decoded;

                            console.log(colors.red(`${section}.${name}: ${docs.join(' ')}`));
                            reject('teebag.setAdmin failed');
                        } else {
                            // Other, CannotLookup, BadOrigin, no extra info
                            console.log(dispatchError.toString());
                            reject('teebag.setAdmin failed');
                        }
                    } else {
                        console.log(colors.green('teebag.setAdmin completed'));
                        resolve('teebag.setAdmin done');
                    }
                }
            });
    });
}
async function teebagSetScheduledEnclave(api: any, Alice: any, block: any) {
    return new Promise(async (resolve, reject) => {
        await api.tx.teebag
            .setScheduledEnclave('Identity', block, hexToU8a(`0x${mrenclave}`))
            .signAndSend(Alice, ({ status, events, dispatchError }) => {
                if (status.isInBlock || status.isFinalized) {
                    if (dispatchError) {
                        if (dispatchError.isModule) {
                            // for module errors, we have the section indexed, lookup
                            const decoded = api.registry.findMetaError(dispatchError.asModule);
                            const { docs, name, section } = decoded;

                            console.log(colors.red(`${section}.${name}: ${docs.join(' ')}`));
                            reject('teebag.setScheduledEnclave failed');
                        } else {
                            // Other, CannotLookup, BadOrigin, no extra info
                            console.log(dispatchError.toString());
                            reject('teebag.setScheduledEnclave failed');
                        }
                    } else {
                        console.log(colors.green('teebag.setScheduledEnclave completed'));
                        resolve('teebag.setScheduledEnclave done');
                    }
                }
            });
    });
}

async function main() {
    // NOTE: If we are using development environment, we can use Alice instead of moving around with keys
    const keyring = new Keyring({ type: 'sr25519' });

    console.log(colors.green('account unlock...'));

    // params: source chain endpoint, destination chain endpoint
    const { destinationAPI } = await initApi('ws://localhost:9944', 'ws://localhost:9944');
    const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });

    await transfer(destinationAPI, Alice);
    await teebagSetAdmin(destinationAPI, Alice);
    await teebagSetScheduledEnclave(destinationAPI, Alice, block);

    console.log(colors.green('done'));
    process.exit();
}

main();
