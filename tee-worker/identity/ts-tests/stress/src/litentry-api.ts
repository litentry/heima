import { KeyringPair } from '@polkadot/keyring/types';
import {
    hexToU8a,
    compactStripLength,
    compactAddLength,
    u8aToString,
    bufferToU8a,
    u8aToHex,
    u8aConcat,
    stringToU8a,
    hexToString,
} from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import crypto, { KeyObject, createPublicKey } from 'crypto';
import { TypeRegistry as SidechainTypeRegistry, Metadata as SidechainMetadata } from 'sidechain-api';
import {
    Bytes,
    Codec,
    FrameSystemEventRecord,
    Getter,
    ApiPromise as ParachainApiPromise,
    WorkerRpcReturnValue,
    LitentryValidationData,
    Assertion,
    CorePrimitivesIdentity,
    TrustedCallSigned,
    PublicGetter,
} from 'parachain-api';
import WebSocketAsPromised from 'websocket-as-promised';
import { Index } from '@polkadot/types/interfaces';
import { Option, u32, u8, Vector } from 'scale-ts';
import WsWebSocket from 'ws';
import type { HexString } from '@polkadot/util/types';
import { ethers } from 'ethers';

async function logLine(log: WritableStream<string>, message: string): Promise<void> {
    const writer = log.getWriter();
    await writer.write(`${message}\n`);
    await writer.releaseLock();
}

export type Wallet =
    | {
          type: 'substrate';
          keyringPair: KeyringPair;
      }
    | { type: 'evm'; wallet: ethers.Wallet };

export type Api = {
    parachainApi: ParachainApiPromise;
    mrEnclave: `0x${string}`;
    teeShieldingKey: crypto.KeyObject;
    teeWorker: WebSocketAsPromised;
    sidechainRegistry: SidechainTypeRegistry;
};

function encryptWithAes(key: string, nonce: Uint8Array, cleartext: Buffer): HexString {
    const secretKey = crypto.createSecretKey(hexToU8a(key));
    const cipher = crypto.createCipheriv('aes-256-gcm', secretKey, nonce, {
        authTagLength: 16,
    });
    let encrypted = cipher.update(cleartext.toString('hex'), 'hex', 'hex');
    encrypted += cipher.final('hex');
    encrypted += cipher.getAuthTag().toString('hex');
    return `0x${encrypted}`;
}

function generateVerificationMessage(
    parachainApi: ParachainApiPromise,
    signer: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    sidechainNonce: number
): HexString {
    const encodedIdentity = parachainApi.createType('CorePrimitivesIdentity', identity).toU8a();
    const encodedWho = parachainApi.createType('CorePrimitivesIdentity', signer).toU8a();
    const encodedSidechainNonce = parachainApi.createType('Index', sidechainNonce);
    const msg = Buffer.concat([encodedSidechainNonce.toU8a(), encodedWho, encodedIdentity]);
    return blake2AsHex(msg, 256);
}

export async function buildValidation(
    parachainApi: ParachainApiPromise,
    sidechainRegistry: SidechainTypeRegistry,
    signerIdentity: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    startingSidechainNonce: number,
    signer: Wallet
): Promise<LitentryValidationData> {
    const message = generateVerificationMessage(parachainApi, signerIdentity, identity, startingSidechainNonce);

    return parachainApi.createType('LitentryValidationData', {
        Web3Validation:
            signer.type === 'substrate'
                ? {
                      Substrate: {
                          message,
                          signature: {
                              Sr25519: u8aToHex(signer.keyringPair.sign(message)),
                          },
                      },
                  }
                : {
                      Evm: {
                          message,
                          signature: {
                              Ethereum: await signer.wallet.signMessage(ethers.utils.arrayify(message)),
                          },
                      },
                  },
    });
}

export async function buildIdentityFromWallet(
    wallet: Wallet,
    api: ParachainApiPromise
): Promise<CorePrimitivesIdentity> {
    if (wallet.type === 'evm') {
        const identity = {
            Evm: wallet.wallet.address,
        };

        return api.createType('CorePrimitivesIdentity', identity);
    }

    const { keyringPair } = wallet;

    const type: string = (() => {
        switch (keyringPair.type) {
            case 'ethereum':
                return 'Evm';
            case 'sr25519':
                return 'Substrate';
            case 'ed25519':
                return 'Substrate';
            case 'ecdsa':
                return 'Substrate';
            default:
                return 'Substrate';
        }
    })();
    const address = keyringPair.addressRaw;
    const identity = {
        [type]: address,
    };

    return api.createType('CorePrimitivesIdentity', identity);
}

export function decodeRpcBytesAsString(value: Bytes): string {
    return u8aToString(compactStripLength(hexToU8a(value.toHex()))[1]);
}

type RequestBody = {
    id: number;
    jsonrpc: string;
    method: string;
};

export async function sendRequest(
    wsClient: WebSocketAsPromised,
    request: RequestBody,
    api: ParachainApiPromise,
    log: WritableStream<string>
): Promise<WorkerRpcReturnValue> {
    const rawRes = await wsClient.sendRequest(request, { requestId: 1, timeout: 6000 });
    const res: WorkerRpcReturnValue = api.createType('WorkerRpcReturnValue', rawRes.result);
    if (res.status.isError) {
        logLine(log, 'Rpc response error: ' + decodeRpcBytesAsString(res.value));
    }

    // unfortunately, the res.value only contains the hash of top
    if (res.status.isTrustedOperationStatus && res.status.asTrustedOperationStatus[0].isInvalid) {
        logLine(log, `Rpc trusted operation execution failed, hash: ${res.value}`);
    }

    return res;
}

export async function getSidechainMetadata(
    wsClient: WebSocketAsPromised,
    parachainApi: ParachainApiPromise,
    log: WritableStream<string>
): Promise<{ sidechainMetaData: SidechainMetadata; sidechainRegistry: SidechainTypeRegistry }> {
    const request = { jsonrpc: '2.0', method: 'state_getMetadata', params: [], id: 1 };
    const resp = await sendRequest(wsClient, request, parachainApi, log);

    const sidechainRegistry = new SidechainTypeRegistry();
    const sidechainMetaData = new SidechainMetadata(sidechainRegistry, resp.value);

    sidechainRegistry.setMetadata(sidechainMetaData);
    return { sidechainMetaData, sidechainRegistry };
}

export async function initWorkerConnection(
    endpoint: string,
    log: WritableStream<string>
): Promise<WebSocketAsPromised> {
    const wsp = new WebSocketAsPromised(endpoint, {
        createWebSocket: (url: string) => {
            const socket = new WsWebSocket(url);
            socket.on('error', (error) => {
                logLine(log, `Socket to ${url} caught error ${JSON.stringify(error)})}`);
            });
            return socket as unknown as WebSocket;
        },
        extractMessageData: (event: any) => event,
        packMessage: (data: any) => JSON.stringify(data),
        unpackMessage: (data: string | ArrayBuffer | Blob) => JSON.parse(data.toString()),
        attachRequestId: (data: any, requestId: string | number) => Object.assign({ id: requestId }, data),
        extractRequestId: (data: any) => data && data.id, // read requestId from message `id` field
    });
    await wsp.open();
    return wsp;
}

const createPublicGetter = (
    parachainApi: ParachainApiPromise,
    publicGetter: [string, string],
    params: any
): PublicGetter => {
    const [variant, argType] = publicGetter;
    const getter = parachainApi.createType('PublicGetter', {
        [variant]: parachainApi.createType(argType, params),
    });

    return getter;
};

export const createSignedTrustedGetter = (
    parachainApi: ParachainApiPromise,
    trustedGetter: [string, string],
    signer: KeyringPair,
    params: any
) => {
    const [variant, argType] = trustedGetter;
    const getter = parachainApi.createType('TrustedGetter', {
        [variant]: parachainApi.createType(argType, params),
    });
    const payload = getter.toU8a();
    const signature = parachainApi.createType('LitentryMultiSignature', {
        [signer.type]: signer.sign(payload),
    });
    return parachainApi.createType('TrustedGetterSigned', {
        getter: getter,
        signature: signature,
    });
};

export function createSignedTrustedGetterUserShieldingKey(
    parachainApi: ParachainApiPromise,
    signer: KeyringPair,
    subject: CorePrimitivesIdentity
) {
    const getterSigned = createSignedTrustedGetter(
        parachainApi,
        ['user_shielding_key', '(HeimaIdentity)'],
        signer,
        subject.toHuman()
    );
    return parachainApi.createType('Getter', { trusted: getterSigned });
}

const sendRequestFromGetter = async (
    teeWorker: WebSocketAsPromised,
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    teeShieldingKey: KeyObject,
    getter: Getter,
    log: WritableStream<string>
): Promise<WorkerRpcReturnValue> => {
    // important: we don't create the `TrustedOperation` type here, but use `Getter` type directly
    //            this is what `state_executeGetter` expects in rust
    const requestParam = await createRsaRequest(
        teeWorker,
        parachainApi,
        mrenclave,
        teeShieldingKey,
        true,
        getter.toU8a()
    );
    const request = {
        jsonrpc: '2.0',
        method: 'state_executeGetter',
        params: [u8aToHex(requestParam)],
        id: 1,
    };
    return sendRequest(teeWorker, request, parachainApi, log);
};

function encryptWithTeeShieldingKey(teeShieldingKey: KeyObject, plaintext: Uint8Array): Buffer {
    return crypto.publicEncrypt(
        {
            key: teeShieldingKey,
            padding: crypto.constants.RSA_PKCS1_OAEP_PADDING,
            oaepHash: 'sha256',
        },
        plaintext
    );
}

const createRsaRequest = async (
    wsp: WebSocketAsPromised,
    parachainApi: ParachainApiPromise,
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

function decodeNonce(nonceInHex: string) {
    const optionalType = Option(Vector(u8));
    const encodedNonce = optionalType.dec(nonceInHex) as number[];
    const nonce = u32.dec(new Uint8Array(encodedNonce));
    return nonce;
}

export const getSidechainNonce = async (
    teeWorker: WebSocketAsPromised,
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    teeShieldingKey: KeyObject,
    subject: CorePrimitivesIdentity,
    log: WritableStream<string>
): Promise<Index> => {
    const getterPublic = createPublicGetter(parachainApi, ['nonce', '(HeimaIdentity)'], subject.toHuman());
    const getter = parachainApi.createType('Getter', { public: getterPublic });
    const nonce = await sendRequestFromGetter(teeWorker, parachainApi, mrenclave, teeShieldingKey, getter, log);
    const nonceValue = decodeNonce(nonce.value.toHex());
    return parachainApi.createType('Index', nonceValue) as Index;
};

export async function getEnclave(api: ParachainApiPromise): Promise<{
    mrEnclave: `0x${string}`;
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

const createSignedTrustedCall = async (
    parachainApi: ParachainApiPromise,
    trustedCall: [string, string],
    signer: Wallet,
    // hex-encoded mrenclave, retrieveable from parachain enclave registry
    // TODO: do we have a RPC getter from the enclave?
    mrenclave: string,
    nonce: Codec,
    params: any,
    withWrappedBytes = false
): Promise<TrustedCallSigned> => {
    const [variant, argType] = trustedCall;
    const call = parachainApi.createType('TrustedCall', {
        [variant]: parachainApi.createType(argType, params),
    });
    let payload = Uint8Array.from([
        ...call.toU8a(),
        ...nonce.toU8a(),
        ...hexToU8a(mrenclave),
        ...hexToU8a(mrenclave), // should be shard, but it's the same as MRENCLAVE in our case
    ]);
    if (withWrappedBytes) {
        payload = u8aConcat(stringToU8a('<Bytes>'), payload, stringToU8a('</Bytes>'));
    }
    const signature =
        signer.type === 'substrate'
            ? parachainApi.createType('LitentryMultiSignature', {
                  [signer.keyringPair.type]: u8aToHex(signer.keyringPair.sign(payload)),
              })
            : parachainApi.createType('LitentryMultiSignature', {
                  ['ethereum']: await signer.wallet.signMessage(payload),
              });
    return parachainApi.createType('TrustedCallSigned', {
        call: call,
        index: nonce,
        signature: signature,
    });
};

export const subscribeToEventsWithExtHash = async (
    requestIdentifier: string,
    parachainApi: ParachainApiPromise
): Promise<FrameSystemEventRecord[]> => {
    return new Promise<FrameSystemEventRecord[]>((resolve, reject) => {
        /*
        WARNING:The unsubscribe function is called inside the Promise callback, which is executed each time a new blockHeader is received.
               `unsubscribe` is intended to unsubscribe a blockHeader if certain conditions are met.
                If you use await, you will actually wait for this function to finish executing.
                However, since it doesn't return a Promise, using await doesn't make sense and can lead to problematic code behaviour.
                soooooo, don't use await here
        */
        const unsubscribe = parachainApi.rpc.chain.subscribeNewHeads(async (blockHeader) => {
            const shiftedApi = await parachainApi.at(blockHeader.hash);

            const allBlockEvents = await shiftedApi.query.system.events();
            const allExtrinsicEvents = allBlockEvents.filter(({ phase }) => phase.isApplyExtrinsic);

            const matchingEvent = allExtrinsicEvents.find((eventRecord) => {
                const eventData = eventRecord.event.data.toHuman();
                return (
                    eventData != undefined &&
                    typeof eventData === 'object' &&
                    'reqExtHash' in eventData &&
                    eventData.reqExtHash === requestIdentifier
                );
            });

            if (matchingEvent == undefined) {
                return;
            }

            const extrinsicIndex = matchingEvent.phase.asApplyExtrinsic;
            const requestEvents = allExtrinsicEvents.filter((eventRecord) =>
                eventRecord.phase.asApplyExtrinsic.eq(extrinsicIndex)
            );

            resolve(requestEvents);
            (await unsubscribe)();
        });
    });
};

export const sendRequestFromTrustedCall = async (
    wsp: WebSocketAsPromised,
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    teeShieldingKey: KeyObject,
    call: TrustedCallSigned,
    log: WritableStream<string>
) => {
    // construct trusted operation
    const trustedOperation = parachainApi.createType('TrustedOperation', { direct_call: call });
    // create the request parameter
    const requestParam = await createRsaRequest(
        wsp,
        parachainApi,
        mrenclave,
        teeShieldingKey,
        false,
        trustedOperation.toU8a()
    );
    const request = {
        jsonrpc: '2.0',
        method: 'author_submitAndWatchRsRequest',
        params: [u8aToHex(requestParam)],
        id: 1,
    };
    return sendRequest(wsp, request, parachainApi, log);
};

export function createSignedTrustedCallSetUserShieldingKey(
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Wallet,
    subject: CorePrimitivesIdentity,
    key: string,
    hash: string,
    withWrappedBytes = false
) {
    return createSignedTrustedCall(
        parachainApi,
        ['set_user_shielding_key', '(HeimaIdentity, HeimaIdentity, RequestAesKey, H256)'],
        signer,
        mrenclave,
        nonce,
        [subject.toHuman(), subject.toHuman(), key, hash],
        withWrappedBytes
    );
}

export function createSignedTrustedCallLinkIdentity(
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Wallet,
    subject: CorePrimitivesIdentity,
    identity: string,
    validationData: string,
    web3networks: string,
    aesKey: string,
    hash: string
) {
    return createSignedTrustedCall(
        parachainApi,
        [
            'link_identity',
            '(HeimaIdentity, HeimaIdentity, HeimaIdentity, HeimaValidationData, Vec<Web3Network>, Option<RequestAesKey>, H256)',
        ],
        signer,
        mrenclave,
        nonce,
        [subject.toHuman(), subject.toHuman(), identity, validationData, web3networks, aesKey, hash]
    );
}

export function createSignedTrustedCallDeactivateIdentity(
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Wallet,
    subject: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    hash: string
) {
    return createSignedTrustedCall(
        parachainApi,
        ['deactivate_identity', '(HeimaIdentity, HeimaIdentity, HeimaIdentity, H256)'],
        signer,
        mrenclave,
        nonce,
        [subject.toHuman(), subject.toHuman(), identity.toHuman(), hash]
    );
}

export function createSignedTrustedCallActivateIdentity(
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Wallet,
    subject: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    hash: string
) {
    return createSignedTrustedCall(
        parachainApi,
        ['activate_identity', '(HeimaIdentity, HeimaIdentity, HeimaIdentity, H256)'],
        signer,
        mrenclave,
        nonce,
        [subject.toHuman(), subject.toHuman(), identity.toHuman(), hash]
    );
}

export function createSignedTrustedCallRequestVc(
    parachainApi: ParachainApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Wallet,
    subject: CorePrimitivesIdentity,
    assertion: Assertion,
    key: string,
    hash: string
) {
    return createSignedTrustedCall(
        parachainApi,
        ['request_vc', '(HeimaIdentity,HeimaIdentity,Assertion,Option<RequestAesKey>,H256)'],
        signer,
        mrenclave,
        nonce,
        [subject.toHuman(), subject.toHuman(), assertion, key, hash]
    );
}
