import { AccountId, Index } from '@polkadot/types/interfaces';
import { assert } from '@polkadot/util';

import { ApiPromise, Identity } from '@heima/parachain-api';
import { GenericAccountId } from '@polkadot/types';

/**
 * Retrieves the nonce for a given identity.
 * 
 * @param api Polkadot.js API instance
 * @param account The account to get the nonce for
 * @returns Promise resolving to the nonce value
 * @throws Error if api is not initialized or account is invalid
 */
export async function getNonce(
  api: ApiPromise,
  account: Identity | string | Uint8Array | AccountId,
): Promise<Index> {
  if (typeof account === 'string' || account instanceof Uint8Array || account instanceof GenericAccountId) {
    return await api.rpc.system.accountNextIndex(account);
  }

  const identity = account as Identity;
  assert(identity.isSubstrate, 'Identity must be a Substrate identity');

  return await api.rpc.system.accountNextIndex(identity.asSubstrate.toHex());
}
