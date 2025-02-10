import { hexToU8a, compactStripLength, u8aToString, hexToString } from '@polkadot/util';
import { ApiPromise, NativeCall, NativeCallAuthenticated } from 'parachain-api';
import { HexString } from '@polkadot/util/types';
import { Codec } from '@polkadot/types-codec/types';
import { createPublicKey } from 'crypto';
import { IntegrationTestContext, nextRequestId } from './context';

type JsonRpcRequest = {
    jsonrpc: string;
    method: string;
    params: unknown;
    id: number;
};

function createJsonRpcRequest(method: string, params: unknown, id: number): JsonRpcRequest {
    return {
        jsonrpc: '2.0',
        method,
        params,
        id,
    };
}

export const getTeeShieldingKey = async (context: IntegrationTestContext) => {
    const request = createJsonRpcRequest('native_getShieldingKey', Uint8Array.from([]), nextRequestId(context));
    const response = new Promise<string>((resolve, reject) =>
        context.teeWsClient.onMessage.addListener((data) => {
            const parsed = JSON.parse(data);
            if (parsed.id !== request.id) {
                return;
            }
            if ('error' in parsed) {
                const transaction = { request, response: parsed };
                console.log('Request failed: ' + JSON.stringify(transaction, null, 2));
                reject(new Error(parsed.error.message, { cause: transaction }));
            }
            const response = decodeRpcBytesAsString(parsed.result);
            context.teeWsClient.onMessage.removeAllListeners();
            resolve(response);
        })
    );
    context.teeWsClient.sendRequest(request);
    const res = await response;

    const shieldingKey = JSON.parse(res) as {
        n: Uint8Array;
        e: Uint8Array;
    };

    console.log('shieldingKey:', shieldingKey);

    return createPublicKey({
        key: {
            alg: 'RSA-OAEP-256',
            kty: 'RSA',
            use: 'enc',
            n: Buffer.from(shieldingKey.n.reverse()).toString('base64url'),
            e: Buffer.from(shieldingKey.e.reverse()).toString('base64url'),
        },
        format: 'jwk',
    });
};

function decodeRpcBytesAsString(value: HexString): string {
    return u8aToString(compactStripLength(hexToU8a(value))[1]);
}
