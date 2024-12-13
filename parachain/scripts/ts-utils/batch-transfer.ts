// run: pnpm exec ts-node batch-transfer.ts

import { initApi } from './initApis';
const fs = require('fs');
const path = require('path');
const prettier = require('prettier');
import colors from 'colors';

// maximal calls are 1000 per batch
const BATCH_SIZE = 750;
async function encodeExtrinsic() {
    // params: source chain endpoint, destination chain endpoint
    const { sourceApi, destinationAPI } = await initApi(
        'wss://rpc.litmus-parachain.litentry.io',
        'wss://rpc.litentry-parachain.litentry.io'
    );
    console.log(colors.green('Fetching system accounts entries...'));

    const entries = await sourceApi.query.system.account.entries();
    console.log(colors.green('system account entries length:'), entries.length);

    let totalIssuance = BigInt(0);
    let account_index = 0;

    const data = entries.map((res: any) => {
        const account = res[0].toHuman();
        const data = res[1].toHuman();
        const free = BigInt(data.data.free.replace(/,/g, ''));
        const reserved = BigInt(data.data.reserved.replace(/,/g, ''));
        const totalBalance = free + reserved;
        totalIssuance += totalBalance;
        account_index++;
        return {
            index: account_index,
            account: account,
            free: free.toString(),
            reserved: reserved.toString(),
            totalBalance: totalBalance.toString(),
        };
    });

    console.log('totalIssuance:', totalIssuance.toString());

    const filename = `system-accounts-entries-litmus-${new Date().toISOString().slice(0, 10)}.json`;
    const filepath = path.join(__dirname, filename);
    const formattedData = prettier.format(JSON.stringify(data), {
        parser: 'json',
        printWidth: 120,
        tabWidth: 2,
        singleQuote: true,
        trailingComma: 'es5',
    });
    fs.writeFileSync(filepath, formattedData);
    console.log(colors.green(`Data saved to ${filename} successfully.`));

    let txs: any[] = [];
    let i = 0;
    let hexData = [];
    const extrinsicsData = [];

    while (data.length > 0) {
        const batch = data.splice(0, BATCH_SIZE);
        const batchTxs = batch.map((entry: any) =>
            destinationAPI.tx.balances.transferKeepAlive(entry.account[0], entry.totalBalance)
        );
        txs = txs.concat(batchTxs);
        if (data.length === 0 || txs.length >= BATCH_SIZE) {
            i++;
            const extrinsics = destinationAPI.tx.utility.forceBatch(batchTxs);
            extrinsicsData.push({ batch: i, extrinsics: extrinsics.toHex() });

            hexData = [
                [
                    {
                        totalIssuance: totalIssuance.toString(),
                    },
                ],
                extrinsicsData,
            ];
            txs = [];
            if (data.length === 0) {
                const extrinsicsFilename = `extrinsics-${new Date().toISOString().slice(0, 10)}.json`;
                const extrinsicsFilepath = path.join(__dirname, extrinsicsFilename);

                const formattedHexData = prettier.format(JSON.stringify(hexData), {
                    parser: 'json',
                    printWidth: 120,
                    tabWidth: 2,
                    singleQuote: true,
                    trailingComma: 'es5',
                });

                fs.writeFileSync(extrinsicsFilepath, formattedHexData);
                console.log(colors.green(`Extrinsics saved to ${extrinsicsFilename} successfully.`));
            }
        }
    }
    process.exit();
}

encodeExtrinsic();
