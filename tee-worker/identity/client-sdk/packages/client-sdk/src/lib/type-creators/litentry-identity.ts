import { isHex, u8aToHex } from '@polkadot/util';
import { base58Decode, decodeAddress } from '@polkadot/util-crypto';

import type { Registry } from '@polkadot/types-codec/types';
import type { LitentryIdentity } from '@litentry/parachain-api';

/**
 * Creates a LitentryIdentity chain type.
 *
 * Notice that addresses and handles are not fully validated. This struct shouldn't be relied on for validation.
 *
 * For Bitcoin, the compressed public key is expected (begins with 02 or 03).
 *
 * For Solana, the address string is expected to be a base58-encoded or hex-encoded string.
 *
 * For Substrate, the address is expected to be a SS58-encoded or hex-encoded address.
 *
 * @example
 * ```ts
 * const substrateIdentity = createLitentryIdentityType(registry, {
 *  addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
 *  type: 'Substrate',
 * });
 *
 * const twitterIdentity = createLitentryIdentityType(registry, {
 *  addressOrHandle: 'my-twitter-handle',
 *  type: 'Twitter',
 * });
 * ```
 */
export function createLitentryIdentityType(
  registry: Registry,
  data:
    | {
        // pubKey or address or handle
        addressOrHandle: string;
        type: LitentryIdentity['type'];
      }
    | `0x${string}`
    | Uint8Array
): LitentryIdentity {
  if (data instanceof Uint8Array || typeof data === 'string') {
    return registry.createType('LitentryIdentity', data);
  }

  const { addressOrHandle, type } = data;
  if (!addressOrHandle || !type) {
    throw new Error('Missing addressOrHandle or type');
  }

  if (type === 'Substrate') {
    return registry.createType('LitentryIdentity', {
      [type]: toPublicKey(addressOrHandle), // ensures substrate acc pubKey
    }) as LitentryIdentity;
  }

  if (type === 'Bitcoin') {
    return registry.createType('LitentryIdentity', {
      [type]: formatBitcoinPublicKey(addressOrHandle),
    }) as LitentryIdentity;
  }

  if (type === 'Solana') {
    return registry.createType('LitentryIdentity', {
      [type]: isHex(addressOrHandle)
        ? addressOrHandle
        : base58Decode(addressOrHandle),
    }) as LitentryIdentity;
  }

  return registry.createType('LitentryIdentity', {
    [type]: addressOrHandle,
  }) as LitentryIdentity;
}

function formatBitcoinPublicKey(key: string): `0x${string}` {
  return isHex(key) ? key : `0x${key}`;
}

/**
 * Returns the public key from a given substrate address.
 *
 * @throws if the address is not a valid substrate address
 */
export function toPublicKey(address: string): `0x${string}` {
  return isHex(address) ? address : u8aToHex(decodeAddress(address));
}
