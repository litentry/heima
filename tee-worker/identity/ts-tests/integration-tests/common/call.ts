import { ApiPromise } from '@polkadot/api';
import { hexToU8a, compactStripLength, u8aToString } from '@polkadot/util';
import WebSocketAsPromised from 'websocket-as-promised';
import type { JsonRpcRequest } from './common-types';
import type { WorkerRpcReturnValue } from '@heima-network/api-argument/identity';
import { Metadata, TypeRegistry } from '@polkadot/types';
import type { Bytes } from '@polkadot/types-codec';
import { createJsonRpcRequest } from './helpers';
// TODO:
// - better place to put these constants?
// - maybe randomise it in test initialisation
//
// the aes key to encrypt an AES request
export const aesKey = '0x22fc82db5b606998ad45099b7978b5b4f9dd4ea6017e57370ac56141caaabd12';

// nonce to encrypt an AES request together with `aesKey`
export const keyNonce = '0x010101010101010101010101';

// send RPC request
export async function sendRequest(
    wsClient: WebSocketAsPromised,
    request: JsonRpcRequest,
    api: ApiPromise
): Promise<WorkerRpcReturnValue> {
    const rawRes = await wsClient.sendRequest(request, { requestId: request.id, timeout: 6000 });

    const res = api.createType('WorkerRpcReturnValue', rawRes.result);
    if (res.status.isError) {
        console.log('Rpc response error: ' + decodeRpcBytesAsString(res.value));
    }

    // unfortunately, the res.value only contains the hash of top
    if (res.status.isTrustedOperationStatus && res.status.asTrustedOperationStatus[0].isInvalid) {
        console.log('Rpc trusted operation execution failed, hash: ', res.value);
    }

    return res;
}

// decode the returned bytes as string
// TODO: is it same as `String::decode` in rust?
// please note we shouldn't use toU8a(), which encodes the Bytes instead of converting
export function decodeRpcBytesAsString(value: Bytes): string {
    return u8aToString(compactStripLength(hexToU8a(value.toHex()))[1]);
}

export async function getSidechainMetadata(
    wsClient: WebSocketAsPromised,
    api: ApiPromise,
    requestId: number
): Promise<{ sidechainMetaData: Metadata; sidechainRegistry: TypeRegistry }> {
    const request = createJsonRpcRequest('state_getMetadata', Uint8Array.from([]), requestId);

    const resp = await sendRequest(wsClient, request, api);

    const sidechainRegistry = new TypeRegistry();
    const sidechainMetaData = new Metadata(sidechainRegistry, resp.value);

    sidechainRegistry.setMetadata(sidechainMetaData);
    return { sidechainMetaData, sidechainRegistry };
}
