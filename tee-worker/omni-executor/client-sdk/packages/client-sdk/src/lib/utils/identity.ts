import { blake2AsU8a } from '@polkadot/util-crypto';
import { Identity } from '@heima/parachain-api';

/**
 * Checks if the given identity is a Web3 identity.
 *
 * @param identity - The identity object to check
 * @returns `true` if the identity is EVM, Substrate, Bitcoin, or Solana based; `false` otherwise
 */
export function isWeb3(identity: Identity): boolean {
  return identity.isEvm || identity.isSubstrate || identity.isBitcoin || identity.isSolana;
}
