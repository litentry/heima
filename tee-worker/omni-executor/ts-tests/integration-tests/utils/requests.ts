import { ApiPromise } from '@polkadot/api';
import { u8aToHex } from '@polkadot/util';
import {
    NativeCallAuthenticatedOperation,
    NativeOperationResponse,
    NativeQueryAuthenticatedOperation,
} from 'parachain-api';
import { createPublicKey } from 'crypto';
import { IntegrationTestContext, nextRequestId } from './context';
import { decodeRpcBytesAsString } from './helpers';
import { createPlainRequest } from './type_creators';
import WebSocketAsPromised from 'websocket-as-promised';

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

export async function sendPlainRequestFromNativeCall(
    context: IntegrationTestContext,
    operation: NativeCallAuthenticatedOperation,
    onMessageReceived?: (response: NativeOperationResponse) => void
) {
    const plainRequest = createPlainRequest(context.api, context.mrEnclave, operation);

    const request = createJsonRpcRequest(
        'native_submitCallPlainRequest',
        [u8aToHex(plainRequest.toU8a())],
        nextRequestId(context)
    );

    return sendRequest(context.teeWsClient, request, context.api, onMessageReceived);
}

export async function sendPlainRequestFromNativeQuery(
    context: IntegrationTestContext,
    operation: NativeQueryAuthenticatedOperation,
    onMessageReceived?: (response: NativeOperationResponse) => void
) {
    const plainRequest = createPlainRequest(context.api, context.mrEnclave, operation);

    const request = createJsonRpcRequest(
        'native_submitQueryPlainRequest',
        [u8aToHex(plainRequest.toU8a())],
        nextRequestId(context)
    );

    return sendRequest(context.teeWsClient, request, context.api, onMessageReceived);
}

async function sendRequest(
    wsClient: WebSocketAsPromised,
    request: JsonRpcRequest,
    api: ApiPromise,
    onMessageReceived?: (response: NativeOperationResponse) => void
): Promise<NativeOperationResponse> {
    const p = new Promise<NativeOperationResponse>((resolve, reject) =>
        wsClient.onMessage.addListener((data) => {
            const parsed = JSON.parse(data);
            console.log('parsed:', JSON.stringify(parsed, null, 2));
            if (parsed.id !== request.id) {
                return;
            }
            if ('error' in parsed) {
                const transaction = { request, response: parsed };
                console.log('Request failed: ' + JSON.stringify(transaction, null, 2));
                reject(new Error(parsed.error.message, { cause: transaction }));
            }
            const response = api.createType('NativeOperationResponse', parsed.result);
            if (onMessageReceived) onMessageReceived(response);
            wsClient.onMessage.removeAllListeners();
            resolve(response);
        })
    );
    wsClient.sendRequest(request);
    return p;
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
