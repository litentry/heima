import { step } from 'mocha-steps';
import { KeyObject } from 'crypto';
import { assert } from 'chai';
import type { IntegrationTestContext, SubstrateSigner } from './common/common-types';
import { initIntegrationTestContext } from './common/utils';
import { getTeeShieldingKey } from './common/di-utils';
import {
    createAuthenticatedTrustedCallCreateAccountStore,
    getOmniAccount,
    sendRequestFromTrustedCall,
} from './common/utils/omni-account-helpers';
import { CorePrimitivesIdentity, CorePrimitivesOmniAccountMemberAccount } from 'parachain-api';

describe('Omni Account', function () {
    let teeShieldingKey: KeyObject;
    let context: IntegrationTestContext;
    let sender: SubstrateSigner;
    let senderIdentity: CorePrimitivesIdentity;
    let omniAccount: string;
    let currentNonce = 0;
    const getNextNonce = () => currentNonce++;

    this.timeout(60000);

    before(async function () {
        const parachainEndpoint = process.env.PARACHAIN_ENDPOINT;
        if (!parachainEndpoint) {
            throw new Error('PARACHAIN_ENDPOINT environment variable is missing.');
        }
        context = await initIntegrationTestContext(parachainEndpoint);
        teeShieldingKey = await getTeeShieldingKey(context);
        const wallet = context.web3Wallets['substrate'];
        sender = wallet['Alice'] as SubstrateSigner;
        senderIdentity = await sender.getIdentity(context);
        omniAccount = await getOmniAccount(context.api, await sender.getIdentity(context));
    });

    step('test create_account_store', async function () {
        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isNone, 'account store already exists');

        const createAccountStoreCall = await createAuthenticatedTrustedCallCreateAccountStore(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            sender,
            senderIdentity
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, createAccountStoreCall);
        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isSome, 'account store not found');

        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 1, 'account store members count should be 1');

        const memberAccount: CorePrimitivesOmniAccountMemberAccount = accountStore.unwrap()[0];
        assert.equal(
            memberAccount.asPublic.asSubstrate.toHex(),
            senderIdentity.asSubstrate.toHex(),
            'account store member is not the expected signer'
        );
    });
});
