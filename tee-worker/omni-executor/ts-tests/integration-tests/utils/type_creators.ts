import type { Enum } from '@polkadot/types-codec';
import { HexString } from '@polkadot/util/types';
import { u8aToHex, hexToU8a, stringToU8a, u8aConcat, compactAddLength } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import { Codec } from '@polkadot/types-codec/types';
import {
    ApiPromise,
    Authentication,
    CorePrimitivesIdentity,
    LitentryMultiSignature,
    NativeCall,
    NativeCallAuthenticatedOperation,
    NativeQuery,
    NativeQueryAuthenticatedOperation,
    OmniAccountPermission,
    PlainRequest,
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

export async function createLitentryMultiSignature(
    api: ApiPromise,
    args: { signer: Signer; payload: Uint8Array | string }
): Promise<LitentryMultiSignature> {
    const { signer, payload } = args;
    const signerType = signer.type();

    // Sign Bytes:
    // For Bitcoin, sign as hex with no prefix; for other types, convert it to raw bytes
    if (payload instanceof Uint8Array) {
        const signature = await signer.sign(signerType === 'bitcoin' ? u8aToHex(payload).substring(2) : payload);

        return api.createType('LitentryMultiSignature', {
            [signerType]: signature,
        });
    }

    // Sign hex:
    // Remove the prefix for bitcoin signature, and use raw bytes for other types
    if (payload.startsWith('0x')) {
        const signature = await signer.sign(signerType === 'bitcoin' ? payload.substring(2) : hexToU8a(payload));

        return api.createType('LitentryMultiSignature', {
            [signerType]: signature,
        });
    }

    // Sign string:
    // For Bitcoin, pass it as it is, for other types, convert it to raw bytes
    const signature = await signer.sign(signerType === 'bitcoin' ? payload : stringToU8a(payload));

    return api.createType('LitentryMultiSignature', {
        [signerType]: signature,
    });
}

export function createNativeCall(api: ApiPromise, call: [string, string], params: unknown): NativeCall {
    const [variant, argType] = call;
    return api.createType('NativeCall', {
        [variant]: api.createType(argType, params),
    });
}

export function createNativeQuery(api: ApiPromise, query: [string, string], params: unknown): NativeQuery {
    const [variant, argType] = query;
    return api.createType('NativeQuery', {
        [variant]: api.createType(argType, params),
    });
}

// We only support web3 authentication in these tests
export async function createNativeAuthenticatedOperation<OP extends Enum>(
    api: ApiPromise,
    operation: OP,
    signer: Signer,
    nonce: Codec,
    mrenclave: string,
    withWrappedBytes = false,
    withPrefix = false
): Promise<OP extends NativeCall ? NativeCallAuthenticatedOperation : NativeQueryAuthenticatedOperation> {
    let payload: string = blake2AsHex(u8aConcat(operation.toU8a(), nonce.toU8a(), hexToU8a(mrenclave)), 256);

    if (withWrappedBytes) {
        payload = `<Bytes>${payload}</Bytes>`;
    }

    if (withPrefix) {
        const prefix = 'Token: ';
        const msg = prefix + payload;
        payload = msg;
        console.log('Signing message: ', payload);
    }

    const signature = await createLitentryMultiSignature(api, {
        signer,
        payload,
    });

    const authentication: Authentication = api.createType('Authentication', {
        Web3: api.createType('(LitentryMultiSignature)', signature),
    });

    if ('isGetAccountStore' in operation) {
        return api.createType('NativeQueryAuthenticatedOperation', {
            operation: operation,
            nonce,
            authentication,
        });
    }

    return api.createType('NativeCallAuthenticatedOperation', {
        operation: operation,
        nonce,
        authentication,
    });
}

export function createPlainRequest(
    api: ApiPromise,
    mrenclave: string,
    authenticated_operation: NativeCallAuthenticatedOperation | NativeQueryAuthenticatedOperation
): PlainRequest {
    return api.createType('PlainRequest', {
        mrenclave: hexToU8a(mrenclave),
        payload: compactAddLength(authenticated_operation.toU8a()),
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
