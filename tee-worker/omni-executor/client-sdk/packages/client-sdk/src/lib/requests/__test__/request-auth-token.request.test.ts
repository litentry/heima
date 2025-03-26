import { ApiPromise, Keyring } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { u8aToHex } from '@polkadot/util';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import { getChain } from '@heima/chaindata';
import { identity, omniExecutor, sidechain } from '@heima/parachain-api';

import { createAccountStore } from '@requests/create-account-store.request';
import { requestAuthToken } from '@requests/request-auth-token.request';
import { createIdentityType } from '@type-creators/identity';
import { toHash } from '@utils/identity';

import { getAndWaitForAccountStoreCreation } from '@test-utils/helpers';

const types = {
    ...identity.types, // Identity is defined here
    ...omniExecutor.types, // NativeCall is defined here
    ...sidechain.types, // AesOutput is defined here
};

describe('request-auth-token', () => {
    let api: ApiPromise;

    beforeAll(async () => {
        api = new ApiPromise({
            provider: new WsProvider(getChain('heima-local').rpcs[0].url),
            types,
        });

        await api.isReady;
        await cryptoWaitReady();
    });

    it('web3', async () => {
        const keyring = new Keyring({ type: 'sr25519' });
        const memberSigner = keyring.addFromUri('//Dave');
        const member = createIdentityType(api.registry, {
            addressOrHandle: memberSigner.address,
            type: 'Substrate',
        });

        await (async () => {
            const { send, payloadToSign = '' } = await createAccountStore(api, { member });

            const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

            await send({
                authentication: {
                    type: 'Web3',
                    signer: member,
                    signature: signatureHex,
                },
            });
        })();

        await getAndWaitForAccountStoreCreation(api, toHash(member));

        // Wait 1 second for the omni_account can be retrieved from the omni_account_storage in omni-executor.
        await new Promise((resolve) => setTimeout(resolve, 1000));

        const { send, payloadToSign = '' } = await requestAuthToken(api, {
            member,
        });

        const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

        const result = await send({
            authentication: {
                type: 'Web3',
                signer: member,
                signature: signatureHex,
            },
        });

        expect(result.token.length).toBeGreaterThan(0);
    });
});
