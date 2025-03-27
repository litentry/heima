import { ApiPromise } from '@polkadot/api';
import { Option, Vec } from '@polkadot/types-codec';
import { AccountId32 } from '@polkadot/types/interfaces';

import { CorePrimitivesOmniAccountMemberAccount } from '@heima-network/parachain-api';

/**
 * Retrieves and waits for the account store to be created for the specified account.
 *
 * This function polls the blockchain at 1-second intervals until it confirms
 * that an account store has been created for the given account.
 *
 * @param api - The Polkadot API instance used to query the blockchain
 * @param account - The account to check for an associated account store
 * @returns A Promise that resolves to the value of the account store when it is confirmed to exist
 */
export async function getAndWaitForAccountStoreCreation(
  api: ApiPromise,
  account: string | AccountId32 | Uint8Array,
): Promise<CorePrimitivesOmniAccountMemberAccount[]> {
  let accountStore: Option<Vec<CorePrimitivesOmniAccountMemberAccount>> | undefined;

  // wait account store created
  while (true) {
    try {
      accountStore = await api.query.omniAccount.accountStore(account);
      if (accountStore && !accountStore.isEmpty) {
        // break when account store is created
        break;
      }
    } catch (_) {
      // do nothing
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  return accountStore.value.toArray();
}
