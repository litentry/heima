import { AddressOrPair, ApiTypes, SubmittableExtrinsic } from '@polkadot/api/types';
import { ApiPromise } from '@polkadot/api';
import { FrameSystemEventRecord } from '@polkadot/types/lookup';

export function sleep(secs: number) {
    return new Promise((resolve) => {
        setTimeout(resolve, secs * 1000);
    });
}

export function signAndSend(tx: SubmittableExtrinsic<ApiTypes>, account: AddressOrPair) {
    return new Promise<{ block: string }>(async (resolve, reject) => {
        await tx.signAndSend(account, (result) => {
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

// After removing the sudo module, we use `EnsureRootOrHalfTechnicalCommittee` instead of `Sudo`,
// and there are only one council members in rococo-dev/heima-dev.
// So only `propose` is required, no vote.
//
// TODO: support to send the `vote extrinsic`, if the number of council members is greater than 2.
export async function sudoWrapperTC(api: ApiPromise, tx: SubmittableExtrinsic<ApiTypes>) {
    const chain = (await api.rpc.system.chain()).toString().toLowerCase();
    if (chain != 'rococo-dev') {
        const threshold = api.createType('Compact<u32>', 1);
        const call = api.createType('Call', tx);
        return api.tx.technicalCommittee.propose(threshold, call, api.createType('Compact<u32>', tx.length));
    } else {
        return api.tx.sudo.sudo(tx);
    }
}

// After removing the sudo module, we use `EnsureRootOrHalfCouncil` instead of `Sudo`,
// and there are only two council members in rococo-dev/heima-dev.
// So only `propose` is required, no vote.
//
// TODO: support to send the `vote extrinsic`, if the number of council members is greater than 2.
export async function sudoWrapperGC(api: ApiPromise, tx: SubmittableExtrinsic<ApiTypes>) {
    const chain = (await api.rpc.system.chain()).toString().toLowerCase();
    if (chain != 'rococo-dev') {
        const threshold = api.createType('Compact<u32>', 1);
        const call = api.createType('Call', tx);
        return api.tx.council.propose(threshold, call, api.createType('Compact<u32>', tx.length));
    } else {
        return api.tx.sudo.sudo(tx);
    }
}

export const subscribeToEvents = async (
    section: string,
    method: string,
    api: ApiPromise
): Promise<FrameSystemEventRecord[]> => {
    return new Promise<FrameSystemEventRecord[]>((resolve, reject) => {
        let blocksToScan = 30;
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

export const observeEvent = async (expectedSection: string, expectedMethod: string, api: ApiPromise): Promise<any> => {
    return new Promise((resolve, reject) => {
        let eventFound = false;
        let result: any;
        const query = (event: any) => true;
        const timeout = setTimeout(() => {
            if (!eventFound) {
                reject(new Error(`Event -${expectedSection}.${expectedMethod} not found within the specified time`));
            }
        }, 5 * 60 * 1000); // 5 minutes

        const unsubscribe = api.rpc.chain.subscribeNewHeads(async (header) => {
            const events = await api.query.system.events.at(header.hash);
            events.forEach(async (record, index) => {
                const { event } = record;
                if (!eventFound && event.section.includes(expectedSection) && event.method.includes(expectedMethod)) {
                    const expectedEvent = {
                        name: { section: event.section, method: event.method },
                        data: event.toHuman().data,
                        block: header.number.toNumber(),
                        event_index: index,
                    };
                    if (query(expectedEvent)) {
                        result = expectedEvent;
                        eventFound = true;
                        clearTimeout(timeout);
                        (await unsubscribe)();
                        resolve(result);
                    }
                }
            });
        });
    });
};
