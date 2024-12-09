const { Keyring } = require('@polkadot/api');
import { initApi } from './initApis';
const { hexToU8a } = require('@polkadot/util');
import colors from 'colors';

// 100 token
const transferAmount = '100000000000000';
const enclaveAccount = process.argv[2];

async function transfer(api, Alice) {
    console.log(colors.green('Transfer start...'));
    return new Promise((resolve, reject) => {
        api.tx.balances.transferKeepAlive(enclaveAccount, transferAmount).signAndSend(Alice, ({ status, events }) => {
            if (status.isInBlock || status.isFinalized) {
                events.forEach(({ event: { method, section } }) => {
                    if (method === 'Transfer' && section === 'balances') {
                        console.log(colors.green('Transfer completed'));
                        resolve('Transfer done');
                        return;
                    }
                });
            }
        });
    });
}

async function main() {
    const keyring = new Keyring({ type: 'sr25519' });

    // params: source chain endpoint, destination chain endpoint
    const { destinationAPI } = await initApi('ws://localhost:9944', 'ws://localhost:9944');
    const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });

    await transfer(destinationAPI, Alice);

    console.log(colors.green('Done'));
    process.exit();
}

main();
