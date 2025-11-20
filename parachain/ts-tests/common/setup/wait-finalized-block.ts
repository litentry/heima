import '@polkadot/api-augment';
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import type { VoidFn } from '@polkadot/api/types';
import type { ISubmittableResult } from '@polkadot/types/types';
import { loadConfig } from '../utils/index.js';

const FINALIZED_BLOCKS_COUNT = 1;
const TIMEOUT_MIN = 5;

/**
 * Connects to the parachain via the config file
 * and waits for `FINALIZED_BLOCKS_COUNT` blocks to
 * get finalized.
 *
 * It times out after `TIMEOUT_MIN` min.
 */
(async () => {
    const config = loadConfig();
    let timeout: NodeJS.Timeout;
    let count = 0;
    let unsub: VoidFn;
    let api: ApiPromise;

    const provider = new WsProvider(config.parachain_ws, 10000);

    console.log(`Connecting to parachain ${config.parachain_ws}`);

    timeout = global.setTimeout(
        async () => {
            if (typeof unsub === 'function') unsub();

            if (api) api.disconnect();
            provider.on('disconnected', () => {
                console.log(
                    `\nno block production detected after ${TIMEOUT_MIN}min, you might want to check it manually. Quit now`
                );
                process.exit(1);
            });
        },
        TIMEOUT_MIN * 60 * 1000
    );

    api = await ApiPromise.create({
        provider: provider,
    });

    unsub = await api.rpc.chain.subscribeFinalizedHeads(async (head) => {
        const blockNumber = head.number.toNumber();
        console.log(`Parachain finalized block #${blockNumber} with hash ${head.hash}`);
        count += 1;

        if (blockNumber >= 1 && count >= FINALIZED_BLOCKS_COUNT) {
            unsub();
            if (timeout) global.clearTimeout(timeout);

            await api.disconnect();
            provider.on('disconnected', () => {
                console.log('Done.');
                process.exit(0);
            });
        }
    });
})();
