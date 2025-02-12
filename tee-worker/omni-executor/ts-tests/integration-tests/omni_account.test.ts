import { step } from 'mocha-steps';
import { assert } from 'chai';
import { CorePrimitivesIdentity, CorePrimitivesOmniAccountMemberAccount } from 'parachain-api';
import { createIntegrationTestContext, IntegrationTestContext } from './utils/context';
import { SubstrateSigner } from './utils/signer';
import { getOmniAccount } from './utils/omni_account';
import { createNativeCall, createNativeCallAuthenticated, createOmniAccountPermission } from './utils/type_creators';
import { sendPlainRequestFromNativeCall } from './utils/requests';
import { buildWeb3ValidationData } from './utils/identity';

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

        const currentNonce = 0;
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

    step('test add_account web3', async function () {
        const currentNonce = 0;
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);
        const validationData = await buildWeb3ValidationData(
            context.api,
            aliceIdentity,
            bobIdentity,
            currentNonce,
            'substrate',
            bob
        );
        const nativeCall = createNativeCall(
            context.api,
            [
                'add_account',
                '(LitentryIdentity, LitentryIdentity, LitentryValidationData, bool, Option<Vec<OmniAccountPermission>>)',
            ],
            [
                aliceIdentity,
                bobIdentity,
                validationData.toHex(),
                false, // publicAccount
                [createOmniAccountPermission(context.api, 'All')],
            ]
        );
        const nativeCallAuthenticated = await createNativeCallAuthenticated(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallAuthenticated);

        const accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');
        const memberAccount1: CorePrimitivesOmniAccountMemberAccount = accountStore.unwrap()[0];
        assert.equal(
            memberAccount1.asPublic.asSubstrate.toHex(),
            aliceIdentity.asSubstrate.toHex(),
            'account store member 1 is not the expected member'
        );
        const memberAccount2: CorePrimitivesOmniAccountMemberAccount = accountStore.unwrap()[1];
        assert.isTrue(memberAccount2.isPrivate);
    });

    step('test publicize_account', async function () {
        const currentNonce = 1;
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);
        const nativeCall = createNativeCall(
            context.api,
            ['publicize_account', '(LitentryIdentity, LitentryIdentity)'],
            [aliceIdentity, bobIdentity]
        );
        const nativeCallAuthenticated = await createNativeCallAuthenticated(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallAuthenticated);
        const accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');
        const memberAccount1: CorePrimitivesOmniAccountMemberAccount = accountStore.unwrap()[0];
        assert.equal(
            memberAccount1.asPublic.asSubstrate.toHex(),
            aliceIdentity.asSubstrate.toHex(),
            'account store member 1 is not the expected member'
        );
        const memberAccount2: CorePrimitivesOmniAccountMemberAccount = accountStore.unwrap()[1];
        assert.isTrue(memberAccount2.isPublic);
        assert.equal(
            memberAccount2.asPublic.asSubstrate.toHex(),
            bobIdentity.asSubstrate.toHex(),
            'account store member 2 is not the expected member'
        );
    });

    step('test remove_account', async function () {
        const currentNonce = 2;
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);

        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        let membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');

        const nativeCall = createNativeCall(
            context.api,
            ['remove_accounts', '(LitentryIdentity, Vec<LitentryIdentity>)'],
            [aliceIdentity, [bobIdentity]]
        );
        const nativeCallAuthenticated = await createNativeCallAuthenticated(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallAuthenticated);

        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 1, 'account store members count should be 1');
    });

});
