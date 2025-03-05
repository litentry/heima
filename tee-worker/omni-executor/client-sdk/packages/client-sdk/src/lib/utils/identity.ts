import { AccountId } from '@polkadot/types/interfaces';
import { blake2AsU8a } from '@polkadot/util-crypto';
import { Identity } from '@heima/parachain-api';
import { Registry } from '@polkadot/types-codec/types';

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
 * Converts the given identity to the omni account.
 * 
 * @reference https://github.com/litentry/heima/blob/dev/common/primitives/core/src/identity.rs#L406
 *
 * @param identity - The identity object to convert
 * @returns An omni account of type AccountId
 */
export function toOmniAccount(registry: Registry, identity: Identity): AccountId {
  const hash = blake2AsU8a(identity.toU8a(), 256);
  return registry.createType<AccountId>('AccountId', hash);
}
