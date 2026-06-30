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
                console.log(`Transaction failed at blockHash ${result.status}`);
                reject(`Transaction is ${result.status}`);
            }
        });
    });
}

// After removing the sudo module, we use `EnsureRootOrHalfTechnicalCommittee` instead of `Sudo`,
// and there are only one council members in rococo-dev/litentry-dev.
// So only `propose` is required, no vote.
//
// TODO: support to send the `vote extrinsic`, if the number of council members is greater than 2.
export async function sudoWrapperTc(api: ApiPromise, tx: SubmittableExtrinsic<ApiTypes>) {
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

// Returns the matching `section.method` events found in a single block, or [] if none.
const matchEventsInBlock = async (
    api: ApiPromise,
    blockHash: Uint8Array | string,
    section: string,
    method: string
): Promise<FrameSystemEventRecord[]> => {
    const shiftedApi = await api.at(blockHash);
    const allBlockEvents = await shiftedApi.query.system.events();
    return allBlockEvents
        .filter(({ phase }) => phase.isApplyExtrinsic)
        .filter(({ event }) => event.section === section && event.method === method);
};

/**
 * Wait for a `section.method` event.
 *
 * The event is frequently emitted by an extrinsic that callers `await` *before* calling
 * this helper, so by the time we subscribe the event may already be in a past block. To be
 * robust against that race (and against block-time changes), we first scan a window of recent
 * blocks back from the current head, then fall back to watching new heads for a bounded
 * wall-clock duration. The budget is time-based so it does not silently shrink when block
 * time drops (e.g. 12s -> 6s).
 */
export const subscribeToEvents = async (
    section: string,
    method: string,
    api: ApiPromise
): Promise<FrameSystemEventRecord[]> => {
    const LOOKBACK_BLOCKS = 10; // recent blocks to scan for an already-emitted event
    const FORWARD_TIMEOUT_MS = 180_000; // wall-clock budget for the forward watch

    // 1) Look back: the triggering extrinsic was likely already mined before we got here.
    const head = await api.rpc.chain.getHeader();
    let cursor = head.hash;
    for (let i = 0; i < LOOKBACK_BLOCKS; i++) {
        const matching = await matchEventsInBlock(api, cursor, section, method);
        if (matching.length > 0) {
            return matching;
        }
        const header = await api.rpc.chain.getHeader(cursor);
        if (header.number.toNumber() === 0) break; // reached genesis
        cursor = header.parentHash;
    }

    // 2) Fall back to watching forward, bounded by wall-clock time rather than block count.
    return new Promise<FrameSystemEventRecord[]>((resolve, reject) => {
        let settled = false;
        const finish = async (unsub: Promise<() => void>, fn: () => void) => {
            if (settled) return;
            settled = true;
            (await unsub)();
            fn();
        };
        const unsubscribe = api.rpc.chain.subscribeNewHeads(async (blockHeader) => {
            const matching = await matchEventsInBlock(api, blockHeader.hash, section, method);
            if (matching.length > 0) {
                await finish(unsubscribe, () => resolve(matching));
            }
        });
        setTimeout(() => {
            finish(unsubscribe, () => reject(new Error(`timed out listening for event ${section}.${method}`)));
        }, FORWARD_TIMEOUT_MS);
    });
};

export const observeEvent = async (expectedSection: string, expectedMethod: string, api: ApiPromise): Promise<any> => {
    return new Promise((resolve, reject) => {
        let eventFound = false;
        let result: any;
        const query = (event: any) => true;
        const timeout = setTimeout(
            () => {
                if (!eventFound) {
                    reject(
                        new Error(`Event -${expectedSection}.${expectedMethod} not found within the specified time`)
                    );
                }
            },
            5 * 60 * 1000
        ); // 5 minutes

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
