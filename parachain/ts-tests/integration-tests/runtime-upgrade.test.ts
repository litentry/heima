import { expect, test } from 'vitest';
import * as fs from 'fs';
import { Keyring, ApiPromise, WsProvider } from '@polkadot/api';
import { describeLitentry } from '../common/utils/integration-setup';
import * as path from 'path';

/**
 * Runtime Upgrade Test using wasm-override approach
 *
 * This test validates that a new runtime can successfully launch and function by using Chopsticks'
 * wasm-override feature instead of simulating the actual upgrade process.
 *
 * See `chopsticks-runtime-upgrade-issue.md` for details on why we use this approach.
 */

async function getRuntimeVersion(api: ApiPromise) {
    const runtime_version = await api.rpc.state.getRuntimeVersion();
    return +runtime_version['specVersion'];
}

describeLitentry('Runtime upgrade test', ``, (context) => {
    test('New runtime launches and works', async () => {
        const parachainName = process.env.PARACHAIN_NAME || 'heima';
        console.log(`Testing runtime upgrade for parachain: ${parachainName}`);

        // Read expected version from WASM file
        const wasmPath = path.resolve('/tmp/runtime.wasm');
        if (!fs.existsSync(wasmPath)) {
            throw new Error(`Runtime WASM not found at ${wasmPath}`);
        }

        // Note: The WASM was already loaded via wasm-override in chopsticks config
        // We're just verifying it works correctly

        const wsProvider = new WsProvider('ws://localhost:9944');
        const api = await ApiPromise.create({ provider: wsProvider });
        await api.isReady;
        console.log('Connected to Chopsticks with new runtime ✅');

        // Get runtime version
        const runtimeVersion = await getRuntimeVersion(api);
        console.log(`Runtime version: ${runtimeVersion}`);

        // Get runtime metadata
        const metadata = await api.rpc.state.getMetadata();
        console.log(`Metadata version: ${metadata.version}`);

        // Verify we can read chain state
        const keyring = new Keyring({ type: 'sr25519' });
        const alice = keyring.addFromUri('//Alice');
        const bob = keyring.addFromUri('//Bob');

        const aliceBalance = await api.query.system.account(alice.address);
        console.log(`Alice's balance: ${aliceBalance.data.free.toString()} ✅`);

        const bobBalanceBefore = await api.query.system.account(bob.address);
        console.log(`Bob's balance before: ${bobBalanceBefore.data.free.toString()}`);

        // Test basic extrinsic: remark
        console.log('Testing basic extrinsic (remark)...');
        const remarkTx = api.tx.system.remark('Runtime upgrade test');

        await new Promise<void>((resolve, reject) => {
            remarkTx
                .signAndSend(alice, ({ status, events }) => {
                    if (status.isInBlock) {
                        console.log(`Remark included in block: ${status.asInBlock.toHex()}`);

                        // Check for success
                        const success = events.find(
                            ({ event }) => event.section === 'system' && event.method === 'ExtrinsicSuccess'
                        );

                        if (success) {
                            console.log('Remark executed successfully ✅');
                            resolve();
                        } else {
                            reject(new Error('Remark failed'));
                        }
                    }
                })
                .catch(reject);
        });

        // Test balance transfer
        console.log('Testing balance transfer...');
        const transferAmount = 1_000_000_000_000; // 1 token (12 decimals)
        const transferTx = api.tx.balances.transferKeepAlive(bob.address, transferAmount);

        await new Promise<void>((resolve, reject) => {
            transferTx
                .signAndSend(alice, ({ status, events }) => {
                    if (status.isInBlock) {
                        console.log(`Transfer included in block: ${status.asInBlock.toHex()}`);

                        const success = events.find(
                            ({ event }) => event.section === 'system' && event.method === 'ExtrinsicSuccess'
                        );

                        if (success) {
                            console.log('Transfer executed successfully ✅');
                            resolve();
                        } else {
                            reject(new Error('Transfer failed'));
                        }
                    }
                })
                .catch(reject);
        });

        // Verify balance changed
        const bobBalanceAfter = await api.query.system.account(bob.address);
        console.log(`Bob's balance after: ${bobBalanceAfter.data.free.toString()}`);

        const balanceIncreased = bobBalanceAfter.data.free.toBigInt() > bobBalanceBefore.data.free.toBigInt();
        expect(balanceIncreased).toBe(true);
        console.log('Balance increase verified ✅');

        // Test querying various pallets to ensure runtime is functional
        console.log('Testing runtime state queries...');

        const blockNumber = await api.query.system.number();
        console.log(`Current block number: ${blockNumber.toString()} ✅`);

        const blockHash = await api.query.system.blockHash(0);
        console.log(`Genesis block hash: ${blockHash.toString()} ✅`);

        const totalIssuance = await api.query.balances.totalIssuance();
        console.log(`Total issuance: ${totalIssuance.toString()} ✅`);

        await api.disconnect();

        console.log('');
        console.log('='.repeat(70));
        console.log('✅ Runtime upgrade test completed successfully!');
        console.log(`Runtime version: ${runtimeVersion}`);
        console.log('The new runtime:');
        console.log('  - Launches successfully');
        console.log('  - Can process extrinsics');
        console.log('  - Can execute transfers');
        console.log('  - Can query state');
        console.log('='.repeat(70));
    }, 60000); // 60 second timeout
});
