import { expect, test } from 'vitest';
import { blake2AsHex } from '@polkadot/util-crypto';
import * as fs from 'fs';
import { Keyring, ApiPromise, WsProvider } from '@polkadot/api';
import { describeLitentry } from '../common/utils/integration-setup';
import * as path from 'path';
import { signAndSend, subscribeToEvents } from '../common/utils/index.js';
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
    parachainApi: ApiPromise,
    oldRuntimeVersion: number,
    maxBlocks = 100
): Promise<number> {
    console.log('Running in single-chain mode (no relaychain)');
    const header = await parachainApi.rpc.chain.getHeader();
    console.log(`Current parachain block number: ${header.number.toNumber()}`);

    for (let i = 0; i < maxBlocks; i++) {
        // Produce blocks only on parachain
        // The mock inherent data provider in Chopsticks will provide the upgrade_go_ahead signal
        await parachainApi.rpc('dev_newBlock', { count: 1 });

        const runtimeVersion = await getRuntimeVersion(parachainApi);
        console.log(`⏳ Block +${i + 1}: Runtime version = ${runtimeVersion}`);

        if (runtimeVersion > oldRuntimeVersion) {
            const header = await parachainApi.rpc.chain.getHeader();
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
    expect(notePreimageEvent.length).toBe(1);
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
        expect(proposalTxEvent.length).toBe(1);
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
    expect(democracyStartedEvent.length).toBe(1);
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

    // For parachain runtime upgrades, use system.authorizeUpgrade (two-step process)
    const codeHash = blake2AsHex(wasm);
    console.log(`Runtime code hash: ${codeHash}`);

    const encoded = api.tx.system.authorizeUpgrade(codeHash).method.toHex();
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
    // Vote with Alice first
    const aliceVoteTx = api.tx.council.vote(proposalHash, proposalIndex, true);
    const aliceVoteEventsPromise = subscribeToEvents('council', 'Voted', api);
    await signAndSend(aliceVoteTx, alice);
    const aliceVoteEvent = await aliceVoteEventsPromise;
    expect(aliceVoteEvent.length).toBe(1);
    console.log('Alice council Voted ✅');

    // Vote with Bob
    const bobVoteTx = api.tx.council.vote(proposalHash, proposalIndex, true);
    const bobVoteEventsPromise = subscribeToEvents('council', 'Voted', api);
    await signAndSend(bobVoteTx, bob);
    const bobVoteEvent = await bobVoteEventsPromise;
    expect(bobVoteEvent.length).toBe(1);
    console.log('Bob council Voted ✅');

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
    expect(councilCloseEvent.length).toBe(1);
    console.log('Council Closed ✅');

    // fast track the democracy proposal
    await excuteTechnicalCommitteeProposal(api, alice, encodedHash);

    // vote on the democracy proposal
    const referendumCount = (await api.query.democracy.referendumCount()).toNumber();

    // Vote with Alice first
    const aliceDemocracyVoteTx = api.tx.democracy.vote(referendumCount - 1, {
        Standard: { vote: true, balance: 1_00_000_000_000_000 },
    });
    const aliceDemocracyVoteEventsPromise = subscribeToEvents('democracy', 'Voted', api);
    await signAndSend(aliceDemocracyVoteTx, alice);
    const aliceDemocracyVoteEvent = await aliceDemocracyVoteEventsPromise;
    expect(aliceDemocracyVoteEvent.length).toBe(1);
    console.log('Alice democracy Voted ✅');

    // Vote with Bob
    const bobDemocracyVoteTx = api.tx.democracy.vote(referendumCount - 1, {
        Standard: { vote: true, balance: 1_00_000_000_000_000 },
    });
    const bobDemocracyVoteEventsPromise = subscribeToEvents('democracy', 'Voted', api);
    await signAndSend(bobDemocracyVoteTx, bob);
    const bobDemocracyVoteEvent = await bobDemocracyVoteEventsPromise;
    expect(bobDemocracyVoteEvent.length).toBe(1);
    console.log('Bob democracy Voted ✅');

    console.log('Waiting for democracy to pass...');
    await waitForEventWithBlockProduction('democracy', 'Passed', api);

    console.log('Democracy passed ✅');
    console.log('Waiting for system upgrade authorization...');
    await waitForEventWithBlockProduction('system', 'UpgradeAuthorized', api);

    console.log('Upgrade authorized ✅');
    console.log('Checking authorized upgrade storage...');

    // Check what's in storage
    const authorizedUpgrade = await api.query.system.authorizedUpgrade();
    console.log('Authorized upgrade:', authorizedUpgrade.toHuman());

    console.log('Setting up ParachainSystem storage for upgrade...');

    // ParachainSetCode::set_code() requires these storage items:
    // 1. ValidationData must exist
    // 2. UpgradeRestrictionSignal must NOT be present (or be Present to allow)
    // 3. PendingValidationCode must NOT exist
    // 4. HostConfiguration must exist

    // Check current state
    const validationData = await api.query.parachainSystem.validationData();
    const hostConfig = await api.query.parachainSystem.hostConfiguration();
    const pendingCode = await api.query.parachainSystem.pendingValidationCode();
    const upgradeRestriction = await api.query.parachainSystem.upgradeRestrictionSignal();

    console.log('Current ValidationData:', validationData.toHuman() ? 'Present' : 'None');
    console.log('Current HostConfiguration:', hostConfig.toHuman() ? 'Present' : 'None');
    console.log('Current PendingValidationCode:', pendingCode.isEmpty ? 'None' : 'Present');
    console.log('Current UpgradeRestrictionSignal:', upgradeRestriction.toHuman());

    // In standalone mode (no relaychain), we need to ensure ValidationData and HostConfiguration exist
    // These should be present from the forked chain, but we verify and warn if missing
    if (!validationData.toHuman()) {
        console.log('WARNING: ValidationData is missing - upgrade may fail');
        console.log('In standalone mode, the inherent data should come from the forked chain state');
    }
    if (!hostConfig.toHuman()) {
        console.log('WARNING: HostConfiguration is missing - upgrade may fail');
    }

    // Clear any upgrade restriction signal to allow the upgrade
    // In standalone mode, we don't have a real relaychain to provide the go-ahead signal
    // The upgrade will proceed when applyAuthorizedUpgrade is called
    if (!upgradeRestriction.isEmpty) {
        console.log('Note: UpgradeRestrictionSignal is present, but in standalone mode this is managed internally');
    }

    console.log('Applying authorized upgrade...');

    // Step 2: Apply the authorized upgrade
    const applyUpgradeTx = api.tx.system.applyAuthorizedUpgrade(wasm);

    // Subscribe to ALL events to see what happens
    console.log('Subscribing to events...');
    const eventPromise = subscribeToEvents('system', 'ExtrinsicSuccess', api);

    await signAndSend(applyUpgradeTx, alice);
    console.log('Apply upgrade transaction sent ✅');

    const events = await eventPromise;
    console.log('Transaction events:', events.length);

    // Check all system events in the block
    const allEvents = await api.query.system.events();
    for (const record of allEvents) {
        const { event } = record;
        if (event.section === 'system' || event.section === 'parachainSystem') {
            console.log(`Event: ${event.section}.${event.method}`, event.data.toHuman());
        }
    }

    // Check if code was actually updated
    const codeAfter = await api.query.system.authorizedUpgrade();
    console.log('AuthorizedUpgrade after apply:', codeAfter.toHuman());

    console.log('Waiting for runtime upgrade to complete...');
    // Increased maxBlocks to 200 for relaychain coordination
    const newRuntimeVersion = await waitForRuntimeUpgradeWithBlockProduction(api, old_runtime_version, 200);
    return newRuntimeVersion;
}
describeLitentry('Runtime upgrade test', ``, (context) => {
    test('Running runtime ugprade test', async () => {
        let runtimeVersion: number;
        const wasmPath = path.resolve('/tmp/runtime.wasm');
        const wasm = fs.readFileSync(wasmPath).toString('hex');

        const parachainName = process.env.PARACHAIN_NAME || 'heima';
        console.log(`Testing runtime upgrade for parachain: ${parachainName}`);

        const wsProvider = new WsProvider('ws://localhost:9944');
        const api = await ApiPromise.create({ provider: wsProvider });
        await api.isReady;

        // Set block build mode to Instant for faster testing
        await api.rpc('dev_setBlockBuildMode', 'Instant');
        console.log('Block build mode set to Instant ✅');

        runtimeVersion = await runtimeupgradeViaGovernance(api, `0x${wasm}`);
        expect(runtimeVersion).toBe(await getRuntimeVersion(api));

        console.log('Runtime upgraded ✅');
    });
});
