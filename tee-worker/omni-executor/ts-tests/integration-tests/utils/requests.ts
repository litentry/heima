import { u8aToHex } from '@polkadot/util';
import { ApiPromise } from '@polkadot/api';
import { NativeTaskWrapper, NativeTaskResponse } from '@heima-network/api-argument/omni';
import { createPublicKey } from 'crypto';
import { IntegrationTestContext, nextRequestId } from './context';
import { decodeRpcBytesAsString } from './helpers';
import { createRawTaskPlain } from './type_creators';
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

export async function sendRawTaskPlain(
    context: IntegrationTestContext,
    nativeTaskWrapper: NativeTaskWrapper,
    onMessageReceived?: (response: NativeTaskResponse) => void
) {
    const plainTask = createRawTaskPlain(context.api, nativeTaskWrapper);

    const request = createJsonRpcRequest(
        'omni_submitNativeTask',
        [u8aToHex(plainTask.toU8a())],
        nextRequestId(context)
    );

    return sendRequest(context.teeWsClient, request, context.api, onMessageReceived);
}

export type SignMessagePayload = {
    client_id: string;
    omni_account: string;
    message_code: string;
};

/**
 * Retrieves the message to sign for a given client ID and Omni account.
 */
export async function getMessageToSign(
    context: IntegrationTestContext,
    clientId: string,
    omniAccount: string
): Promise<SignMessagePayload> {
    const request = createJsonRpcRequest(
        'omni_getMessageCode',
        { client_id: clientId, omni_account: omniAccount },
        nextRequestId(context)
    );

    const response = new Promise<SignMessagePayload>((resolve, reject) =>
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
            const response = parsed.result;
            context.teeWsClient.onMessage.removeAllListeners();
            resolve(response);
        })
    );
    context.teeWsClient.sendRequest(request);
    return response;
}

async function sendRequest(
    wsClient: WebSocketAsPromised,
    request: JsonRpcRequest,
    api: ApiPromise,
    onMessageReceived?: (response: NativeTaskResponse) => void
): Promise<NativeTaskResponse> {
    const p = new Promise<NativeTaskResponse>((resolve, reject) =>
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
            const response = api.createType('NativeTaskResponse', parsed.result);
            if (onMessageReceived) onMessageReceived(response);
            wsClient.onMessage.removeAllListeners();
            resolve(response);
        })
    );
    wsClient.sendRequest(request);
    return p;
}

export const getTeeShieldingKey = async (context: IntegrationTestContext) => {
    const request = createJsonRpcRequest('omni_getShieldingKey', Uint8Array.from([]), nextRequestId(context));
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
