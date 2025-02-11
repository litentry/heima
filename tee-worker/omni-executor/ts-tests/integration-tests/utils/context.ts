import type { HexString } from '@polkadot/util/types';

import { ApiPromise, WsProvider, identity, omniAccount, omniExecutor } from 'parachain-api';
import { hexToString } from '@polkadot/util';
import WebSocketAsPromised from 'websocket-as-promised';
import WsAsPromiseOptions from 'websocket-as-promised/types/options';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { createWeb3Wallets } from './wallet';
import { KeyObject, createPublicKey } from 'crypto';
import WebSocket from 'ws';
import { Wallets } from './wallet';

export type IntegrationTestContext = {
    teeWsClient: WebSocketAsPromised;
    api: ApiPromise;
    // teeShieldingKey: KeyObject;
    mrEnclave: HexString;
    web3Wallets: Wallets;
    chainIdentifier: number;
    requestId: number;
};

export async function createIntegrationTestContext(
    parachainEndpoint: string = 'ws://localhost:9944',
    workerEndpoint: string = 'ws://localhost:2000'
): Promise<IntegrationTestContext> {
    await cryptoWaitReady();

    const provider = new WsProvider(parachainEndpoint);
    const api = await ApiPromise.create({
        provider,
        types: {
            ...identity.types,
            ...omniAccount.types,
            ...omniExecutor.types,
        },
    });
    const web3Wallets = createWeb3Wallets();
    const chainIdentifier = api.registry.chainSS58 as number;
    const wsp = await initTeeWorkerConnection(workerEndpoint);
    const requestId = 1;
    // const { mrEnclave, teeShieldingKey } = await getEnclave(api);
    const mrEnclave = ('0x' + '0'.repeat(64)) as HexString;

    return {
        teeWsClient: wsp,
        api,
        // teeShieldingKey,
        mrEnclave,
        web3Wallets,
        chainIdentifier,
        requestId,
    };
}

export function nextRequestId(context: IntegrationTestContext): number {
    const nextId = context.requestId + 1;
    context.requestId = nextId;
    return nextId;
}

async function initTeeWorkerConnection(endpoint: string): Promise<WebSocketAsPromised> {
    const options = {
        createWebSocket: (url: string) => new WebSocket(url),
        extractMessageData: (event: any) => event,
        packMessage: (data: any) => JSON.stringify(data),
        unpackMessage: (data: string | ArrayBuffer | Blob) => JSON.parse(data.toString()),
        attachRequestId: (data: any, requestId: string | number) => Object.assign({ id: requestId }, data),
        extractRequestId: (data: any) => data && data.id, // read requestId from message `id` field
    } as unknown;
    const websocket = new WebSocketAsPromised(endpoint, options as WsAsPromiseOptions);
    await websocket.open();
    return websocket;
}

async function getEnclave(api: ApiPromise): Promise<{
    mrEnclave: HexString;
    teeShieldingKey: KeyObject;
}> {
    const enclaveIdentifier = api.createType('Vec<AccountId>', await api.query.teebag.enclaveIdentifier('Identity'));
    const primaryEnclave = (await api.query.teebag.enclaveRegistry(enclaveIdentifier[0])).unwrap();

    const shieldingPubkeyBytes = api.createType('Option<Bytes>', primaryEnclave.shieldingPubkey).unwrap();
    const shieldingPubkey = hexToString(shieldingPubkeyBytes.toHex());

    const teeShieldingKey = createPublicKey({
        key: {
            alg: 'RSA-OAEP-256',
            kty: 'RSA',
            use: 'enc',
            n: Buffer.from(JSON.parse(shieldingPubkey).n.reverse()).toString('base64url'),
            e: Buffer.from(JSON.parse(shieldingPubkey).e.reverse()).toString('base64url'),
        },
        format: 'jwk',
    });
    //@TODO mrEnclave should verify from storage
    const mrEnclave = primaryEnclave.mrenclave.toHex();
    return {
        mrEnclave,
        teeShieldingKey,
    };
}
