import { step } from 'mocha-steps';
import { assert } from 'chai';
import { Identity } from '@heima-network/api-augment/omni';
import { createIntegrationTestContext, IntegrationTestContext } from './utils/context';
import { SubstrateSigner } from './utils/signer';
import { getOmniAccount } from './utils/omni_account';
import { createNativeTask, createNativeTaskWrapper } from './utils/type_creators';
import { getWeb3SignInMessage, sendRawTaskPlain } from './utils/requests';
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

    // AccountStore tests removed after pallet refactoring
    
    step('test request auth token', async function () {
        let msgToSign = await getWeb3SignInMessage(context, 'heima', omniAccount);
        console.log('msgCode:', msgToSign);
        const nativeTask = createNativeTask(context.api, ['RequestAuthToken', 'HeimaIdentity'], aliceIdentity);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '00000',
            context.api.createType('Index', currentNonce),
            msgToSign
        );
        console.log('nativeTaskWrapper:', nativeTaskWrapper.toHuman());
        const response = await sendRawTaskPlain(context, nativeTaskWrapper);
        
        // Response should contain an auth token
        assert.exists(response, 'Response should exist');
        currentNonce++;
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

        // Create RequestIntent task with AccountId (omniAccount) instead of Identity
        const nativeTask = createNativeTask(
            context.api,
            ['RequestIntent', '(AccountId32, u32, Intent)'],
            [omniAccount, intentId, intent]
        );
        const msgToSign = await getWeb3SignInMessage(context, 'heima', omniAccount);
        const nativeTaskWrapper = await createNativeTaskWrapper(
            context.api,
            nativeTask,
            aliceWallet,
            '00001',
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
