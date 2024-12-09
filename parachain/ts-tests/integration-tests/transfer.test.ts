import { expect } from 'chai';
import { step } from 'mocha-steps';

import { signAndSend, describeLitentry } from '../common/utils';

describeLitentry('Test Balance Transfer', ``, (context) => {
    console.log(`Test Balance Transfer`);

    step('Transfer 1000 unit from Eve to Bob', async function () {
        // Get the initial balance of Eve and Bob
        const { nonce: eveInitNonce, data: eveInitBalance } = await context.api.query.system.account(
            context.eve.address
        );
        const { nonce: bobInitNonce, data: bobInitBalance } = await context.api.query.system.account(
            context.bob.address
        );

        const tx = context.api.tx.balances.transferKeepAlive(context.bob.address, 1000);
        await signAndSend(tx, context.eve);

        const { nonce: eveCurrentNonce, data: eveCurrentBalance } = await context.api.query.system.account(
            context.eve.address
        );
        const { nonce: bobCurrentNonce, data: bobCurrentBalance } = await context.api.query.system.account(
            context.bob.address
        );

        expect(eveCurrentNonce.toNumber()).to.equal(eveInitNonce.toNumber() + 1);
        expect(bobCurrentBalance.free.toBigInt()).to.equal(bobInitBalance.free.toBigInt() + BigInt(1000));
    });

    step('Transfer 1000 unit back to Eve from Bob', async function () {
        // Get the initial balance of Eve and Bob
        const { nonce: eveInitNonce, data: eveInitBalance } = await context.api.query.system.account(
            context.eve.address
        );
        const { nonce: bobInitNonce, data: bobInitBalance } = await context.api.query.system.account(
            context.bob.address
        );

        const tx = context.api.tx.balances.transferKeepAlive(context.eve.address, 1000);
        await signAndSend(tx, context.bob);

        const { nonce: eveCurrentNonce, data: eveCurrentBalance } = await context.api.query.system.account(
            context.eve.address
        );
        const { nonce: bobCurrentNonce, data: bobCurrentBalance } = await context.api.query.system.account(
            context.bob.address
        );

        expect(bobCurrentNonce.toNumber()).to.equal(bobInitNonce.toNumber() + 1);
        expect(eveCurrentBalance.free.toBigInt()).to.equal(eveInitBalance.free.toBigInt() + BigInt(1000));
    });
});
