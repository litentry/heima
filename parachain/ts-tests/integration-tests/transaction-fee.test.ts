import { u8aConcat } from '@polkadot/util';
import { expect } from 'chai';
import { step } from 'mocha-steps';

import { signAndSend, describeLitentry, loadConfig, sleep } from '../common/utils';

describeLitentry('Test Transaction Fee', ``, (context) => {
    console.log(`Test Transaction Fee`);

    // These are default proportions configured in runtime as constants
    const TREASURY_PROPORTION = 40;
    const AUTHOR_PROPORTION = 0;
    const BURNED_PROPORTION = 60;

    step('Verify transaction fee distribution', async function () {
        // Get Treasury account
        const treasuryAccount = u8aConcat(
            'modl',
            context.api.consts.treasury && context.api.consts.treasury.palletId
                ? context.api.consts.treasury.palletId.toU8a(true)
                : 'py/trsry',
            new Uint8Array(32)
        ).subarray(0, 32);

        // Get the initial balances of Alice (the block author), Eve and Treasury
        const { data: aliceInitBalance } = await context.api.query.system.account(context.alice.address);
        const { data: eveInitBalance } = await context.api.query.system.account(context.eve.address);
        const { data: treasuryInitBalance } = await context.api.query.system.account(treasuryAccount);

        // Send a transaction from Eve
        const transferAmount = 1000;
        const txTransfer = context.api.tx.balances.transferKeepAlive(context.bob.address, transferAmount);
        await signAndSend(txTransfer, context.eve);

        // Get the current balances of Alice (the block author), Eve and Treasury
        const { data: aliceCurrentBalance } = await context.api.query.system.account(context.alice.address);
        const { data: eveCurrentBalance } = await context.api.query.system.account(context.eve.address);
        const { data: treasuryCurrentBalance } = await context.api.query.system.account(treasuryAccount);

        // Calculate transaction fee
        const txFee = eveInitBalance.free.toNumber() - eveCurrentBalance.free.toNumber() - transferAmount;
        const treasuryBalanceIncrease = treasuryCurrentBalance.free.toNumber() - treasuryInitBalance.free.toNumber();
        const aliceBalanceIncrease = aliceCurrentBalance.free.toNumber() - aliceInitBalance.free.toNumber();

        console.log(
            `The actual transaction fee is ${txFee}, the Block author (Alice) has an balance increase of ${aliceBalanceIncrease}, and the Treasury pot has an balance increase of ${treasuryBalanceIncrease}`
        );

        const totalProportion = TREASURY_PROPORTION + AUTHOR_PROPORTION + BURNED_PROPORTION;
        expect(treasuryBalanceIncrease).to.approximately((txFee / totalProportion) * TREASURY_PROPORTION, 10);
        expect(aliceBalanceIncrease).to.approximately((txFee / totalProportion) * AUTHOR_PROPORTION, 10);
    });
});
