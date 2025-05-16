import { ApiPromise } from '@polkadot/api';
import { Identity } from '@heima-network/api-argument/omni';
import { encodeAddress } from '@polkadot/util-crypto';
import { Index } from '@polkadot/types/interfaces';

export async function getOmniAccount(api: ApiPromise, identity: Identity): Promise<string> {
    const omniAccount = await api.rpc.state.call('OmniAccountApi_omni_account', identity.toHex());

    return encodeAddress(omniAccount.toHex());
}

export const getOmniAccountNonce = async (
    parachainApi: ApiPromise,
    memberIdentity: Identity
): Promise<Index> => {
    const omniAccount = await getOmniAccount(parachainApi, memberIdentity);
    const nonce = await parachainApi.rpc.system.accountNextIndex(omniAccount);

    return nonce;
};
