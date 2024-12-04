//run: pnpm exec ts-node transfer-to-whitelist.ts

import { initApi } from './initApis';
import colors from 'colors';
let whiteList: any;
import * as ExcelJS from 'exceljs';
//100 token
const transferAmount = '100000000000000';

function transfer(api: any) {
    let txs: any = [];
    for (let index = 0; index < whiteList.length; index++) {
        try {
            let tx = api.tx.balances.transferKeepAlive(whiteList[index], transferAmount);
            txs.push(tx);
        } catch (error: any) {
            //maybe invalied address or other error remove it from whitelist
            console.log(
                colors.red('transfer Error: '),
                `${error.message}. Removing ${whiteList[index]} from whiteList.`
            );
            whiteList.splice(index, 1);
            index--;
        }
    }
    const transfer_hex = api.tx.utility.batchAll(txs).toHex();

    return transfer_hex;
}

async function main() {
    //download whitelist from google sheet (https://docs.google.com/spreadsheets/d/1QD0gVraqDDOkdJk-vhLMZEbdnbAOt_ogiJ3uHau_1Kw/edit#gid=950765040)
    //put the whitelist in the same folder as this script
    //read whitelist from excel
    const workbook = new ExcelJS.Workbook();
    await workbook.xlsx.readFile('Whitelist R1 & R2.xlsx');

    // Read sheet 'Whitelisted Addresses for R1'
    const worksheet = workbook.getWorksheet('Whitelisted Addresses for R1');

    // Read the second column of the sheet and skip the first row
    whiteList = worksheet.getColumn(2).values.slice(1);

    // params: source chain endpoint, destination chain endpoint
    const { sourceApi } = await initApi('ws://localhost:9944', 'ws://localhost:9944');

    const transfer_hex = transfer(sourceApi);
    console.log(colors.green('transfer_hex'), transfer_hex);
    console.log(colors.green('done'));
    process.exit();
}

main();
