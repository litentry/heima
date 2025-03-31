import { ApiPromise, Keyring } from '@polkadot/api';
import { hexToU8a, compactStripLength, u8aToString } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';

export function decodeRpcBytesAsString(value: HexString): string {
    return u8aToString(compactStripLength(hexToU8a(value))[1]);
}

export async function fundAccount(api: ApiPromise, account: string, amount: bigint) {
    console.log(`Funding account ${account} with ${amount}`);
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const transfer = api.tx.balances.transferAllowDeath(account, amount);

    return new Promise<void>((resolve, reject) => {
        transfer
            .signAndSend(alice, ({ isFinalized }) => {
                if (isFinalized) resolve();
            })
            .catch(reject);
    });
}

export function sleep(secs: number) {
    return new Promise((resolve) => {
        setTimeout(resolve, secs * 1000);
    });
}
