import type { Enum } from '@polkadot/types-codec';
import { HexString } from '@polkadot/util/types';
import { u8aToHex, hexToU8a, stringToU8a, u8aConcat, compactAddLength } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import { Codec } from '@polkadot/types-codec/types';
import {
    ApiPromise,
    OmniAuth,
    CorePrimitivesIdentity,
    HeimaMultiSignature,
    NativeTask,
    OmniAccountPermission,
    RawRequest,
    NativeTaskWrapper,
} from 'parachain-api';
import { Signer } from './signer';

export async function createIdentityType(
    api: ApiPromise,
    address: HexString | string,
    type: CorePrimitivesIdentity['type']
): Promise<CorePrimitivesIdentity> {
    const identity = {
        [type]: address,
    };
    return api.createType('CorePrimitivesIdentity', identity);
}

export async function createHeimaMultiSignature(
    api: ApiPromise,
    args: { signer: Signer; payload: Uint8Array | string }
): Promise<HeimaMultiSignature> {
    const { signer, payload } = args;
    const signerType = signer.type();

    // Sign Bytes:
    // For Bitcoin, sign as hex with no prefix; for other types, convert it to raw bytes
    if (payload instanceof Uint8Array) {
        const signature = await signer.sign(signerType === 'bitcoin' ? u8aToHex(payload).substring(2) : payload);

        return api.createType('HeimaMultiSignature', {
            [signerType]: signature,
        });
    }

    // Sign hex:
    // Remove the prefix for bitcoin signature, and use raw bytes for other types
    if (payload.startsWith('0x')) {
        const signature = await signer.sign(signerType === 'bitcoin' ? payload.substring(2) : hexToU8a(payload));

        return api.createType('HeimaMultiSignature', {
            [signerType]: signature,
        });
    }

    // Sign string:
    // For Bitcoin, pass it as it is, for other types, convert it to raw bytes
    const signature = await signer.sign(signerType === 'bitcoin' ? payload : stringToU8a(payload));

    return api.createType('HeimaMultiSignature', {
        [signerType]: signature,
    });
}

export function createNativeTask(api: ApiPromise, call: [string, string], params: unknown): NativeTask {
    const [variant, argType] = call;
    return api.createType('NativeTask', {
        [variant]: api.createType(argType, params),
    });
}

// We only support web3 authentication in these tests
export async function createNativeTaskWrapper(
    api: ApiPromise,
    task: NativeTask,
    signer: Signer,
    nonce: Codec,
    mrenclave: string,
    withWrappedBytes = false,
    withPrefix = false
): Promise<NativeTaskWrapper> {
    let payload: string = blake2AsHex(u8aConcat(task.toU8a(), nonce.toU8a(), hexToU8a(mrenclave)), 256);

    if (withWrappedBytes) {
        payload = `<Bytes>${payload}</Bytes>`;
    }

    if (withPrefix) {
        const prefix = 'Token: ';
        const msg = prefix + payload;
        payload = msg;
        console.log('Signing message: ', payload);
    }

    const signature = await createHeimaMultiSignature(api, {
        signer,
        payload,
    });

    const auth: OmniAuth = api.createType('OmniAuth', {
        Web3: api.createType('(HeimaMultiSignature)', signature),
    });


    let n = api.createType('Option<Nonce>', nonce);
    let a = api.createType('Option<OmniAuth>', auth);

    return api.createType('NativeTaskWrapper', {
        task: task,
        nonce: n,
        auth: a,
    });
}

export function createRawRequestPlain(
    api: ApiPromise,
    nativeTaskWrapper: NativeTaskWrapper
): RawRequest {
    return api.createType('RawRequest', {
        ['Plain']: NativeTaskWrapper,
    });
}

type OmniAccountPermissionString =
    | 'All'
    | 'AccountManagement'
    | 'RequestNativeIntent'
    | 'RequestEthereumIntent'
    | 'RequestSolanaIntent';

export function createOmniAccountPermission(
    api: ApiPromise,
    permission: OmniAccountPermissionString
): OmniAccountPermission {
    return api.createType('OmniAccountPermission', permission);
}
