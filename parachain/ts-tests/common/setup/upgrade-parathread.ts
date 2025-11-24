import '@polkadot/api-augment';
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import type { ISubmittableResult } from '@polkadot/types/types';
import { loadConfig } from '../utils/index.js';

// upgrade the parathread to parachain by forcibly leasing a certain period
// can be used to extend the leasing period if it's already in onboarding process
async function upgrade_parathread_to_parachain(api: ApiPromise) {
    console.log('start onboarding parachain');
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    // leasing period in days
    const leasing_period = 999;

    return new Promise(async (resolvePromise) => {
        await api.tx.sudo
            .sudo(api.tx.slots.forceLease(process.env.PARACHAIN_ID, alice.address, 0, 0, leasing_period))
            .signAndSend(alice, ({ status }: ISubmittableResult) => {
                console.log(`Current status is ${status}`);
                if (status.isInBlock) {
                    console.log(`Transaction included at blockHash ${status.asInBlock}`);
                } else if (status.isFinalized) {
                    resolvePromise(0);
                }
            });
    });
}

(async () => {
    console.log('update to parachain ...');
    const config = loadConfig();

    const provider = new WsProvider(config.relaychain_ws);
    const api = await ApiPromise.create({
        provider: provider,
    });

    await upgrade_parathread_to_parachain(api);
    await api.disconnect();
    provider.on('disconnected', () => {
        console.log('Disconnect from relaychain');
        process.exit(0);
    });
})();
