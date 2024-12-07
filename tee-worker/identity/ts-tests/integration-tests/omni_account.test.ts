import { step } from 'mocha-steps';
import { KeyObject } from 'crypto';
import { assert } from 'chai';
import type { IntegrationTestContext, SubstrateSigner } from './common/common-types';
import {
    buildIdentityHelper,
    initIntegrationTestContext,
    buildWeb2Validation,
    Web2ValidationConfig,
} from './common/utils';
import { getTeeShieldingKey } from './common/di-utils';
import {
    createAuthenticatedTrustedCallAddAccount,
    createAuthenticatedTrustedCallCreateAccountStore,
    getOmniAccount,
    sendRequestFromTrustedCall,
    buildWeb3ValidationData,
    createAuthenticatedTrustedCallRemoveAccounts,
    createAuthenticatedTrustedCallPublicizeAccount,
    fundAccount,
    createAuthenticatedTrustedCallTransferNativeIntent,
} from './common/utils/native-request-helpers';
import { CorePrimitivesIdentity, CorePrimitivesOmniAccountMemberAccount } from 'parachain-api';
import { encodeAddress } from '@polkadot/util-crypto';

describe('Omni Account', function () {
    this.timeout(120000);
    let teeShieldingKey: KeyObject;
    let context: IntegrationTestContext;
    let aliceWallet: SubstrateSigner;
    let aliceIdentity: CorePrimitivesIdentity;
    let omniAccount: string;

    before(async function () {
        const parachainEndpoint = process.env.PARACHAIN_ENDPOINT;
        if (!parachainEndpoint) {
            throw new Error('PARACHAIN_ENDPOINT environment variable is missing.');
        }
        context = await initIntegrationTestContext(parachainEndpoint);
        teeShieldingKey = await getTeeShieldingKey(context);
        const wallet = context.web3Wallets['substrate'];
        aliceWallet = wallet['Alice'] as SubstrateSigner;
        aliceIdentity = await aliceWallet.getIdentity(context);
        omniAccount = await getOmniAccount(context.api, aliceIdentity);
    });

    step('test create_account_store', async function () {
        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isNone, 'account store already exists');

        const currentNonce = 0;
        const createAccountStoreCall = await createAuthenticatedTrustedCallCreateAccountStore(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceWallet,
            aliceIdentity
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, createAccountStoreCall);
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
        const bobIdentity = await bob.getIdentity(context);
        const validationData = await buildWeb3ValidationData(
            context,
            aliceIdentity,
            bobIdentity,
            currentNonce,
            'substrate',
            bob
        );
        const addAccountCall = await createAuthenticatedTrustedCallAddAccount(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceWallet,
            aliceIdentity,
            bobIdentity,
            validationData.toHex()
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, addAccountCall);

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

    step('test add_account web2', async function () {
        const currentNonce = 1;

        // twitter
        const twitterIdentity = await buildIdentityHelper('mock_user', 'Twitter', context);
        const validationConfig: Web2ValidationConfig = {
            identityType: 'Twitter',
            context,
            signerIdentitity: aliceIdentity,
            linkIdentity: twitterIdentity,
            verificationType: 'PublicTweet',
            validationNonce: currentNonce,
        };
        const validationData = await buildWeb2Validation(validationConfig);
        const addAccountCall = await createAuthenticatedTrustedCallAddAccount(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceWallet,
            aliceIdentity,
            twitterIdentity,
            validationData.toHex(),
            true // public account
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, addAccountCall);

        const accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 3, 'account store members count should be 3');
        const memberAccount3: CorePrimitivesOmniAccountMemberAccount = accountStore.unwrap()[2];
        assert.equal(
            memberAccount3.asPublic.asTwitter.toString(),
            twitterIdentity.asTwitter.toString(),
            'account store member 3 is not the expected member'
        );
    });

    step('test remove_account', async function () {
        const currentNonce = 2;
        const twitterIdentity = await buildIdentityHelper('mock_user', 'Twitter', context);

        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        let membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 3, 'account store members count should be 3');

        const removeAccountsCall = await createAuthenticatedTrustedCallRemoveAccounts(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceWallet,
            aliceIdentity,
            [twitterIdentity]
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, removeAccountsCall);

        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');
    });

    step('test publicize_account', async function () {
        const currentNonce = 3;
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context);
        const publicizeAccountCall = await createAuthenticatedTrustedCallPublicizeAccount(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceWallet,
            aliceIdentity,
            bobIdentity
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, publicizeAccountCall);

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

    step('test request_intent (TransferNative)', async function () {
        const initialBalance = context.api.createType('u128', 50000000000000000000n);
        await fundAccount(context.api, omniAccount, initialBalance.toBigInt());
        const bobAddress = encodeAddress(context.web3Wallets['substrate']['Bob'].getAddressRaw());
        const currentNonce = 4;
        const {
            data: { free: bobInitialBalance },
        } = await context.api.query.system.account(bobAddress);

        const transferAmount = context.api.createType('u128', 10000000000000000000n);
        const requestIntentCall = await createAuthenticatedTrustedCallTransferNativeIntent(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceWallet,
            aliceIdentity,
            bobAddress,
            transferAmount.toBigInt()
        );

        await sendRequestFromTrustedCall(context, teeShieldingKey, requestIntentCall);

        const { data: bobAccountDataAfter } = await context.api.query.system.account(bobAddress);
        assert.equal(
            bobAccountDataAfter.free.toBigInt(),
            bobInitialBalance.toBigInt() + transferAmount.toBigInt(),
            'Bob balance should be increased by 10'
        );

        const { data: omniAccountData } = await context.api.query.system.account(omniAccount);
        assert.equal(
            omniAccountData.free.toBigInt(),
            initialBalance.toBigInt() - transferAmount.toBigInt(),
            'omni account balance should be decreased by 10'
        );
    });
});
