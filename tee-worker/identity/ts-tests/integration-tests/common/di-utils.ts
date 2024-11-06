import { ApiPromise } from '@polkadot/api';
import { u8aToHex, hexToU8a, compactAddLength, bufferToU8a, u8aConcat, stringToU8a } from '@polkadot/util';
import { Codec } from '@polkadot/types/types';
import { TypeRegistry } from '@polkadot/types';
import { Bytes } from '@polkadot/types-codec';
import { IntegrationTestContext, JsonRpcRequest } from './common-types';
import type {
    WorkerRpcReturnValue,
    TrustedCallSigned,
    Getter,
    CorePrimitivesIdentity,
    TrustedGetterSigned,
    TrustedCall,
} from 'parachain-api';
import {
    encryptWithTeeShieldingKey,
    Signer,
    encryptWithAes,
    sleep,
    createLitentryMultiSignature,
    decryptWithAes,
} from './utils';
import { aesKey, decodeRpcBytesAsString, keyNonce } from './call';
import { createPublicKey, KeyObject } from 'crypto';
import WebSocketAsPromised from 'websocket-as-promised';
import { H256, Index } from '@polkadot/types/interfaces';
import { blake2AsHex, base58Encode, blake2AsU8a } from '@polkadot/util-crypto';
import { createJsonRpcRequest, nextRequestId, stfErrorToString } from './helpers';

// Send the request to worker ws
// we should perform different actions based on the returned status:
//
// `Submitted`:
// the request is submitted to the top pool, we should start to subscribe to parachain headers to wait for async parachain event
//
// `InSidechainBlock`
// the request is included in a sidechain block: the state mutation of sidechain is done, the promise is resolved
// the corresponding parachain event should be emitted **around** that, it's not guaranteed if it's before or after this status
// due to block inclusion delays from the parachain
//
export async function sendRequest(
    wsClient: WebSocketAsPromised,
    request: JsonRpcRequest,
    api: ApiPromise,
    onMessageReceived?: (res: WorkerRpcReturnValue) => void
): Promise<WorkerRpcReturnValue> {
    const p = new Promise<WorkerRpcReturnValue>((resolve, reject) =>
        wsClient.onMessage.addListener((data) => {
            const parsed = JSON.parse(data);
            if (parsed.id !== request.id) {
                return;
            }

            if ('error' in parsed) {
                const transaction = { request, response: parsed };
                console.log('Request failed: ' + JSON.stringify(transaction, null, 2));
                reject(new Error(parsed.error.message, { cause: transaction }));
            }

            const result = parsed.result;
            const res = api.createType('WorkerRpcReturnValue', result);

            if (res.status.isError) {
                console.log('Rpc response error: ' + decodeRpcBytesAsString(res.value));
            }

            if (res.status.isTrustedOperationStatus && res.status.asTrustedOperationStatus[0].isInvalid) {
                console.log('Rpc trusted operation execution failed, hash: ', res.value.toHex());
                const stfError = api.createType('StfError', res.value);
                const msg = stfErrorToString(stfError);
                console.log('TrustedOperationStatus error: ', msg);
            }
            // sending every response we receive from websocket
            if (onMessageReceived) onMessageReceived(res);

            // resolve it once `do_watch` is false, meaning it's the final response
            if (res.do_watch.isFalse) {
                // TODO: maybe only remove this listener
                wsClient.onMessage.removeAllListeners();
                resolve(res);
            } else {
                // `do_watch` is true means: hold on - there's still something coming
                console.log('do_watch is true, continue watching ...');
            }
        })
    );

    wsClient.sendRequest(request);
    return p;
}

// TrustedCalls are defined in:
// https://github.com/litentry/litentry-parachain/blob/d4be11716fdb46021194bbe9fe791b15249a369e/tee-worker/app-libs/stf/src/trusted_call.rs#L61
//
// About the signature, it's signed with `KeyringPair` here.
// In reality we need to get the user's signature on the `payload`.
export const createSignedTrustedCall = async (
    parachainApi: ApiPromise,
    trustedCall: [string, string],
    signer: Signer,
    // hex-encoded mrenclave, retrieveable from parachain enclave registry
    // TODO: do we have a RPC getter from the enclave?
    mrenclave: string,
    nonce: Codec,
    params: any,
    withWrappedBytes = false,
    withPrefix = false
): Promise<TrustedCallSigned> => {
    const [variant, argType] = trustedCall;
    const call: TrustedCall = parachainApi.createType('TrustedCall', {
        [variant]: parachainApi.createType(argType, params),
    });
    let payload: string = blake2AsHex(
        u8aConcat(
            call.toU8a(),
            nonce.toU8a(),
            hexToU8a(mrenclave),
            hexToU8a(mrenclave) // should be shard, but it's the same as MRENCLAVE in our case
        ),
        256
    );

    if (withWrappedBytes) {
        payload = `<Bytes>${payload}</Bytes>`;
    }

    if (withPrefix) {
        const prefix = getSignatureMessagePrefix(call);
        const msg = prefix + payload;
        payload = msg;
        console.log('Signing message: ', payload);
    }

    const signature = await createLitentryMultiSignature(parachainApi, {
        signer,
        payload,
    });

    return parachainApi.createType('TrustedCallSigned', {
        call: call,
        index: nonce,
        signature: signature,
    });
};

// See TrustedCall.signature_message_prefix
export function getSignatureMessagePrefix(call: TrustedCall): string {
    if (call.isLinkIdentity) {
        return "By linking your identity to our platform, you're taking a step towards a more integrated experience. Please be assured, this process is safe and involves no transactions of your assets. Token: ";
    }

    if (call.isRequestBatchVc) {
        const [, , assertions] = call.asRequestBatchVc;
        const length = assertions.length;

        return `We are going to help you generate ${length} secure credential${
            length > 1 ? 's' : ''
        }. Please be assured, this process is safe and involves no transactions of your assets. Token: `;
    }

    return 'Token: ';
}

export const createSignedTrustedGetter = async (
    parachainApi: ApiPromise,
    trustedGetter: [string, string],
    signer: Signer,
    params: any
): Promise<TrustedGetterSigned> => {
    const [variant, argType] = trustedGetter;
    const getter = parachainApi.createType('TrustedGetter', {
        [variant]: parachainApi.createType(argType, params),
    });
    const payload = blake2AsU8a(getter.toU8a(), 256);

    let signature = await createLitentryMultiSignature(parachainApi, {
        signer,
        payload,
    });

    return parachainApi.createType('TrustedGetterSigned', {
        getter,
        signature,
    });
};

export const createPublicGetter = (parachainApi: ApiPromise, publicGetter: [string, string], params: any) => {
    const [variant, argType] = publicGetter;
    const getter = parachainApi.createType('PublicGetter', {
        [variant]: parachainApi.createType(argType, params),
    });

    return getter;
};

export async function createSignedTrustedCallLinkIdentity(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    primeIdentity: CorePrimitivesIdentity,
    identity: string,
    validationData: string,
    web3networks: string,
    aesKey: string,
    hash: string,
    options?: { withWrappedBytes?: boolean; withPrefix?: boolean }
) {
    return createSignedTrustedCall(
        parachainApi,
        [
            'link_identity',
            '(LitentryIdentity, LitentryIdentity, LitentryIdentity, LitentryValidationData, Vec<Web3Network>, Option<RequestAesKey>, H256)',
        ],
        signer,
        mrenclave,
        nonce,
        [primeIdentity.toHuman(), primeIdentity.toHuman(), identity, validationData, web3networks, aesKey, hash],
        options?.withWrappedBytes,
        options?.withPrefix
    );
}

export async function createSignedTrustedCallSetIdentityNetworks(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    primeIdentity: CorePrimitivesIdentity,
    identity: string,
    web3networks: string,
    aesKey: string,
    hash: string
) {
    return createSignedTrustedCall(
        parachainApi,
        [
            'set_identity_networks',
            '(LitentryIdentity, LitentryIdentity, LitentryIdentity, Vec<Web3Network>, Option<RequestAesKey>, H256)',
        ],
        signer,
        mrenclave,
        nonce,
        [primeIdentity.toHuman(), primeIdentity.toHuman(), identity, web3networks, aesKey, hash]
    );
}

export async function createSignedTrustedCallRequestVc(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    primeIdentity: CorePrimitivesIdentity,
    assertion: string,
    aesKey: string,
    hash: string,
    options?: { withWrappedBytes?: boolean; withPrefix?: boolean }
) {
    return await createSignedTrustedCall(
        parachainApi,
        ['request_vc', '(LitentryIdentity, LitentryIdentity, Assertion, Option<RequestAesKey>, H256)'],
        signer,
        mrenclave,
        nonce,
        [primeIdentity.toHuman(), primeIdentity.toHuman(), assertion, aesKey, hash],
        options?.withWrappedBytes,
        options?.withPrefix
    );
}

export async function createSignedTrustedCallRequestBatchVc(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    primeIdentity: CorePrimitivesIdentity,
    assertion: string,
    aesKey: string,
    hash: string,
    options?: { withWrappedBytes?: boolean; withPrefix?: boolean }
) {
    return await createSignedTrustedCall(
        parachainApi,
        [
            'request_batch_vc',
            '(LitentryIdentity, LitentryIdentity, BoundedVec<Assertion, ConstU32<32>>, Option<RequestAesKey>, H256)',
        ],
        signer,
        mrenclave,
        nonce,
        [primeIdentity.toHuman(), primeIdentity.toHuman(), assertion, aesKey, hash],
        options?.withWrappedBytes,
        options?.withPrefix
    );
}

export async function createSignedTrustedCallDeactivateIdentity(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    primeIdentity: CorePrimitivesIdentity,
    identity: string,
    aesKey: string,
    hash: string
) {
    return createSignedTrustedCall(
        parachainApi,
        ['deactivate_identity', '(LitentryIdentity, LitentryIdentity, LitentryIdentity, Option<RequestAesKey>, H256)'],
        signer,
        mrenclave,
        nonce,
        [primeIdentity.toHuman(), primeIdentity.toHuman(), identity, aesKey, hash]
    );
}
export async function createSignedTrustedCallActivateIdentity(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    primeIdentity: CorePrimitivesIdentity,
    identity: string,
    aesKey: string,
    hash: string
) {
    return createSignedTrustedCall(
        parachainApi,
        ['activate_identity', '(LitentryIdentity, LitentryIdentity, LitentryIdentity, Option<RequestAesKey>, H256)'],
        signer,
        mrenclave,
        nonce,
        [primeIdentity.toHuman(), primeIdentity.toHuman(), identity, aesKey, hash]
    );
}

export async function createSignedTrustedGetterIdGraph(
    parachainApi: ApiPromise,
    signer: Signer,
    primeIdentity: CorePrimitivesIdentity
): Promise<Getter> {
    const getterSigned = await createSignedTrustedGetter(
        parachainApi,
        ['id_graph', '(LitentryIdentity)'],
        signer,
        primeIdentity.toHuman()
    );
    return parachainApi.createType('Getter', { trusted: getterSigned });
}

export const getSidechainNonce = async (
    context: IntegrationTestContext,
    primeIdentity: CorePrimitivesIdentity
): Promise<Index> => {
    const request = createJsonRpcRequest(
        'author_getNextNonce',
        [base58Encode(hexToU8a(context.mrEnclave)), primeIdentity.toHex()],
        nextRequestId(context)
    );
    const res = await sendRequest(context.tee, request, context.api);
    const nonceHex = res.value.toHex();
    let nonce = 0;

    if (nonceHex) {
        nonce = context.api.createType('Index', '0x' + nonceHex.slice(2)?.match(/../g)?.reverse().join('')).toNumber();
    }

    return context.api.createType('Index', nonce);
};

export const getIdGraphHash = async (
    context: IntegrationTestContext,
    teeShieldingKey: KeyObject,
    primeIdentity: CorePrimitivesIdentity
): Promise<H256> => {
    const getterPublic = createPublicGetter(
        context.api,
        ['id_graph_hash', '(LitentryIdentity)'],
        primeIdentity.toHuman()
    );
    const getter = context.api.createType('Getter', { public: getterPublic });
    const res = await sendRsaRequestFromGetter(context, teeShieldingKey, getter);
    const hash = context.api.createType('Option<Bytes>', hexToU8a(res.value.toHex())).unwrap();
    return context.api.createType('H256', hash);
};

export const sendRequestFromTrustedCall = async (
    context: IntegrationTestContext,
    teeShieldingKey: KeyObject,
    call: TrustedCallSigned,
    onMessageReceived?: (res: WorkerRpcReturnValue) => void
) => {
    // construct trusted operation
    const trustedOperation = context.api.createType('TrustedOperation', { direct_call: call });
    console.log('trustedOperation: ', JSON.stringify(trustedOperation.toHuman(), null, 2));
    // create the request parameter
    const requestParam = await createAesRequest(
        context.api,
        context.mrEnclave,
        teeShieldingKey,
        hexToU8a(aesKey),
        trustedOperation.toU8a()
    );

    const isRequestVc = call.call.isRequestVc || call.call.isRequestBatchVc;
    const request = createJsonRpcRequest(
        isRequestVc ? 'author_requestVc' : 'author_submitAndWatchAesRequest',
        [u8aToHex(requestParam)],
        nextRequestId(context)
    );
    return sendRequest(context.tee, request, context.api, onMessageReceived);
};

/** @deprecated use `sendAesRequestFromGetter` instead */
export const sendRsaRequestFromGetter = async (
    context: IntegrationTestContext,
    teeShieldingKey: KeyObject,
    getter: Getter
): Promise<WorkerRpcReturnValue> => {
    // important: we don't create the `TrustedOperation` type here, but use `Getter` type directly
    //            this is what `state_executeGetter` expects in rust
    const requestParam = await createRsaRequest(context.api, context.mrEnclave, teeShieldingKey, true, getter.toU8a());
    const request = createJsonRpcRequest('state_executeGetter', [u8aToHex(requestParam)], nextRequestId(context));
    // in multiworker setup in some cases state might not be immediately propagated to other nodes so we wait 1 sec
    // hopefully we will query correct state
    await sleep(1);
    return sendRequest(context.tee, request, context.api);
};

export const sendAesRequestFromGetter = async (
    context: IntegrationTestContext,
    teeShieldingKey: KeyObject,
    aesKey: Uint8Array,
    getter: Getter
): Promise<WorkerRpcReturnValue> => {
    // important: we don't create the `TrustedOperation` type here, but use `Getter` type directly
    //            this is what `state_executeAesGetter` expects in rust
    const requestParam = await createAesRequest(
        context.api,
        context.mrEnclave,
        teeShieldingKey,
        aesKey,
        getter.toU8a()
    );
    const request = createJsonRpcRequest('state_executeAesGetter', [u8aToHex(requestParam)], nextRequestId(context));
    // in multiworker setup in some cases state might not be immediately propagated to other nodes so we wait 1 sec
    // hopefully we will query correct state
    await sleep(1);
    const res = await sendRequest(context.tee, request, context.api);
    const aesOutput = context.api.createType('AesOutput', res.value);
    const decryptedValue = decryptWithAes(u8aToHex(aesKey), aesOutput, 'hex');

    return context.api.createType('WorkerRpcReturnValue', {
        value: decryptedValue,
        do_watch: res.do_watch,
        status: res.status,
    });
};

// get TEE's shielding key directly via RPC
export const getTeeShieldingKey = async (context: IntegrationTestContext) => {
    const request = createJsonRpcRequest('author_getShieldingKey', Uint8Array.from([]), nextRequestId(context));
    const res = await sendRequest(context.tee, request, context.api);
    const k = JSON.parse(decodeRpcBytesAsString(res.value)) as {
        n: Uint8Array;
        e: Uint8Array;
    };

    return createPublicKey({
        key: {
            alg: 'RSA-OAEP-256',
            kty: 'RSA',
            use: 'enc',
            n: Buffer.from(k.n.reverse()).toString('base64url'),
            e: Buffer.from(k.e.reverse()).toString('base64url'),
        },
        format: 'jwk',
    });
};

// given an encoded trusted operation, construct a rsa request bytes that are sent in RPC request parameters
export const createRsaRequest = async (
    parachainApi: ApiPromise,
    mrenclave: string,
    teeShieldingKey: KeyObject,
    isGetter: boolean,
    top: Uint8Array
) => {
    let payload;
    if (isGetter) {
        payload = compactAddLength(top);
    } else {
        payload = compactAddLength(bufferToU8a(encryptWithTeeShieldingKey(teeShieldingKey, top)));
    }

    return parachainApi.createType('RsaRequest', { shard: hexToU8a(mrenclave), payload }).toU8a();
};

// given an encoded trusted operation, construct an aes request bytes that are sent in RPC request parameters
export const createAesRequest = async (
    parachainApi: ApiPromise,
    mrenclave: string,
    teeShieldingKey: KeyObject,
    aesKey: Uint8Array,
    top: Uint8Array
) => {
    const encryptedAesKey = compactAddLength(bufferToU8a(encryptWithTeeShieldingKey(teeShieldingKey, aesKey)));
    return parachainApi
        .createType('AesRequest', {
            shard: hexToU8a(mrenclave),
            key: encryptedAesKey,
            payload: parachainApi
                .createType('AesOutput', {
                    ciphertext: compactAddLength(
                        hexToU8a(encryptWithAes(u8aToHex(aesKey), hexToU8a(keyNonce), Buffer.from(top)))
                    ),
                    aad: hexToU8a('0x'),
                    nonce: hexToU8a(keyNonce),
                })
                .toU8a(),
        })
        .toU8a();
};

export function decodeIdGraph(sidechainRegistry: TypeRegistry, value: Bytes) {
    const idgraphBytes = sidechainRegistry.createType('Option<Bytes>', hexToU8a(value.toHex()));
    return sidechainRegistry.createType(
        'Vec<(CorePrimitivesIdentity, PalletIdentityManagementTeeIdentityContext)>',
        idgraphBytes.unwrap()
    );
}

export function getTopHash(parachainApi: ApiPromise, call: TrustedCallSigned) {
    const trustedOperation = parachainApi.createType('TrustedOperation', { direct_call: call });
    return blake2AsHex(trustedOperation.toU8a());
}
