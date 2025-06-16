import { ApiPromise } from '@polkadot/api';
import { Identity } from '@heima-network/api-augment/omni';
import { encodeAddress } from '@polkadot/util-crypto';
import { Index } from '@polkadot/types/interfaces';

export async function getOmniAccount(api: ApiPromise, identity: Identity, clientId: string = 'heima'): Promise<string> {
    // Create a tuple of (client_id, identity) for the runtime API call
    const params = api.createType('(Text, Identity)', [clientId, identity]);
    const omniAccount = await api.rpc.state.call('OmniAccountApi_omni_account', params.toHex());

    return encodeAddress(omniAccount.toHex());
}

export const getOmniAccountNonce = async (
    parachainApi: ApiPromise,
    memberIdentity: Identity,
    clientId: string = 'heima'
): Promise<Index> => {
    const omniAccount = await getOmniAccount(parachainApi, memberIdentity, clientId);
    const nonce = await parachainApi.rpc.system.accountNextIndex(omniAccount);

    return nonce;
};
