import { assert } from '@polkadot/util';

import type { Registry } from '@polkadot/types-codec/types';
import type { HexString } from '@polkadot/util/types';
import type { AesOutput } from '@litentry/parachain-api';

/**
 * Creates a KeyAesOutput sidechain type.
 *
 * Heads-up: ensure data.ciphertext is in hex format. Using Uint may cause a bytes out range error.
 *
 * @example
 * build from object
 * ```ts
 * const identity = createKeyAesOutputType(registry, {
 *  ciphertext: '0x...',
 *  nonce: '0x...',
 *  aad: '0x...',
 * });
 * ```
 *
 * build from hex string
 * ```ts
 * const identity = createKeyAesOutputType(registry, `0x...`);
 * ```
 */
export function createKeyAesOutputType(
  registry: Registry,
  data:
    | HexString
    | {
        // This one is important to keep as Hex, as Uint may fall out of Byte length and throw error
        ciphertext: HexString;
        nonce: HexString | Uint8Array;
        aad: HexString | Uint8Array;
      }
): AesOutput {
  if (typeof data !== 'string') {
    assert(
      typeof data.ciphertext === 'string',
      'ciphertext must be a hex string'
    );
  }
  return registry.createType('AesOutput', data);
}
