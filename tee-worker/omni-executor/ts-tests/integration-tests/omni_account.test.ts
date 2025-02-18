import { step } from 'mocha-steps';
import { assert } from 'chai';
import { CorePrimitivesIdentity, CorePrimitivesOmniAccountMemberAccount } from 'parachain-api';
import { createIntegrationTestContext, IntegrationTestContext } from './utils/context';
import { SubstrateSigner } from './utils/signer';
import { getOmniAccount } from './utils/omni_account';
import {
    createNativeCall,
    createNativeAuthenticatedOperation,
    createOmniAccountPermission,
    createNativeQuery,
} from './utils/type_creators';
import { sendPlainRequestFromNativeCall, sendPlainRequestFromNativeQuery } from './utils/requests';
import { buildWeb3ValidationData } from './utils/identity';
import { fundAccount, sleep } from './utils/helpers';
import { encodeAddress } from '@polkadot/util-crypto';

describe('OmniAccount', function () {
    this.timeout(30000);
    let context: IntegrationTestContext;
    let aliceWallet: SubstrateSigner;
    let aliceIdentity: CorePrimitivesIdentity;
    let omniAccount: string;
    let currentNonce = 0;

    before(async function () {
        context = await createIntegrationTestContext(process.env.PARACHAIN_ENDPOINT, process.env.OMNI_WORKER_ENDPOINT);
        aliceWallet = context.web3Wallets['substrate']['Alice'] as SubstrateSigner;
        aliceIdentity = await aliceWallet.getIdentity(context.api);
        omniAccount = await getOmniAccount(context.api, aliceIdentity);
    });

    step('test create account_store', async function () {
        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isNone, 'accountStore already exists');

        const nativeCall = createNativeCall(context.api, ['create_account_store', 'LitentryIdentity'], aliceIdentity);
        const nativeCallOperation = await createNativeAuthenticatedOperation(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallOperation);

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
        await sleep(6); // wait for the worker's storage to be updated
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
        const nativeCallOperation = await createNativeAuthenticatedOperation(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallOperation);
        currentNonce++;

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
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);
        const nativeCall = createNativeCall(
            context.api,
            ['publicize_account', '(LitentryIdentity, LitentryIdentity)'],
            [aliceIdentity, bobIdentity]
        );
        const nativeCallOperation = await createNativeAuthenticatedOperation(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallOperation);
        currentNonce++;

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

    step('test set_permissions', async function () {
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);

        // current permissions
        let accountPermissions = await context.api.query.omniAccount.memberAccountPermissions(bobIdentity.hash);
        assert.equal(accountPermissions.length, 1, 'permissions length should be 1 before set permissions');
        assert.equal(
            accountPermissions[0].toString(),
            'All',
            'permission is not the expected permission before set permissions'
        );

        const newPermissions = [
            createOmniAccountPermission(context.api, 'RequestNativeIntent'),
            createOmniAccountPermission(context.api, 'RequestEthereumIntent'),
        ];
        const nativeCall = createNativeCall(
            context.api,
            ['set_permissions', '(LitentryIdentity, LitentryIdentity, Vec<OmniAccountPermission>)'],
            [aliceIdentity, bobIdentity, newPermissions]
        );
        const nativeCallOperation = await createNativeAuthenticatedOperation(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallOperation);
        currentNonce++;

        accountPermissions = await context.api.query.omniAccount.memberAccountPermissions(bobIdentity.hash);

        assert.equal(accountPermissions.length, 2, 'permissions length should be 2');
        assert.equal(
            accountPermissions[0].toString(),
            'RequestNativeIntent',
            'permission 1 is not the expected permission'
        );
        assert.equal(
            accountPermissions[1].toString(),
            'RequestEthereumIntent',
            'permission 2 is not the expected permission'
        );
    });

    step('test get_account_store', async function () {
        const nativeQuery = createNativeQuery(context.api, ['get_account_store', '(LitentryIdentity)'], aliceIdentity);
        const nativeQueryOperation = await createNativeAuthenticatedOperation(
            context.api,
            nativeQuery,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        const response = await sendPlainRequestFromNativeQuery(context, nativeQueryOperation);
        assert.isTrue(response.isOk, 'response should be ok');
        assert.isTrue(response.asOk.isQueryResponse, 'response should be query response');
        assert.isTrue(response.asOk.asQueryResponse.isAccountStore, 'response should be account store');

        const accountStore = response.asOk.asQueryResponse.asAccountStore;
        assert.equal(accountStore.length, 2, 'account store members count should be 2');

        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);

        assert.equal(
            accountStore[0].toHex(),
            aliceIdentity.toHex(),
            'account store member is not the expected identity (Alice)'
        );
        assert.equal(
            accountStore[1].toHex(),
            bobIdentity.toHex(),
            'account store member is not the expected identity (Bob)'
        );
    });

    step('test remove_account', async function () {
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
        const nativeCallOperation = await createNativeAuthenticatedOperation(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallOperation);
        currentNonce++;

        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 1, 'account store members count should be 1');
    });

    step('test request_intent (TransferNative)', async function () {
        const initialBalance = context.api.createType('u128', 50000000000000000000n);
        await fundAccount(context.api, omniAccount, initialBalance.toBigInt());
        const bobAddress = encodeAddress(context.web3Wallets['substrate']['Bob'].getAddressRaw());
        const {
            data: { free: bobInitialBalance },
        } = await context.api.query.system.account(bobAddress);
        const transferAmount = context.api.createType('u128', 10000000000000000000n);
        const intent = context.api.createType('Intent', {
            TransferNative: context.api.createType('IntentTransferNative', {
                to: bobAddress,
                value: transferAmount.toBigInt(),
            }),
        });
        const nativeCall = createNativeCall(
            context.api,
            ['request_intent', '(LitentryIdentity, Intent)'],
            [aliceIdentity, intent]
        );
        const nativeCallOperation = await createNativeAuthenticatedOperation(
            context.api,
            nativeCall,
            aliceWallet,
            context.api.createType('Index', currentNonce),
            context.mrEnclave
        );
        await sendPlainRequestFromNativeCall(context, nativeCallOperation);
        currentNonce++;

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
