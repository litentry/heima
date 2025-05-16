import { ApiPromise } from '@polkadot/api';
import { KeyObject } from 'crypto';
import WebSocketAsPromised from 'websocket-as-promised';
import type { IntegrationTestContext } from '../common-types';
import type { Metadata, TypeRegistry } from '@polkadot/types';
import type { HexString } from '@polkadot/util/types';
import { initIntegrationTestContext } from './context';

export function describeLitentry(title: string, cb: (context: IntegrationTestContext) => void) {
    describe(title, function () {
        // Set timeout to 6000 seconds
        this.timeout(6000000);

        const context: IntegrationTestContext = {
            mrEnclave: '0x11' as HexString,
            api: {} as ApiPromise,
            tee: {} as WebSocketAsPromised,
            teeShieldingKey: {} as KeyObject,
            web3Wallets: {} as any,
            sidechainMetaData: {} as Metadata,
            sidechainRegistry: {} as TypeRegistry,
            // default LitentryRococo
            chainIdentifier: 42,
            requestId: 0,
        };

        before('Starting Litentry(parachain&tee)', async function () {
            //env url

            const tmp = await initIntegrationTestContext(process.env.PARACHAIN_ENDPOINT!);
            context.mrEnclave = tmp.mrEnclave;
            context.api = tmp.api;
            context.tee = tmp.tee;
            context.teeShieldingKey = tmp.teeShieldingKey;
            context.web3Wallets = tmp.web3Wallets;
            context.sidechainMetaData = tmp.sidechainMetaData;
            context.sidechainRegistry = tmp.sidechainRegistry;
            context.chainIdentifier = tmp.chainIdentifier;
        });

        after(() => Promise.resolve());

        cb(context);
    });
}
