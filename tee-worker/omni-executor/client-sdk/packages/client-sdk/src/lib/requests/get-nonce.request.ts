import { AccountId, Index } from '@polkadot/types/interfaces';

import { ApiPromise, Identity } from '@heima-network/parachain-api';

import { toHash } from '@utils/identity';

/**
 * Retrieves the omni account nonce for a given identity.
 *
 * @param api Polkadot.js API instance
 * @param identity The identity to get the omni account nonce for
 * @returns Promise resolving to the nonce value
 */
export async function getOmniAccountNonceWithIdentity(api: ApiPromise, identity: Identity): Promise<Index> {
  const omniAccount = await api.query.omniAccount.memberAccountHash(toHash(identity));

  if (omniAccount.isSome) {
    return getAccountNonce(api, omniAccount.value);
  }

  // return 0 if the omni account is not found
  return api.createType<Index>('Index', 0);
}

/**
 * Retrieves the nonce for a given account.
 *
 * @param api Polkadot.js API instance
 * @param account The account to get the nonce for
 * @returns Promise resolving to the nonce value
 */
export async function getAccountNonce(api: ApiPromise, account: string | Uint8Array | AccountId): Promise<Index> {
  return api.rpc.system.accountNextIndex(account);
}
