import { expect } from 'chai';
import { step } from 'mocha-steps';
import {
    signAndSend,
    describeLitentry,
    loadConfig,
    subscribeToEvents,
    sudoWrapperGC,
    sudoWrapperTC,
} from '../common/utils';
import precompileStakingContractAbi from '../common/abi/precompile/Staking.json';
import precompileBridgeContractAbi from '../common/abi/precompile/Bridge.json';
const BN = require('bn.js');
import { evmToAddress } from '@polkadot/util-crypto';
import { KeyringPair } from '@polkadot/keyring/types';
import { HexString } from '@polkadot/util/types';
import { ethers } from 'ethers';
import { destResourceId } from '../common/utils/consts';

const toBigInt = (int: number) => BigInt(int) * BigInt(1e18);
const bn1e18 = new BN(10).pow(new BN(18));
const bn100 = new BN(100);
const bn1000 = new BN(1000);

describeLitentry('Test Parachain Precompile Contract', ``, (context) => {
    const config = loadConfig();

    const precompileStakingContractAddress = '0x000000000000000000000000000000000000502d';
    const precompileBridgeContractAddress = '0x000000000000000000000000000000000000503d';
    const evmAccountRaw = {
        privateKey: '0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391',
        address: '0xaaafB3972B05630fCceE866eC69CdADd9baC2771',
        mappedAddress: evmToAddress('0xaaafB3972B05630fCceE866eC69CdADd9baC2771', 31),
        publicKey: '0x93eac2793cb6d9e837b0f8da1a63dbc0db2ca848c05cbe66db139157922f78f9',
    };

    // candidate: collator address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
    // transform to bytes32(public key) reference:https://polkadot.subscan.io/tools/format_transform?input=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY&type=All
    const collatorPublicKey = '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';

    const provider = new ethers.providers.WebSocketProvider(config.parachain_ws);
    const wallet = new ethers.Wallet(evmAccountRaw.privateKey, provider);

    const precompileStakingContract = new ethers.Contract(
        precompileStakingContractAddress,
        precompileStakingContractAbi,
        provider
    );
    const precompileBridgeContract = new ethers.Contract(
        precompileBridgeContractAddress,
        precompileBridgeContractAbi,
        provider
    );

    const executeTransaction = async (delegateTransaction: any, contractAddress: HexString, label = '') => {
        console.log(`=== Executing ${label} ===`);
        const tx = await wallet.sendTransaction({
            to: contractAddress,
            data: delegateTransaction,
            gasLimit: 1000000,
            nonce: await wallet.getTransactionCount(),
            gasPrice: await provider.getGasPrice(),
        });
        await tx.wait();
        return tx;
    };

    const printBalance = (label: string, bl: any) => {
        console.log(
            label,
            'free',
            Number(bl.free.toString()) / 1e18,
            'reserved',
            Number(bl.reserved.toString()) / 1e18
        );
    };

    // this function makes two transactions first one from token owner substrate account to owner evm mapped account
    // and then it transfers to recepient
    const transferTokens = async (from: KeyringPair, to: any) => {
        const aliceEVMMappedAccount = from.publicKey.slice(0, 20); // pretend to be evm

        let aliceMappedSustrateAccount = evmToAddress(aliceEVMMappedAccount, 31);

        console.log('transfer from Alice to alice EMV');

        // Deposit money into substrate account's truncated EVM address's mapping substrate account
        const tx_init = context.api.tx.balances.transferKeepAlive(
            aliceMappedSustrateAccount,
            new BN('70000000000000000000')
        ); // 70
        await signAndSend(tx_init, context.alice);

        // 25000 is min_gas_price setup
        const tx = context.api.tx.evm.call(
            aliceEVMMappedAccount, // evm like
            to.address, // evm like
            '0x',
            new BN('65000000000000000000'), // 65
            1000000,
            25000000000,
            null,
            null,
            []
        );
        await signAndSend(tx, from);
    };

    const isPendingRequest = async () =>
        await precompileStakingContract.delegationRequestIsPending(evmAccountRaw.publicKey, collatorPublicKey);

    const collatorDetails = async () => {
        const response = await context.api.query.parachainStaking.autoCompoundingDelegations(collatorPublicKey);
        const collators = response.toJSON() as { address: string; value: number }[];
        return collators[0];
    };

    step('Set ExtrinsicFilter mode to Test', async function () {
        let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode('Test'));
        await signAndSend(extrinsic, context.alice);
    });

    step('Address with insufficient amount of tokens', async function () {
        const randomEvmWallet = ethers.Wallet.createRandom();
        const delegateWithAutoCompound = precompileStakingContract.interface.encodeFunctionData(
            'delegateWithAutoCompound',
            [collatorPublicKey, ethers.utils.parseUnits('60', 18), 1]
        );

        try {
            const tx = await randomEvmWallet.sendTransaction({
                to: precompileStakingContractAddress,
                data: delegateWithAutoCompound,
                gasLimit: 1000000,
                nonce: await randomEvmWallet.getTransactionCount(),
                gasPrice: await provider.getGasPrice(),
            });
            await tx.wait();

            expect(true).to.eq(false); // test should fail here
        } catch (e) {
            expect(e).to.be.instanceof(Error);
        }
    });

    step('Test precompile bridge contract', async function () {
        console.time('Test precompile bridge contract');

        const dest_address = '0xaaafb3972b05630fccee866ec69cdadd9bac2772'; // random address
        let balance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;

        // balance should be greater than 0.01
        if (parseInt(balance.free.toString()) < parseInt('10000000000000000')) {
            await transferTokens(context.alice, evmAccountRaw);

            expect(parseInt(balance.free.toString())).to.gt(parseInt('10000000000000000'));
        }

        const updateFeeTx = await sudoWrapperGC(
            context.api,
            context.api.tx.assetsHandler.setResource(destResourceId, {
                fee: new BN('1000000000000000'), //0.001
                asset: null,
            })
        );
        await signAndSend(updateFeeTx, context.alice);

        const AssetInfo = (await context.api.query.assetsHandler.resourceToAssetInfo(destResourceId)).toHuman() as any;

        const bridge_fee = AssetInfo.fee;
        expect(bridge_fee.toString().replace(/,/g, '')).to.eq(ethers.utils.parseUnits('0.001', 18).toString());
        // set chainId to whitelist
        const whitelistChainTx = await sudoWrapperGC(context.api, context.api.tx.chainBridge.whitelistChain(0));
        await signAndSend(whitelistChainTx, context.alice);

        // The above two steps are necessary, otherwise the contract transaction will be reverted.
        // transfer native token
        const transferNativeTx = precompileBridgeContract.interface.encodeFunctionData('transferAssets', [
            ethers.utils.parseUnits('0.01', 18).toString(),
            0,
            destResourceId,
            dest_address,
        ]);

        await executeTransaction(transferNativeTx, precompileBridgeContractAddress, 'transferAssets');
        const eventsPromise = subscribeToEvents('chainBridge', 'FungibleTransfer', context.api);
        const events = (await eventsPromise).map(({ event }) => event);

        expect(events.length).to.eq(1);
        const event_data = events[0].toHuman().data! as Array<string>;

        // FungibleTransfer(BridgeChainId, DepositNonce, ResourceId, u128, Vec<u8>)
        expect(event_data[0]).to.eq('0');
        expect(event_data[2]).to.eq(destResourceId);

        // 0.01 - 0.001 = 0.009
        const expectedBalance = bn1e18.div(bn100).sub(bn1e18.div(bn1000));
        expect(event_data[3].toString().replace(/,/g, '')).to.eq(expectedBalance.toString());
        expect(event_data[4]).to.eq(dest_address);

        console.timeEnd('Test precompile bridge contract');
    });

    // To see full params types for the interfaces, check notion page: https://web3builders.notion.site/Parachain-Precompile-Contract-0c34929e5f16408084446dcf3dd36006
    step('Test precompile staking contract', async function () {
        console.time('Test precompile staking contract');

        let balance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;
        printBalance('initial balance', balance);

        // top up LITs if insufficient amount for staking or they are not reserved (require: 50 LITs minimum)
        if (
            parseInt(balance.free.toString()) < parseInt('60000000000000000000') &&
            Number(balance.reserved.toString()) === 0
        ) {
            console.log('transferring more tokens');

            await transferTokens(context.alice, evmAccountRaw);

            balance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;
            printBalance('balance after transferring', balance);
        }

        const autoCompoundPercent = 20;

        let collator = (await context.api.query.parachainStaking.candidateInfo(context.alice.address)).toHuman();
        const staking_status = (
            await context.api.query.parachainStaking.delegatorState(evmAccountRaw.mappedAddress)
        ).toHuman();
        console.log('EVM Account Delegator State:', staking_status);

        if (collator === null) {
            console.log('Alice not candidate? Try joining');
            // 5001 will be big enough for both Litentry and Rococo
            let join_extrinsic = context.api.tx.parachainStaking.joinCandidates(
                ethers.utils.parseUnits('5001', 18).toString()
            );
            await signAndSend(join_extrinsic, context.alice);
        }
        collator = (await context.api.query.parachainStaking.candidateInfo(context.alice.address)).toHuman();

        // delegateWithAutoCompound(collator, amount, percent)
        const delegateWithAutoCompound = precompileStakingContract.interface.encodeFunctionData(
            'delegateWithAutoCompound',
            [collatorPublicKey, ethers.utils.parseUnits('60', 18).toString(), autoCompoundPercent]
        );

        let afterDelegateBalance = balance;
        // skip test if already delegated
        if (Number(balance.reserved.toString()) === 0) {
            await executeTransaction(
                delegateWithAutoCompound,
                precompileStakingContractAddress,
                'delegateWithAutoCompound'
            );

            afterDelegateBalance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;

            expect(afterDelegateBalance.reserved.toString()).to.eq(toBigInt(60).toString());
            const collator = await collatorDetails();
            expect(collator.value).to.eq(autoCompoundPercent);
        }

        // delegatorBondMore(collator, amount)
        const delegatorBondMore = precompileStakingContract.interface.encodeFunctionData('delegatorBondMore', [
            collatorPublicKey,
            ethers.utils.parseUnits('1', 18).toString(),
        ]);
        await executeTransaction(delegatorBondMore, precompileStakingContractAddress, 'delegatorBondMore');

        const { data: balanceAfterBondMore } = await context.api.query.system.account(evmAccountRaw.mappedAddress);

        expect(balanceAfterBondMore.reserved.toBigInt()).to.eq(afterDelegateBalance.reserved.toBigInt() + toBigInt(1));

        const setAutoCompound = precompileStakingContract.interface.encodeFunctionData('setAutoCompound', [
            collatorPublicKey,
            autoCompoundPercent + 5,
        ]);

        await executeTransaction(setAutoCompound, precompileStakingContractAddress, 'setAutoCompound');
        const collatorAfterCompound = await collatorDetails();
        expect(collatorAfterCompound.value).to.eq(autoCompoundPercent + 5);

        // scheduleDelegatorBondLess(collator, amount)
        expect(await isPendingRequest()).to.be.false;

        const scheduleDelegatorBondLess = precompileStakingContract.interface.encodeFunctionData(
            'scheduleDelegatorBondLess',
            [collatorPublicKey, ethers.utils.parseUnits('5', 18).toString()]
        );
        await executeTransaction(
            scheduleDelegatorBondLess,
            precompileStakingContractAddress,
            'scheduleDelegatorBondLess'
        );
        expect(await isPendingRequest()).to.be.true;

        // cancelDelegationRequest(collator)
        const cancelDelegationRequest = precompileStakingContract.interface.encodeFunctionData(
            'cancelDelegationRequest',
            [collatorPublicKey]
        );

        expect(await isPendingRequest()).to.be.true;
        await executeTransaction(cancelDelegationRequest, precompileStakingContractAddress, 'cancelDelegationRequest');
        expect(await isPendingRequest()).to.be.false;

        // only makes sense when parachain is compiled with `fast-runtime` feature, otherwise we'll
        // never make it within reasonable time
        if (config.parachain_fast_runtime === 'true') {
            // testing bond less + execution
            await executeTransaction(
                scheduleDelegatorBondLess,
                precompileStakingContractAddress,
                'scheduleDelegatorBondLess again to test execution'
            );
            expect(await isPendingRequest()).to.be.true;

            // executeDelegationRequest(delegator, collator);
            const executeDelegationRequest = precompileStakingContract.interface.encodeFunctionData(
                'executeDelegationRequest',
                [evmAccountRaw.publicKey, collatorPublicKey]
            );
            await executeTransaction(
                executeDelegationRequest,
                precompileStakingContractAddress,
                'executeDelegationRequest'
            );
            const { data: balanceAfterBondLess } = await context.api.query.system.account(evmAccountRaw.mappedAddress);
            expect(balanceAfterBondLess.reserved.toBigInt()).to.eq(
                balanceAfterBondMore.reserved.toBigInt() - toBigInt(5)
            );

            // testing revoke delegation + execute
            // scheduleRevokeDelegation(collator);
            const scheduleRevokeDelegation = precompileStakingContract.interface.encodeFunctionData(
                'scheduleRevokeDelegation',
                [collatorPublicKey]
            );
            await executeTransaction(
                scheduleRevokeDelegation,
                precompileStakingContractAddress,
                'scheduleRevokeDelegation'
            );

            await executeTransaction(
                executeDelegationRequest,
                precompileStakingContractAddress,
                'executeDelegationRequest'
            );
            const { data: balanceAfterRevoke } = await context.api.query.system.account(evmAccountRaw.mappedAddress);
            expect(balanceAfterRevoke.reserved.toBigInt()).to.eq(toBigInt(0));

            // delegate(collator, amount);
            const delegate = precompileStakingContract.interface.encodeFunctionData('delegate', [
                collatorPublicKey,
                ethers.utils.parseUnits('57', 18).toString(),
            ]);
            await executeTransaction(delegate, precompileStakingContractAddress, 'delegate');
            const { data: balanceAfterDelegate } = await context.api.query.system.account(evmAccountRaw.mappedAddress);
            expect(balanceAfterDelegate.reserved.toBigInt()).to.eq(toBigInt(57));
        }

        console.timeEnd('Test precompile staking contract');
    });

    step('Set ExtrinsicFilter mode to Normal', async function () {
        let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode('Normal'));
        await signAndSend(extrinsic, context.alice);
    });
});
