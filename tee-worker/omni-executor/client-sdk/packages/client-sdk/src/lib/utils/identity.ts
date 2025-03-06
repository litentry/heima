import { blake2AsHex } from '@polkadot/util-crypto';
import { Identity } from '@heima/parachain-api';
import { HexString } from '@polkadot/util/types';

/**
 * Checks if the given identity is a Web3 identity.
 *
 * @param identity - The identity object to check
 * @returns `true` if the identity is EVM, Substrate, Bitcoin, or Solana based; `false` otherwise
 */
export function isWeb3(identity: Identity): boolean {
  return identity.isEvm || identity.isSubstrate || identity.isBitcoin || identity.isSolana;
}

/**
 * Converts the given identity to hash.
 *
 * @reference https://github.com/litentry/heima/blob/dev/common/primitives/core/src/identity.rs#L523
 *
 * @param identity - The identity object to convert
 * @returns A hash hex string of the identity
 */
export function toHash(identity: Identity): HexString {
  return blake2AsHex(identity.toU8a(), 256);
}
