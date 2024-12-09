import { WsProvider, ApiPromise, CorePrimitivesTeebagTypesEnclave } from 'parachain-api';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { hexToString } from '@polkadot/util';
import WebSocketAsPromised from 'websocket-as-promised';
import WebSocket from 'ws';
import Options from 'websocket-as-promised/types/options';
import { KeyObject } from 'crypto';
import { getSidechainMetadata } from '../call';
import { createWeb3Wallets } from '../helpers';
import type { IntegrationTestContext } from '../common-types';
import { identity, vc, trusted_operations, sidechain, omniAccount } from 'parachain-api';
import crypto from 'crypto';
import type { HexString } from '@polkadot/util/types';
// maximum block number that we wait in listening events before we timeout
export const defaultListenTimeoutInBlockNumber = 15;

export async function initWorkerConnection(endpoint: string): Promise<WebSocketAsPromised> {
    const wsp = new WebSocketAsPromised(endpoint, <Options>(<unknown>{
        createWebSocket: (url: any) => new WebSocket(url),
        extractMessageData: (event: any) => event,
        packMessage: (data: any) => JSON.stringify(data),
        unpackMessage: (data: string | ArrayBuffer | Blob) => JSON.parse(data.toString()),
        attachRequestId: (data: any, requestId: string | number) => Object.assign({ id: requestId }, data),
        extractRequestId: (data: any) => data && data.id, // read requestId from message `id` field
    }));
    await wsp.open();
    return wsp;
}

export async function initIntegrationTestContext(
    parachainEndpoint?: string,
    enclaveEndpoint?: string
): Promise<IntegrationTestContext> {
    const provider = new WsProvider(parachainEndpoint);
    await cryptoWaitReady();

    const web3Wallets = createWeb3Wallets();

    const types = {
        ...identity.types,
        ...vc.types,
        ...trusted_operations.types,
        ...sidechain.types,
        ...omniAccount.types,
    };

    const api = await ApiPromise.create({
        provider,
        types,
    });

    const chainIdentifier = api.registry.chainSS58 as number;

    const workerEndpoint = enclaveEndpoint ? enclaveEndpoint : await getenclaveEndpoint(api);

    const wsp = await initWorkerConnection(workerEndpoint);
    const requestId = 1;

    const { sidechainMetaData, sidechainRegistry } = await getSidechainMetadata(wsp, api, requestId);
    const { mrEnclave, teeShieldingKey } = await getEnclave(api);
    return {
        tee: wsp,
        api,
        teeShieldingKey,
        mrEnclave,
        web3Wallets,
        sidechainMetaData,
        sidechainRegistry,
        chainIdentifier,
        requestId,
    };
}

async function getenclaveEndpoint(api: ApiPromise): Promise<string> {
    const registry = await api.query.teebag.enclaveRegistry.entries();
    const identityEnclaves = registry.reduce((enclaves, [, enclave]) => {
        if (enclave.isEmpty || !enclave.unwrap().workerType.isIdentity) {
            return enclaves;
        }
        return [...enclaves, enclave.unwrap()];
    }, [] as CorePrimitivesTeebagTypesEnclave[]);

    if (identityEnclaves.length === 0) {
        throw new Error('No identity worker found');
    }

    return identityEnclaves[Math.floor(Math.random() * identityEnclaves.length)].url.toHuman() as string;
}

export async function getEnclave(api: ApiPromise): Promise<{
    mrEnclave: HexString;
    teeShieldingKey: KeyObject;
}> {
    const enclaveIdentifier = api.createType('Vec<AccountId>', await api.query.teebag.enclaveIdentifier('Identity'));
    const primaryEnclave = (await api.query.teebag.enclaveRegistry(enclaveIdentifier[0])).unwrap();

    const shieldingPubkeyBytes = api.createType('Option<Bytes>', primaryEnclave.shieldingPubkey).unwrap();
    const shieldingPubkey = hexToString(shieldingPubkeyBytes.toHex());

    const teeShieldingKey = crypto.createPublicKey({
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
