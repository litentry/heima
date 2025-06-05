import { step } from 'mocha-steps';
import { assert } from 'chai';
import { Identity, MemberAccount } from '@heima-network/api-argument/omni';
import { createIntegrationTestContext, IntegrationTestContext } from './utils/context';
import { SubstrateSigner } from './utils/signer';
import { getOmniAccount } from './utils/omni_account';
import { createNativeTask, createNativeTaskWrapper, createOmniAccountPermission } from './utils/type_creators';
import { getMessageToSign, sendRawTaskPlain } from './utils/requests';
import { buildWeb3ValidationData } from './utils/identity';
import { fundAccount, sleep } from './utils/helpers';
import { encodeAddress } from '@polkadot/util-crypto';

describe('OmniAccount', function () {
    this.timeout(120000);
    let context: IntegrationTestContext;
    let aliceWallet: SubstrateSigner;
    let aliceIdentity: Identity;
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

        let msgToSign = await getMessageToSign(context, 'heima', omniAccount);
        console.log('msgCode:', msgToSign);
        const nativeTask = createNativeTask(context.api, ['CreateAccountStore', 'HeimaIdentity'], aliceIdentity);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '00000',
            context.api.createType('Index', currentNonce),
            msgToSign
        );
        console.log('nativeTaskWrapper:', nativeTaskWrapper.toHuman());
        await sendRawTaskPlain(context, nativeTaskWrapper);

        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isSome, 'account store not found');

        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 1, 'account store members count should be 1');

        const memberAccount: MemberAccount = accountStore.unwrap()[0];
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
        const nativeTask = createNativeTask(
            context.api,
            [
                'AddAccount',
                '(HeimaIdentity, HeimaIdentity, HeimaValidationData, bool, Option<Vec<OmniAccountPermission>>)',
            ],
            [
                aliceIdentity,
                bobIdentity,
                validationData.toHex(),
                false, // publicAccount
                [createOmniAccountPermission(context.api, 'All')],
            ]
        );
        const msgToSign = await getMessageToSign(context, 'heima', omniAccount);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '00001',
            context.api.createType('Index', currentNonce),
            msgToSign
        );
        await sendRawTaskPlain(context, nativeTaskWrapper);
        currentNonce++;

        const accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');
        const memberAccount1: MemberAccount = accountStore.unwrap()[0];
        assert.equal(
            memberAccount1.asPublic.asSubstrate.toHex(),
            aliceIdentity.asSubstrate.toHex(),
            'account store member 1 is not the expected member'
        );
        const memberAccount2: MemberAccount = accountStore.unwrap()[1];
        assert.isTrue(memberAccount2.isPrivate);
    });

    step('test publicize_account', async function () {
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);
        const nativeTask = createNativeTask(
            context.api,
            ['PublicizeAccount', '(HeimaIdentity, HeimaIdentity)'],
            [aliceIdentity, bobIdentity]
        );
        const msgToSign = await getMessageToSign(context, 'heima', omniAccount);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '00002',
            context.api.createType('Index', currentNonce),
            msgToSign
        );
        await sendRawTaskPlain(context, nativeTaskWrapper);
        currentNonce++;

        const accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        const membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');
        const memberAccount1: MemberAccount = accountStore.unwrap()[0];
        assert.equal(
            memberAccount1.asPublic.asSubstrate.toHex(),
            aliceIdentity.asSubstrate.toHex(),
            'account store member 1 is not the expected member'
        );
        const memberAccount2: MemberAccount = accountStore.unwrap()[1];
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
        const nativeTask = createNativeTask(
            context.api,
            ['SetPermissions', '(HeimaIdentity, HeimaIdentity, Vec<OmniAccountPermission>)'],
            [aliceIdentity, bobIdentity, newPermissions]
        );
        const msgToSign = await getMessageToSign(context, 'heima', omniAccount);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '00003',
            context.api.createType('Index', currentNonce),
            msgToSign
        );
        await sendRawTaskPlain(context, nativeTaskWrapper);
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

    step('test remove_account', async function () {
        const bob = context.web3Wallets['substrate']['Bob'] as SubstrateSigner;
        const bobIdentity = await bob.getIdentity(context.api);

        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        let membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');

        const nativeTask = createNativeTask(
            context.api,
            ['RemoveAccounts', '(HeimaIdentity, Vec<HeimaIdentity>)'],
            [aliceIdentity, [bobIdentity]]
        );
        const msgToSign = await getMessageToSign(context, 'heima', omniAccount);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '000004',
            context.api.createType('Index', currentNonce),
            msgToSign
        );
        await sendRawTaskPlain(context, nativeTaskWrapper);
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
        const intentId = context.api.createType('u32', 1n);

        const nativeTask = createNativeTask(
            context.api,
            ['RequestIntent', '(HeimaIdentity, u32, Intent)'],
            [aliceIdentity, intentId, intent]
        );
        const msgToSign = await getMessageToSign(context, 'heima', omniAccount);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '00005',
            context.api.createType('Index', currentNonce),
            msgToSign
        );
        const response = await sendRawTaskPlain(context, nativeTaskWrapper);
        console.log('response:', response.toHuman());

        currentNonce++;

        // wait for the intent to be processed and the tx to be finalized on-chain
        await sleep(36);

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
