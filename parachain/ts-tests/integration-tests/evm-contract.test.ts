import { assert, expect } from 'chai';
import { step } from 'mocha-steps';
import { signAndSend, describeLitentry, loadConfig, sudoWrapperTC } from '../common/utils';
import { compiled } from '../common/utils/compile';
import { evmToAddress } from '@polkadot/util-crypto';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { Web3 } from 'web3';
const BN = require('bn.js');
import { ethers } from 'ethers';

describeLitentry('Test EVM Module Contract', ``, (context) => {
    const config = loadConfig();
    const evmAccountRaw = {
        privateKey: '0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391',
        address: '0xaaafB3972B05630fCceE866eC69CdADd9baC2771',
        mappedAddress: evmToAddress('0xaaafB3972B05630fCceE866eC69CdADd9baC2771', 31),
    };

    step('Set ExtrinsicFilter mode to Test', async function () {
        let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode('Test'));
        await signAndSend(extrinsic, context.alice);
    });

    step('Transfer Value from Eve to EVM external account', async function () {
        let eveMappedEVMAccount = context.eve.publicKey.slice(0, 20);
        let eveMappedSustrateAccount = evmToAddress(eveMappedEVMAccount, 31);

        // Deposit money into substrate account's truncated EVM address's mapping substrate account
        const tx_init = context.api.tx.balances.transferKeepAlive(
            eveMappedSustrateAccount,
            new BN('30000000000000000000')
        );
        await signAndSend(tx_init, context.eve);

        // Get the initial balance of Eve and EVM external account
        const eveInitNonce = (await context.api.query.system.account(context.eve.address)).nonce;
        // EVM module transfer for substrate account
        const evmAccountInitBalance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;

        let value = new BN('20000000000000000000'); // 20 000 000 000 000 000 000
        // 25000 is min_gas_price setup
        const tx = context.api.tx.evm.call(
            eveMappedEVMAccount,
            evmAccountRaw.address,
            '0x',
            value,
            1000000,
            25000000000,
            null,
            null,
            []
        );
        await signAndSend(tx, context.eve);

        const eveCurrentNonce = (await context.api.query.system.account(context.eve.address)).nonce;
        const evmAccountCurrentBalance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;

        // If a substrate account using pallet_evm to trigger evm transaction,
        // it will bump 2 for nonce (one for substrate extrinsic, one for evm).
        // +1 nonce for original substrate account, plus another 1 nonce for original substrate account's truncated evm address's mapped susbtrate account.
        expect(eveCurrentNonce.toNumber()).to.equal(eveInitNonce.toNumber() + 1);
        expect(evmAccountCurrentBalance.free.toBigInt()).to.equal(
            evmAccountInitBalance.free.toBigInt() + BigInt(value)
        );
    });

    step('Transfer some value back to Eve Mapped account from EVM external account', async function () {
        // Get the initial balance of Eve and EVM external account
        let eveMappedEVMAccount = context.eve.publicKey.slice(0, 20);
        let eveMappedSustrateAccount = evmToAddress(eveMappedEVMAccount, 31);
        const eveInitBalance = (await context.api.query.system.account(eveMappedSustrateAccount)).data;
        const evmAccountInitNonce = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).nonce;

        // Create Web3 instance
        const web3 = new Web3(config.parachain_ws);

        let value = ethers.utils.parseUnits('0.1', 18).toString(); // 0.1
        // ExistentialDeposit = 100 000 000 000 000 000
        // Sign Tx with PK
        console.log(`Tx Signing with: ${evmAccountRaw.privateKey}`);
        const transferTransaction = await web3.eth.accounts.signTransaction(
            {
                from: evmAccountRaw.address,
                to: u8aToHex(eveMappedEVMAccount),
                value: value, // must be higher than ExistentialDeposit
                gasPrice: 25000000000,
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
        expect(eveCurrentBalance.free.toBigInt()).to.equal(eveInitBalance.free.toBigInt() + BigInt(value));
    });

    step('Test substrate signature can not access ultra vires evm/substrate account', async function () {
        // Get the initial balance of Eve and EVM external account
        const eveInitNonce = (await context.api.query.system.account(context.eve.address)).nonce;
        const evmAccountInitBalance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;

        let eveMappedEVMAccount = context.eve.publicKey.slice(0, 20);
        let value = new BN('100000000000000000'); // ExistentialDeposit = 100 000 000 000 000 000
        // Sign Tx with substrate signature, try manipulate evm account out of substrate signature's control
        // 25000 is min_gas_price setup
        const tx = context.api.tx.evm.call(
            evmAccountRaw.address,
            eveMappedEVMAccount,
            '0x',
            value,
            1000000,
            25000000000,
            null,
            null,
            []
        );
        await signAndSend(tx, context.eve);

        const eveCurrentNonce = (await context.api.query.system.account(context.eve.address)).nonce;
        const evmAccountCurrentBalance = (await context.api.query.system.account(evmAccountRaw.mappedAddress)).data;

        // Extrinsic succeed with failed origin
        // So the evm transaction nonce bump will not be triggered
        expect(eveCurrentNonce.toNumber()).to.equal(eveInitNonce.toNumber() + 1);
        // Which means balance unchanged
        expect(evmAccountCurrentBalance.free.toBigInt()).to.equal(evmAccountInitBalance.free.toBigInt());
    });

    step('Deploy and test contract by EVM external account', async function () {
        // Get the bytecode and API
        const bytecode = compiled.evm.bytecode.object;
        const abi = compiled.abi;
        const web3 = new Web3(config.parachain_ws);

        const deploy = async (accountFrom: any) => {
            console.log(`Attempting to deploy from account ${accountFrom.address}`);
            const hello = new web3.eth.Contract(abi);

            const helloTx = hello.deploy({
                data: bytecode,
                arguments: [],
            });

            const createTransaction = await web3.eth.accounts.signTransaction(
                {
                    data: helloTx.encodeABI(),
                    gas: await helloTx.estimateGas(),
                    gasPrice: await web3.eth.getGasPrice(),
                    nonce: await web3.eth.getTransactionCount(accountFrom.address),
                },
                accountFrom.privateKey
            );

            const createReceipt = await web3.eth.sendSignedTransaction(createTransaction.rawTransaction!);
            console.log(`Contract deployed at address: ${createReceipt.contractAddress}`);

            return createReceipt;
        };

        const deployed = await deploy(evmAccountRaw);
        if (!deployed.contractAddress) {
            console.log('deployed', deployed);
        }

        // Test get message contract method
        const sayMessage = async (contractAddress: string): Promise<string> => {
            const hello = new web3.eth.Contract(abi, contractAddress);
            console.log(`Making a call to contract at address: ${contractAddress}`);

            const data: string = await hello.methods.sayMessage().call();
            console.log(`The current message is: ${data}`);

            return data;
        };

        const message = await sayMessage(deployed.contractAddress!);
        const initialResult = message === 'Hello World' ? 1 : 0;
        assert.equal(1, initialResult, 'Contract initial storage query mismatch');

        // Test set message contract method
        const setMessage = async (contractAddress: string, accountFrom: any, message: string) => {
            console.log(`Calling the setMessage function in contract at address: ${contractAddress}`);

            const hello = new web3.eth.Contract(abi, contractAddress);
            const helloTx = hello.methods.setMessage(message);

            const createTransaction = await web3.eth.accounts.signTransaction(
                {
                    to: contractAddress,
                    data: helloTx.encodeABI(),
                    gas: await helloTx.estimateGas(),
                    nonce: await web3.eth.getTransactionCount(accountFrom.address),
                    gasPrice: await web3.eth.getGasPrice(),
                },
                accountFrom.privateKey
            );

            const createReceipt = await web3.eth.sendSignedTransaction(createTransaction.rawTransaction!);
            console.log(`Tx successful with hash: ${createReceipt.transactionHash}`);
        };
        const setMsg = await setMessage(deployed.contractAddress!, evmAccountRaw, 'Goodbye World');
        const sayMsg = await sayMessage(deployed.contractAddress!);
        const setResult = sayMsg === 'Goodbye World' ? 1 : 0;
        assert.equal(1, setResult, 'Contract modified storage query mismatch');
    });

    step('Set ExtrinsicFilter mode to Normal', async function () {
        let extrinsic = await sudoWrapperTC(context.api, context.api.tx.extrinsicFilter.setMode('Normal'));
        await signAndSend(extrinsic, context.alice);
    });
});
