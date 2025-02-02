import type { LitentryIdentity, MemberAccount } from '@litentry/parachain-api';

import type { ApiPromise } from '@polkadot/api';
import { encodeAddress } from '@polkadot/util-crypto';

/**
 * Retrieve the omni account graph for a given identity.
 * @param api - Polkadot API instance
 * @param who - Litentry identity to query
 * @returns Array of member accounts or null if not found
 * @throws Error if API calls fail
 */
export async function getOmniGraph(
  api: ApiPromise,
  { who }: { who: LitentryIdentity }
): Promise<MemberAccount[] | null> {
  try {
    const omniAccount = await api.rpc.state.call('OmniAccountApi_omni_account', who.toHex());

    const omniAccountAddress = encodeAddress(omniAccount.toHex());

    const accountStore = await api.query.omniAccount.accountStore(omniAccountAddress);

    if (accountStore.isNone) {
      return null;
    }

    return accountStore.unwrap() as MemberAccount[];
  } catch (e) {
    // swallow - empty omni graph
    console.warn('failed to get omni graph', e);
    return null;
  }
}
