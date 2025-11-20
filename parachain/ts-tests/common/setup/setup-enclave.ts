import '@polkadot/api-augment';
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import { loadConfig, signAndSend, sudoWrapperGc } from '../utils/index.js';
import { hexToU8a } from '@polkadot/util';

const mrenclave = process.argv[2];
const block = process.argv[3];

async function setAliceAsAdmin(api: ApiPromise, config: any) {
    // Get keyring of Alice, who is also the sudo in dev chain spec
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    const tx = await sudoWrapperGc(api, api.tx.teebag.setAdmin('esqZdrqhgH8zy1wqYh1aLKoRyoRWLFbX9M62eKfaTAoK67pJ5'));

    console.log(`Setting Alice as Admin for Teebag`);
    return signAndSend(tx, alice);
}

async function setScheduledEnclave(api: ApiPromise, config: any) {
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    const tx = api.tx.teebag.setScheduledEnclave('Identity', block, hexToU8a(`0x${mrenclave}`));

    console.log('Schedule Enclave Extrinsic sent');
    return signAndSend(tx, alice);
}

(async () => {
    console.log('Schedule enclave on parachain ...');
    const config = loadConfig();

    const provider = new WsProvider(config.parachain_ws);
    const api = await ApiPromise.create({
        provider: provider,
    });

    await setAliceAsAdmin(api, config);
    await setScheduledEnclave(api, config);

    await api.disconnect();
    provider.on('disconnected', () => {
        console.log('Disconnect from relaychain');
        process.exit(0);
    });
})();
