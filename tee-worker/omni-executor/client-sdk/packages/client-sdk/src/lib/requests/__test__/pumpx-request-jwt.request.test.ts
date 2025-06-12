import { ApiPromise } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import {
    identity,
    omniExecutor,
    sidechain,
    omniAccount,
} from '@heima-network/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { requestPumpxJwt } from '@requests/pumpx-request-jwt.request';
import { requestEmailVerificationCode } from '@requests/request-email-verification-code.request';

const types = {
    ...identity.types, // Identity is defined here
    ...omniAccount.types, // OmniAccountPermission is defined here
    ...omniExecutor.types, // NativeCall is defined here
    ...sidechain.types, // AesOutput is defined here
};

describe('pumpx-request-jwt', () => {
    let api: ApiPromise;

    beforeAll(async () => {
        api = new ApiPromise({
            provider: new WsProvider(process.env.PARACHAIN_NETWORK),
            types,
        });

        await api.isReady;
        await cryptoWaitReady();
    });

    // This test is just an example. It requires receiving email verification codes, which cannot be done by running this unit test.
    describe.skip('web2 identity (email) authentication', () => {
        // Update the email address to your own email address
        const email = 'xxx@xxx.com';

        // Step 1
        it('request email verification code', async () => {
            await requestEmailVerificationCode({ email, clientId: 'pumpx-test-client' });
        });

        // Step 2
        it('request pumpx jwt', async () => {
            // Update the email verification code from the email inbox that requested in step 1
            const verificationCode = 'emailVerificationCodeYouReceivedInEmail';

            const member = createIdentityType(api.registry, {
                addressOrHandle: email,
                type: 'Email',
            });

            const { send } = await requestPumpxJwt(api, {
                member,
                // inviteCode: 'inviteCode',
                // googleCode: 'googleCode',
                // lang: 'en',
            });

            const { jwt } = await send({ authData: { type: 'Email', verificationCode } });

            expect(jwt).toBeDefined();
            expect(jwt.access_token).toBeDefined();
            console.log(JSON.stringify(jwt.toJSON(), null, 2));
        });
    });
});
