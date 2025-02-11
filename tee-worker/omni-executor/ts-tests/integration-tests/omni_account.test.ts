import { step } from 'mocha-steps';
import { assert } from 'chai';
import { CorePrimitivesIdentity, CorePrimitivesOmniAccountMemberAccount } from 'parachain-api';
import { createIntegrationTestContext, IntegrationTestContext } from './utils/context';
import { SubstrateSigner } from './utils/signer';
import { getOmniAccount } from './utils/omni_account';
import { createNativeCall, createNativeCallAuthenticated } from './utils/type_creators';
import { sendPlainRequestFromNativeCall } from './utils/requests';

describe('OmniAccount', function () {
    this.timeout(120000);
    let context: IntegrationTestContext;
    let aliceWallet: SubstrateSigner;
    let aliceIdentity: CorePrimitivesIdentity;
    let omniAccount: string;

    before(async function () {
        context = await createIntegrationTestContext(process.env.PARACHAIN_ENDPOINT, process.env.OMNI_WORKER_ENDPOINT);
        aliceWallet = context.web3Wallets['substrate']['Alice'] as SubstrateSigner;
        aliceIdentity = await aliceWallet.getIdentity(context.api);
        omniAccount = await getOmniAccount(context.api, aliceIdentity);
    });

    step('test create account_store', async function () {
        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isNone, 'accountStore already exists');

        const currentNonce = 2;
        const nativeCall = createNativeCall(context.api, ['create_account_store', 'LitentryIdentity'], aliceIdentity);
        const nativeCallAuthenticated = await createNativeCallAuthenticated(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallAuthenticated);

        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isSome, 'account store not found');

        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 1, 'account store members count should be 1');

        const memberAccount: CorePrimitivesOmniAccountMemberAccount = accountStore.unwrap()[0];
        assert.equal(
            memberAccount.asPublic.asSubstrate.toHex(),
            aliceIdentity.asSubstrate.toHex(),
            'account store member is not the expected signer'
        );
    });
});
