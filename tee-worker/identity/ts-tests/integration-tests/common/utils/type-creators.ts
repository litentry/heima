import { u8aToHex, hexToU8a, stringToU8a } from '@polkadot/util';
import type { ApiPromise } from '@polkadot/api';
import type { LitentryMultiSignature } from '@heima-network/api-argument/identity';
import type { Signer } from './crypto';

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
