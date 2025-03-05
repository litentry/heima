import { AccountId, Index } from '@polkadot/types/interfaces';

import { ApiPromise, Identity } from '@heima/parachain-api';

import { toOmniAccount } from '@utils/identity';

/**
 * Retrieves the omni account nonce for a given identity.
 *
 * @param api Polkadot.js API instance
 * @param identity The identity to get the nonce for
 * @returns Promise resolving to the nonce value
 */
export async function getOmniAccountNonceWithIdentity(api: ApiPromise, identity: Identity): Promise<Index> {
  return await getOmniAccountNonce(api, toOmniAccount(api.registry, identity));
}

/**
 * Retrieves the nonce for a given omni account.
 *
 * @param api Polkadot.js API instance
 * @param account The omni account to get the nonce for
 * @returns Promise resolving to the nonce value
 */
export async function getOmniAccountNonce(api: ApiPromise, account: string | Uint8Array | AccountId): Promise<Index> {
  return await api.rpc.system.accountNextIndex(account);
}
