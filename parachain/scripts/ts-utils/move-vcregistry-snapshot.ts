// run: pnpm exec tsx move-vcregistry-snapshot.ts

import { initApi } from './initApis';
import colors from 'colors';
import fs from 'node:fs';
import path from 'node:path';
import prettier from 'prettier';

//set the maximal calls are 500 per batch
const BATCH_SIZE = 500;
async function encodeExtrinsic() {
    // params: source chain endpoint, destination chain endpoint
    const { sourceApi, destinationAPI } = await initApi('ws://localhost:9944', 'ws://localhost:9944');
    console.log(colors.green('get vcRegistry entries...'));

    const entries = await sourceApi.query.vcManagement.vcRegistry.entries();
    const data = entries.map((res: any) => {
        return { index: res[0].toHuman(), vc: res[1].toHuman() };
    });

    const dateSuffix = new Date().toISOString().slice(0, 10);
    const filename = `VCRegistry-${dateSuffix}.json`;
    const filepath = path.join(__dirname, filename);
    const formattedData = await prettier.format(JSON.stringify(data), {
        parser: 'json',
        printWidth: 120,
        tabWidth: 2,
        singleQuote: true,
        trailingComma: 'es5',
    });
    fs.writeFileSync(filepath, formattedData);
    console.log(colors.green(`Data saved to ${filename} successfully.`));

    let txs: any[] = [];
    console.log(colors.green('vcRegistry data length'), data.length);
    let i = 0;
    const hexData = [];

    while (data.length > 0) {
        const batch = data.splice(0, BATCH_SIZE);
        const batchTxs = batch.map((entry: any) =>
            destinationAPI.tx.vcManagement.addVcRegistryItem(
                entry.index[0],
                entry.vc.subject,
                entry.vc.assertion,
                entry.vc.hash_
            )
        );
        txs = txs.concat(batchTxs);
        if (data.length === 0 || txs.length >= BATCH_SIZE) {
            i++;
            const extrinsics = destinationAPI.tx.utility.batch(batchTxs);
            hexData.push({ batch: i, extrinsics: extrinsics.toHex() });
            // console.log(colors.green(`extrinsic ${i} encode`), extrinsics.toHex());
            txs = [];
            if (data.length === 0) {
                const extrinsicsFilename = `extrinsics-${dateSuffix}.json`;
                const extrinsicsFilepath = path.join(__dirname, extrinsicsFilename);

                const formattedHexData = await prettier.format(JSON.stringify(hexData), {
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
