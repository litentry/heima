import { hexToU8a } from '@polkadot/util';
import type { IntegrationTestContext } from './common-types';

import { AddressOrPair, FrameSystemEventRecord } from 'parachain-api';

import { Keyring } from '@polkadot/keyring';
import { ApiTypes, SubmittableExtrinsic } from '@polkadot/api/types';
import { ApiPromise } from '@polkadot/api';
// for DI-test
export const subscribeToEventsWithExtHash = async (
    requestIdentifier: string,
    context: IntegrationTestContext
): Promise<FrameSystemEventRecord[]> => {
    return new Promise<FrameSystemEventRecord[]>((resolve, reject) => {
        let blocksToScan = 30;
        /* 
        WARNING:The unsubscribe function is called inside the Promise callback, which is executed each time a new blockHeader is received. 
               `unsubscribe` is intended to unsubscribe a blockHeader if certain conditions are met. 
                If you use await, you will actually wait for this function to finish executing. 
                However, since it doesn't return a Promise, using await doesn't make sense and can lead to problematic code behaviour.
                soooooo, don't use await here
        */
        const unsubscribe = context.api.rpc.chain.subscribeNewHeads(async (blockHeader) => {
            const shiftedApi = await context.api.at(blockHeader.hash);

            const allBlockEvents = await shiftedApi.query.system.events();
            const allExtrinsicEvents = allBlockEvents.filter(({ phase }) => phase.isApplyExtrinsic);

            const matchingEvent = allExtrinsicEvents.filter((eventRecord) => {
                const eventData = eventRecord.event.data.toHuman();
                return (
                    eventData != undefined &&
                    typeof eventData === 'object' &&
                    'reqExtHash' in eventData &&
                    eventData.reqExtHash === requestIdentifier
                );
            });

            if (matchingEvent.length == 0) {
                blocksToScan -= 1;
                if (blocksToScan < 1) {
                    reject(new Error(`timed out listening for reqExtHash: ${requestIdentifier} in parachain events`));
                    (await unsubscribe)();
                }
                return;
            }

            resolve(matchingEvent);
            (await unsubscribe)();
        });
    });
};

// for II-test
export const subscribeToEvents = async (
    section: string,
    method: string,
    api: ApiPromise
): Promise<FrameSystemEventRecord[]> => {
    return new Promise<FrameSystemEventRecord[]>((resolve, reject) => {
        let blocksToScan = 15;
        const unsubscribe = api.rpc.chain.subscribeNewHeads(async (blockHeader) => {
            const shiftedApi = await api.at(blockHeader.hash);

            const allBlockEvents = await shiftedApi.query.system.events();
            const allExtrinsicEvents = allBlockEvents.filter(({ phase }) => phase.isApplyExtrinsic);

            const matchingEvent = allExtrinsicEvents.filter(({ event, phase }) => {
                return event.section === section && event.method === method;
            });

            if (matchingEvent.length == 0) {
                blocksToScan -= 1;
                if (blocksToScan < 1) {
                    reject(new Error(`timed out listening for event ${section}.${method}`));
                    (await unsubscribe)();
                }
                return;
            }

            resolve(matchingEvent);
            (await unsubscribe)();
        });
    });
};

export async function waitForBlock(api: ApiPromise, blockNumber: number, blocksToCheck = 5) {
    let count = 0;

    return new Promise<void>((resolve, reject) => {
        const unsubscribe = api.rpc.chain.subscribeNewHeads(async (header) => {
            console.log(`Chain is at block: #${header.number}`);

            if (header.number.toNumber() === blockNumber) {
                (await unsubscribe)();
                resolve();
            }

            if (++count === blocksToCheck) {
                (await unsubscribe)();
                reject(new Error(`Timeout: Block #${blockNumber} not reached within ${blocksToCheck} blocks.`));
            }
        });
    });
}

export async function setAliceAsAdmin(api: ApiPromise) {
    // Get keyring of Alice, who is also the sudo in dev chain spec
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    const tx = await sudoWrapperGc(api, api.tx.teebag.setAdmin('esqZdrqhgH8zy1wqYh1aLKoRyoRWLFbX9M62eKfaTAoK67pJ5'));

    console.log(`Setting Alice as Admin for Teebag`);
    return signAndSend(tx, alice);
}

export function signAndSend(tx: SubmittableExtrinsic<ApiTypes>, account: AddressOrPair) {
    return new Promise<{ block: string }>((resolve, reject) => {
        tx.signAndSend(account, (result) => {
            console.log(`Current status is ${result.status}`);
            if (result.status.isInBlock) {
                console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
            } else if (result.status.isFinalized) {
                console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
                resolve({
                    block: result.status.asFinalized.toString(),
                });
            } else if (result.status.isInvalid) {
                reject(`Transaction is ${result.status}`);
            }
        });
    });
}
// After removing the sudo module, we use `EnsureRootOrHalfCouncil` instead of `Sudo`,
// and there are only two council members in rococo-dev/litentry-dev.
// So only `propose` is required, no vote.
//
// TODO: support to send the `vote extrinsic`, if the number of council members is greater than 2.
export async function sudoWrapperGc(api: ApiPromise, tx: SubmittableExtrinsic<ApiTypes>) {
    const chain = (await api.rpc.system.chain()).toString().toLowerCase();
    if (chain != 'rococo-dev') {
        const threshold = api.createType('Compact<u32>', 1);
        const call = api.createType('Call', tx);
        return api.tx.council.propose(threshold, call, api.createType('Compact<u32>', tx.length));
    } else {
        return api.tx.sudo.sudo(tx);
    }
}

export async function setScheduledEnclave(api: ApiPromise, block: number, mrenclave: string) {
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    const tx = api.tx.teebag.setScheduledEnclave('Identity', block, hexToU8a(`0x${mrenclave}`));

    console.log('Schedule Enclave Extrinsic sent');
    return signAndSend(tx, alice);
}
