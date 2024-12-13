import { createERCDepositData, describeCrossChainTransfer } from '../common/setup/setup-bridge';
import { step } from 'mocha-steps';
import { numberToHex } from 'web3-utils';
import { signAndSend, sleep } from '../common/utils';
import { assert } from 'chai';
import { BigNumber, ethers } from 'ethers';
import { BN } from 'bn.js';
import { destResourceId } from '../common/utils/consts';

const bn100e18 = new BN(10).pow(new BN(18)).mul(new BN(100));

describeCrossChainTransfer('Test Cross-chain Transfer', ``, (context) => {
    const ferdieSubstratePubkey = '0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c'; // from keyring `//Ferdie`

    step('Transfer 100 LIT from eth to parachain', async function () {
        let bridge = context.ethConfig.bridge.connect(context.ethConfig.wallets.bob);
        let erc20 = context.ethConfig.erc20.connect(context.ethConfig.wallets.bob);

        const depositAmount = numberToHex('100,000,000,000,000,000,000'.replace(/,/g, ''));
        let destinationChainID = parseInt(context.parachainConfig.api.consts.chainBridge.bridgeChainId.toString());

        const beforeAccountData = await context.parachainConfig.api.query.system.account(
            context.parachainConfig.ferdie.address
        );

        // approve
        await erc20.approve(context.ethConfig.erc20Handler.address, depositAmount);
        await sleep(6);

        // deposit
        let data = createERCDepositData(depositAmount, 32, ferdieSubstratePubkey);
        await bridge.deposit(destinationChainID, destResourceId, data);
        await sleep(12 * 4);

        const afterAccountData = await context.parachainConfig.api.query.system.account(
            context.parachainConfig.ferdie.address
        );

        assert.equal(
            bn100e18.add(new BN(beforeAccountData.data.free.toString())).toString(),
            new BN(afterAccountData.data.free.toString()).toString()
        );
    });

    step('Transfer 100 LIT from parachain to eth', async function () {
        let erc20 = context.ethConfig.erc20.connect(context.ethConfig.wallets.bob);
        const b: BigNumber = await erc20.balanceOf(context.ethConfig.wallets.charlie.address);

        await signAndSend(
            context.parachainConfig.api.tx.bridgeTransfer.transferAssets(
                ethers.utils.parseUnits('100', 18).toString(),
                context.ethConfig.wallets.charlie.address,
                0,
                destResourceId
            ),
            context.parachainConfig.bob
        );
        await sleep(15);
        const actual_receive = BigNumber.from('99,000,000,000,000,000,000'.replace(/,/g, ''));
        assert.equal(
            b.add(actual_receive).toString(),
            (await erc20.balanceOf(context.ethConfig.wallets.charlie.address)).toString()
        );
    });

    step('Boundary testing on ethereum: over the maximum balance', async function () {
        const beforeHandlerBalance: BigNumber = await context.ethConfig.erc20.balanceOf(
            context.ethConfig.erc20Handler.address
        );

        const AssetInfo = (
            await context.parachainConfig.api.query.assetsHandler.resourceToAssetInfo(destResourceId)
        ).toHuman() as any;
        const fee = AssetInfo.fee;

        const bridgeInterface = new ethers.utils.Interface(require('../common/abi/bridge/Bridge.json').abi);

        await signAndSend(
            context.parachainConfig.api.tx.bridgeTransfer.transferAssets(
                beforeHandlerBalance
                    .add(BigNumber.from(100))
                    .add(BigNumber.from(fee.replace(/,/g, '')))
                    .toString(),
                context.ethConfig.wallets.charlie.address,
                0,
                destResourceId
            ),
            context.parachainConfig.bob
        );

        const provider = context.ethConfig.wallets.bob.provider;
        const currentBlock = await provider.getBlockNumber();
        await sleep(15);
        for (let i = currentBlock; i <= (await provider.getBlockNumber()); i++) {
            const block = await provider.getBlockWithTransactions(i);
            for (let j = 0; j < block.transactions.length; j++) {
                if (block.transactions[j].to === context.ethConfig.bridge.address) {
                    const tx = block.transactions[j];
                    const decodedInput = bridgeInterface.parseTransaction({ data: tx.data, value: tx.value });

                    // The last vote proposal of threshold should failed
                    if (decodedInput.name === 'voteProposal') {
                        console.log('Found a voteProposal event');
                        break;
                    }
                }
            }
        }
        const afterHandlerBalance = await context.ethConfig.erc20.balanceOf(context.ethConfig.erc20Handler.address);
        assert.equal(
            afterHandlerBalance.toString(),
            beforeHandlerBalance.toString(),
            'afterHandlerBalance is not equal to beforeHandlerBalance'
        );
    });

    step('Boundary testing on ethereum: equal to the maximum balance', async function () {
        const beforeHandlerBalance: BigNumber = await context.ethConfig.erc20.balanceOf(
            context.ethConfig.erc20Handler.address
        );

        const beforeReceiptBalance: BigNumber = await context.ethConfig.erc20.balanceOf(
            context.ethConfig.wallets.charlie.address
        );

        const erc20 = context.ethConfig.erc20.connect(context.ethConfig.wallets.bob);
        const AssetInfo = (
            await context.parachainConfig.api.query.assetsHandler.resourceToAssetInfo(destResourceId)
        ).toHuman() as any;
        const fee = AssetInfo.fee;

        await signAndSend(
            context.parachainConfig.api.tx.bridgeTransfer.transferAssets(
                beforeHandlerBalance.add(BigNumber.from(fee.replace(/,/g, ''))).toString(),
                context.ethConfig.wallets.charlie.address,
                0,
                destResourceId
            ),
            context.parachainConfig.bob
        );
        await sleep(15);
        const afterReceiptBalance: BigNumber = await erc20.balanceOf(context.ethConfig.wallets.charlie.address);
        assert.equal(
            (await erc20.balanceOf(context.ethConfig.erc20Handler.address)).toString(),
            '0',
            'handler balance is not 0'
        );
        assert.equal(
            afterReceiptBalance.toString(),
            beforeReceiptBalance.add(BigNumber.from(beforeHandlerBalance)).toString(),
            'afterReceiptBalance is not correct'
        );
    });

    step('Boundary testing on parachain', async function () {
        let bridge = context.ethConfig.bridge.connect(context.ethConfig.wallets.bob);

        let erc20 = context.ethConfig.erc20.connect(context.ethConfig.wallets.bob);
        const maximum_issuance = new BN(
            (await context.parachainConfig.api.query.assetsHandler.maximumIssuance()).toString()
        );
        await context.ethConfig.erc20.mint(
            context.ethConfig.wallets.bob.address,
            maximum_issuance.sub(new BN(1000)).toString()
        );
        const depositAmount = numberToHex('100,000,000,000,000,000,000'.replace(/,/g, ''));
        let destinationChainID = parseInt(context.parachainConfig.api.consts.chainBridge.bridgeChainId.toString());

        await erc20.approve(context.ethConfig.erc20Handler.address, depositAmount);
        await sleep(2);
        let data = createERCDepositData(depositAmount, 32, ferdieSubstratePubkey);

        await bridge.deposit(destinationChainID, destResourceId, data);
        let expectResult = false;
        const block = await context.parachainConfig.api.rpc.chain.getBlock();
        const blockNumber = block.block.header.number;
        const unsubscribe = await context.parachainConfig.api.rpc.chain.subscribeNewHeads(async (header) => {
            console.log(`Chain is at block: #${header.number}`);
            const signedBlock = await context.parachainConfig.api.rpc.chain.getBlock(header.hash);
            const apiAt = await context.parachainConfig.api.at(signedBlock.block.header.hash);
            const allRecords = await apiAt.query.system.events();
            if (header.number.toNumber() > blockNumber.toNumber() + 4) {
                unsubscribe();
                assert.fail('expect the transaction fail in the last 4 blocks, but not found');
            }
            signedBlock.block.extrinsics.forEach((ex, index) => {
                if (!(ex.method.section === 'chainBridge' && ex.method.method === 'acknowledgeProposal')) {
                    return;
                }
                allRecords
                    .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
                    .forEach(({ event }) => {
                        if (context.parachainConfig.api.events.system.ExtrinsicFailed.is(event)) {
                            const [dispatchError, dispatchInfo] = event.data;
                            let errorInfo;
                            // decode the error
                            if (dispatchError.isModule) {
                                // for module errors, we have the section indexed, lookup
                                // (For specific known errors, we can also do a check against the
                                // context.parachainConfig.api.errors.<module>.<ErrorName>.is(dispatchError.asModule) guard)
                                const decoded = context.parachainConfig.api.registry.findMetaError(
                                    dispatchError.asModule
                                );
                                errorInfo = `${decoded.section}.${decoded.name}`;
                            } else {
                                // Other, CannotLookup, BadOrigin, no extra info
                                errorInfo = dispatchError.toString();
                            }
                            expectResult = true;
                            console.log(`chainBridge.acknowledgeProposal:: ExtrinsicFailed:: ${errorInfo}`);
                            return;
                        }
                    });
            });
            if (expectResult) {
                unsubscribe();
                assert.exists('');
            }
        });
    });
});
