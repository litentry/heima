import { ApiPromise, WsProvider } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import colors from 'colors';

/*
 * @param fromEndpoint - the endpoint of the source chain
 * @param toEndpoint - the endpoint of the destination chain
 */

export const initApi = async (fromEndpoint: string, toEndpoint: string) => {
    const sourceProvider = new WsProvider(fromEndpoint);
    const destinationProvider = new WsProvider(toEndpoint);

    const sourceApi = await ApiPromise.create({ provider: sourceProvider });
    const destinationAPI = await ApiPromise.create({ provider: destinationProvider });
    await cryptoWaitReady();
    console.log(colors.green('api is ready'));

    return { sourceApi, destinationAPI };
};
