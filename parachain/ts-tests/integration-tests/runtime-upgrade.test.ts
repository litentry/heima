import { blake2AsHex } from '@polkadot/util-crypto';
import * as fs from 'fs';
import { Keyring, ApiPromise, WsProvider } from '@polkadot/api';
import { describeLitentry } from '../common/utils/integration-setup';
import '@polkadot/wasm-crypto/initOnlyAsm';
import * as path from 'path';
import { expect } from 'chai';
import { step } from 'mocha-steps';
import { signAndSend, subscribeToEvents } from '../common/utils';
import { KeyringPair } from '@polkadot/keyring/types';
import { Event } from '@polkadot/types/interfaces/system';
import { ApiTypes, SubmittableExtrinsic } from '@polkadot/api/types';

async function getRuntimeVersion(api: ApiPromise) {
    const runtime_version = await api.rpc.state.getRuntimeVersion();
    return +runtime_version['specVersion'];
}

async function waitForEventWithBlockProduction(
    section: string,
    method: string,
    api: ApiPromise,
    maxBlocks = 100
): Promise<Event> {
    const header = await api.rpc.chain.getHeader();
    console.log(`Current block number: ${header.number.toNumber()}`);
    for (let i = 0; i < maxBlocks; i++) {
        await api.rpc('dev_newBlock', { count: 1 });

        const events = await api.query.system.events();
        for (const record of events) {
            const { event } = record;
            if (event.section === section && event.method === method) {
                const header = await api.rpc.chain.getHeader();
                console.log(
                    `✅ Event ${section}.${method} observed after ${i + 1} blocks at: ${header.number.toNumber()}`
                );
                return event;
            }
        }
    }

    throw new Error(`❌ Timed out waiting for event ${section}.${method} after ${maxBlocks} blocks`);
}

async function waitForRuntimeUpgradeWithBlockProduction(
    api: ApiPromise,
    oldRuntimeVersion: number,
    maxBlocks = 100
): Promise<number> {
    const header = await api.rpc.chain.getHeader();
    console.log(`Current block number: ${header.number.toNumber()}`);
    for (let i = 0; i < maxBlocks; i++) {
        await api.rpc('dev_newBlock', { count: 1 });

        const runtimeVersion = await getRuntimeVersion(api);
        console.log(`⏳ Block +${i + 1}: Runtime version = ${runtimeVersion}`);

        if (runtimeVersion > oldRuntimeVersion) {
            const header = await api.rpc.chain.getHeader();
            console.log(
                `✅ Runtime upgraded to version ${runtimeVersion} after ${i + 1} blocks at: ${header.number.toNumber()}`
            );
            return runtimeVersion;
        }
    }

    throw new Error(`❌ Timeout: runtime not upgraded after ${maxBlocks} blocks`);
}

async function excuteNotePreimage(api: ApiPromise, signer: KeyringPair, encoded: string) {
    const notePreimageTx = api.tx.preimage.notePreimage(encoded);
    const eventsPromise = subscribeToEvents('preimage', 'Noted', api);
    await signAndSend(notePreimageTx, signer);
    const notePreimageEvent = (await eventsPromise).map(({ event }) => event);
    expect(notePreimageEvent.length === 1, 'Note preimage failed');
    console.log('Preimage noted ✅');
}

async function excuteCouncilProposal(
    api: ApiPromise,
    signer: KeyringPair,
    proposal: SubmittableExtrinsic<ApiTypes>
): Promise<Event[]> {
    return new Promise(async (resolve) => {
        const proposalTx = api.tx.council.propose(2, proposal, proposal.encodedLength);
        const eventsPromise = subscribeToEvents('council', 'Proposed', api);
        await signAndSend(proposalTx, signer);
        const proposalTxEvent = (await eventsPromise).map(({ event }) => event);
        expect(proposalTxEvent.length === 1, 'Council proposal failed');
        console.log('Council Proposed ✅');
        resolve(proposalTxEvent);
    });
}

async function excuteTechnicalCommitteeProposal(
    api: ApiPromise,
    signer: KeyringPair,
    encodedHash: string
): Promise<void> {
    const proposal = api.tx.democracy.fastTrack(encodedHash, 10, 1);
    const eventsPromise = subscribeToEvents('technicalCommittee', 'Executed', api);
    const techCommitteeProposalTx = api.tx.technicalCommittee.propose(1, proposal, proposal.encodedLength);
    await signAndSend(techCommitteeProposalTx, signer);
    const democracyStartedEvent = (await eventsPromise).map(({ event }) => event);
    expect(democracyStartedEvent.length === 1);
    console.log('Tech committee proposal executed ✅');
}

/// Pushes a polkadot runtime update via governance.
/// preimage => council proposal => vote => democracy pass => fast track => democracy proposal => democracy vote => enactAuthorizedUpgrade.
async function runtimeupgradeViaGovernance(api: ApiPromise, wasm: string) {
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const bob = keyring.addFromUri('//Bob');

    const old_runtime_version = await getRuntimeVersion(api);
    console.log(`Old runtime version = ${old_runtime_version}`);

    const encoded = api.tx.parachainSystem.authorizeUpgrade(blake2AsHex(wasm), false).method.toHex();
    const encodedHash = blake2AsHex(encoded);
    console.log(`Preimage hash: ${encodedHash}`);

    // Submit the preimage (if it doesn't already exist)
    let preimageStatus = (await api.query.preimage.requestStatusFor(encodedHash)).toHuman();
    if (!preimageStatus) {
        await excuteNotePreimage(api, alice, encoded);
    }
    const externalMotion = api.tx.democracy.externalProposeMajority({ Legacy: encodedHash });

    // propose the council proposal
    const proposedEvent = await excuteCouncilProposal(api, alice, externalMotion);
    const proposalHash = proposedEvent[0].data[2].toString();
    const proposalIndex = Number(proposedEvent[0].data[1].toHuman());

    // vote on the council proposal
    const voteTx = api.tx.council.vote(proposalHash, proposalIndex, true);
    const voteEventsPromise = subscribeToEvents('council', 'Voted', api);

    await Promise.all([await signAndSend(voteTx, alice), await signAndSend(voteTx, bob)]);
    const voteTxEvent = (await voteEventsPromise).map(({ event }) => event);
    expect(voteTxEvent.length === 2);
    console.log('Alice Bob council Voted ✅');

    // close the council proposal
    const councilCloseTx = api.tx.council.close(
        proposalHash,
        proposalIndex,
        {
            refTime: 1_000_000_000,
            proofSize: 1_000_000,
        },
        externalMotion.encodedLength
    );
    const closeEventsPromise = subscribeToEvents('council', 'Closed', api);
    await signAndSend(councilCloseTx, alice);
    const councilCloseEvent = (await closeEventsPromise).map(({ event }) => event);
    expect(councilCloseEvent.length === 1);
    console.log('Council Closed ✅');

    // fast track the democracy proposal
    await excuteTechnicalCommitteeProposal(api, alice, encodedHash);

    // vote on the democracy proposal
    const democracyVoteEventsPromise = subscribeToEvents('democracy', 'Voted', api);
    const referendumCount = (await api.query.democracy.referendumCount()).toNumber();
    const democracyVoteTx = api.tx.democracy.vote(referendumCount - 1, {
        Standard: { vote: true, balance: 1_00_000_000_000_000 },
    });

    await Promise.all([await signAndSend(democracyVoteTx, alice), await signAndSend(democracyVoteTx, bob)]);
    const democracyVoteEvent = (await democracyVoteEventsPromise).map(({ event }) => event);
    expect(democracyVoteEvent.length === 2);
    console.log('Alice Bob democracy Voted ✅');

    console.log('Waiting for democracy to pass...');
    await waitForEventWithBlockProduction('democracy', 'Passed', api);

    console.log('Waiting for parachainSystem upgrade authorize...');
    await waitForEventWithBlockProduction('system', 'UpgradeAuthorized', api);

    // enact the upgrade
    const parachainSystemScheduleUpgradeTx = api.tx.parachainSystem.enactAuthorizedUpgrade(wasm);
    await signAndSend(parachainSystemScheduleUpgradeTx, alice);

    console.log('Waiting for runtime upgrade to be applied...');
    await waitForEventWithBlockProduction('parachainSystem', 'ValidationFunctionApplied', api);

    const newRuntimeVersion = await waitForRuntimeUpgradeWithBlockProduction(api, old_runtime_version);
    return newRuntimeVersion;
}
describeLitentry('Runtime upgrade test', ``, (context) => {
    step('Running runtime ugprade test', async function () {
        let runtimeVersion: number;
        const wasmPath = path.resolve('/tmp/runtime.wasm');
        const wasm = fs.readFileSync(wasmPath).toString('hex');

        const wsProvider = new WsProvider('ws://localhost:9944');
        const api = await ApiPromise.create({ provider: wsProvider });
        await api.isReady;

        runtimeVersion = await runtimeupgradeViaGovernance(api, `0x${wasm}`);
        expect(runtimeVersion === (await getRuntimeVersion(api)));

        console.log('Runtime upgraded ✅');
    });
});
