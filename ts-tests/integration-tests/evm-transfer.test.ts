import { assert, expect } from 'chai';
import { step } from 'mocha-steps';

import { signAndSend, describeLitentry, loadConfig, sudoWrapperTC } from '../common/utils';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { createPair, encodeAddress } from '@polkadot/keyring';
import { evmToAddress } from '@polkadot/util-crypto';
import { Web3 } from 'web3';

describeLitentry('Test EVM Module Transfer', ``, (context) => {
    console.log(`Test EVM Module Transfer`);
    const config = loadConfig();
    step('Transfer Value from Eve to EVM external account', async function () {
        // In case evm is not enabled in Normal Mode, switch back to filterMode, after test.
        // We do not test mode in initialization since ts-test concerns filter function too.
        const filterMode = (await context.api.query.extrinsicFilter.mode()).toHuman();
        if ('Test' !== filterMode) {
            let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode('Test'));
            let temp = await context.api.rpc.chain.getBlock();
            console.log(`setMode await Before: ${temp.block.header.number}`);
            await signAndSend(extrinsic, context.alice);
            temp = await context.api.rpc.chain.getBlock();
            console.log(`setMode await end: ${temp.block.header.number}`);
        }

        // EVM module transfer for substrate account
        const evmAccountRaw = {
            privateKey: '0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391',
            address: '0xaaafB3972B05630fCceE866eC69CdADd9baC2771',
            mappedAddress: evmToAddress('0xaaafB3972B05630fCceE866eC69CdADd9baC2771', 31),
        };
        console.log(`evm mapped substrate address: ${evmAccountRaw.mappedAddress}`);
        const { nonce: evmAccountInitNonce, data: evmAccountInitBalance } = await context.api.query.system.account(
            evmAccountRaw.mappedAddress
        );

        let eveMappedEVMAccount = context.eve.publicKey.slice(0, 20);
        let eveMappedSustrateAccount = evmToAddress(eveMappedEVMAccount, 31);
        console.log(`eve address: ${context.eve.publicKey}`);
        console.log(`eveMappedEVMAccount: ${eveMappedEVMAccount}`);

        // Deposit money into substrate account's truncated EVM address's mapping substrate account
        const tx_init = context.api.tx.balances.transfer(eveMappedSustrateAccount, 30000000000000);
        await signAndSend(tx_init, context.eve);

        // Get the initial balance of Eve and EVM external account
        const { nonce: eveInitNonce, data: eveInitBalance } = await context.api.query.system.account(
            context.eve.address
        );

        let value = 20000000000000; // 20 000 000 000 000
        // 25000 is min_gas_price setup
        const tx = context.api.tx.evm.call(
            eveMappedEVMAccount,
            evmAccountRaw.address,
            '0x',
            value,
            1000000,
            25000,
            null,
            null,
            []
        );
        let block = await context.api.rpc.chain.getBlock();
        const blockNumber = block.block.header.number;
        console.log(`evm call await before: ${block.block.header.number}`);
        await signAndSend(tx, context.eve);
        let temp = await context.api.rpc.chain.getBlock();
        console.log(`evm call await end: ${temp.block.header.number}`);

        const { nonce: eveCurrentNonce, data: eveCurrentBalance } = await context.api.query.system.account(
            context.eve.address
        );
        const { nonce: evmAccountCurrentNonce, data: evmAccountCurrentBalance } =
            await context.api.query.system.account(evmAccountRaw.mappedAddress);

        // If a substrate account using pallet_evm to trigger evm transaction,
        // it will bump 2 for nonce (one for substrate extrinsic, one for evm).
        // +1 nonce for original substrate account, plus another 1 nonce for original substrate account's truncated evm address's mapped susbtrate account.
        expect(eveCurrentNonce.toNumber()).to.equal(eveInitNonce.toNumber() + 1);
        expect(evmAccountCurrentBalance.free.toBigInt()).to.equal(
            evmAccountInitBalance.free.toBigInt() + BigInt(value)
        );

        // In case evm is not enabled in Normal Mode, switch back to filterMode, after test.
        let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode(filterMode));
        await signAndSend(extrinsic, context.alice);
    });

    step('Transfer some value back to Eve Mapped account from EVM external account', async function () {
        // In case evm is not enabled in Normal Mode, switch back to filterMode, after test.
        // We do not test mode in initialization since ts-test concerns filter function too.
        const filterMode = (await context.api.query.extrinsicFilter.mode()).toHuman();
        if ('Test' !== filterMode) {
            let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode('Test'));
            let temp = await context.api.rpc.chain.getBlock();
            console.log(`setMode await Before: ${temp.block.header.number}`);
            await signAndSend(extrinsic, context.alice);
            temp = await context.api.rpc.chain.getBlock();
            console.log(`setMode await end: ${temp.block.header.number}`);
        }

        // Get the initial balance of Eve and EVM external account
        let eveMappedEVMAccount = context.eve.publicKey.slice(0, 20);
        let eveMappedSustrateAccount = evmToAddress(eveMappedEVMAccount, 31);
        const { nonce: eveInitNonce, data: eveInitBalance } = await context.api.query.system.account(
            eveMappedSustrateAccount
        );
        const evmAccountRaw = {
            privateKey: '0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391',
            address: '0xaaafB3972B05630fCceE866eC69CdADd9baC2771',
            mappedAddress: evmToAddress('0xaaafB3972B05630fCceE866eC69CdADd9baC2771', 31),
        };
        const { nonce: evmAccountInitNonce, data: evmAccountInitBalance } = await context.api.query.system.account(
            evmAccountRaw.mappedAddress
        );
        console.log(`evmAccount Balance: ${evmAccountInitBalance}`);

        // Create Web3 instance
        const web3 = new Web3(config.parachain_ws);

        let value = 100000000000;
        // ExistentialDeposit = 100 000 000 000 (0x174876E800)
        // Sign Tx with PK
        console.log(`Tx Signing with: ${evmAccountRaw.privateKey}`);
        const transferTransaction = await web3.eth.accounts.signTransaction(
            {
                from: evmAccountRaw.address,
                to: u8aToHex(eveMappedEVMAccount),
                value: value, // must be higher than ExistentialDeposit
                gasPrice: 25000,
                gas: 1000000,
            },
            evmAccountRaw.privateKey
        );
        console.log(`Tx Signed with: ${transferTransaction.rawTransaction}`);
        const transferReceipt = await web3.eth.sendSignedTransaction(transferTransaction.rawTransaction!);
        console.log(`Tx successful with hash: ${transferReceipt.transactionHash}`);

        const { nonce: eveCurrentNonce, data: eveCurrentBalance } = await context.api.query.system.account(
            eveMappedSustrateAccount
        );
        const { nonce: evmAccountCurrentNonce, data: evmAccountCurrentBalance } =
            await context.api.query.system.account(evmAccountRaw.mappedAddress);

        console.log(`evmAccount Balance: ${evmAccountCurrentBalance}`);

        expect(evmAccountCurrentNonce.toNumber()).to.equal(evmAccountInitNonce.toNumber() + 1);
        expect(eveCurrentBalance.free.toBigInt()).to.equal(eveInitBalance.free.toBigInt() + BigInt(100000000000));

        // In case evm is not enabled in Normal Mode, switch back to filterMode, after test.
        let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode(filterMode));
        await signAndSend(extrinsic, context.alice);
    });

    step('Test substrate signature can not access ultra vires evm/substrate account', async function () {
        // In case evm is not enabled in Normal Mode, switch back to filterMode, after test.
        // We do not test mode in initialization since ts-test concerns filter function too.
        const filterMode = (await context.api.query.extrinsicFilter.mode()).toHuman();
        if ('Test' !== filterMode) {
            let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode('Test'));
            await signAndSend(extrinsic, context.alice);
        }

        // Get the initial balance of Eve and EVM external account
        const { nonce: eveInitNonce, data: eveInitBalance } = await context.api.query.system.account(
            context.eve.address
        );
        // EVM module transfer for substrate account
        const evmAccountRaw = {
            privateKey: '0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391',
            address: '0xaaafB3972B05630fCceE866eC69CdADd9baC2771',
            mappedAddress: evmToAddress('0xaaafB3972B05630fCceE866eC69CdADd9baC2771', 31),
        };
        const { nonce: evmAccountInitNonce, data: evmAccountInitBalance } = await context.api.query.system.account(
            evmAccountRaw.mappedAddress
        );

        const evmAccount = createPair(
            { toSS58: encodeAddress, type: 'ethereum' },
            { publicKey: hexToU8a(evmAccountRaw.mappedAddress), secretKey: new Uint8Array([]) }
        );

        let eveMappedEVMAccount = context.eve.publicKey.slice(0, 20);
        let value = 100000000000; // ExistentialDeposit = 100 000 000 000 (0x174876E800)
        // Sign Tx with substrate signature, try manipulate evm account out of substrate signature's control
        // 25000 is min_gas_price setup
        const tx = context.api.tx.evm.call(
            evmAccountRaw.address,
            eveMappedEVMAccount,
            '0x',
            value,
            1000000,
            25000,
            null,
            null,
            []
        );
        await signAndSend(tx, context.eve);

        const { nonce: eveCurrentNonce, data: eveCurrentBalance } = await context.api.query.system.account(
            context.eve.address
        );
        const { nonce: evmAccountCurrentNonce, data: evmAccountCurrentBalance } =
            await context.api.query.system.account(evmAccountRaw.mappedAddress);

        // Extrinsic succeed with failed origin
        // So the evm transaction nonce bump will not be triggered
        expect(eveCurrentNonce.toNumber()).to.equal(eveInitNonce.toNumber() + 1);
        // Which means balance unchanged
        expect(evmAccountCurrentBalance.free.toBigInt()).to.equal(evmAccountInitBalance.free.toBigInt());

        // In case evm is not enabled in Normal Mode, switch back to filterMode, after test.
        let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode(filterMode));
        await signAndSend(extrinsic, context.alice);
    });
});
