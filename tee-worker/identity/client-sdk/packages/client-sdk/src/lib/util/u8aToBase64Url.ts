import type { U8aLike } from '@polkadot/util/types';
import { base64Encode, base64Trim } from '@polkadot/util-crypto';

/**
 * Creates a base64-URL value. Padding is omitted.
 *
 * @see https://en.wikipedia.org/wiki/Base64#RFC_4648
 * @example
 * import { stringToU8a } from '@polkadot/util';
 * import { base64Encode } from '@polkadot/util-crypto';
 *
 * const input = stringToU8a('fo ob');
 * const base64Output = base64Encode(input);
 * const base64urlOutput = u8aToBase64Url(input);
 *
 * console.log(`base64: ${base64Output}`);
 * console.log(`base64url: ${base64urlOutput}`);
 * // base64: Zm8gYm8=
 * // base64url: Zm8gYm8
 */
export function u8aToBase64Url(value: U8aLike): string {
  return (
    // Remove padding (`=`) from base64
    base64Trim(base64Encode(value))
      // Replace `+` with `-`
      .replace(/\+/g, '-')
      // Replace `/` with `_`
      .replace(/\//g, '_')
  );
}
