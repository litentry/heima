import WebSocket from 'ws';
import { assert } from 'chai';
import { webcrypto } from 'crypto';
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';

import { identity, trusted_operations, sidechain, vc } from '@litentry/parachain-api';
import { request, createLitentryIdentityType } from '@litentry/client-sdk';

import { nodeEndpoint, enclaveEndpoint } from '../config';
import { u8aToHex, u8aToString } from '@polkadot/util';

let api: ApiPromise;
let keyring: Keyring;

const originalCrypto = globalThis.crypto;

before(async () => {
    console.log(`[node] ${nodeEndpoint}`);
    console.log(`[worker] ${enclaveEndpoint}`);
    console.log(`-----------------------------------`, '\n');

    const wsProvider = new WsProvider(nodeEndpoint);

    api = await ApiPromise.create({
        provider: wsProvider,
        // Hardcoded for now until we need to support non-litentry networks
        types: Object.assign({}, identity.types, trusted_operations.types, sidechain.types, vc.types),
    });

    keyring = new Keyring({ type: 'sr25519' });

    //
    // Web API Mocks for @litentry/enclave
    //

    // enable globalThis.crypto.subtle
    Object.defineProperty(global, 'crypto', {
        value: webcrypto,
        writable: true,
    });
    // expose global websocket
    Object.defineProperty(global, 'WebSocket', {
        value: WebSocket,
        writable: true,
    });
});

after(() => {
    api.disconnect();

    // restore globalThis.crypto
    Object.defineProperty(global, 'crypto', {
        value: originalCrypto,
        writable: true,
    });
    // expose global websocket
    Object.defineProperty(global, 'WebSocket', {
        value: undefined,
        writable: true,
    });
});

/**
 * This test implicitly checks for:
 *
 * 1. clients can connect to the Parachain's Node and the Enclave's Worker over WSS.
 * 2. a registered Enclave worker exists in the Parachain's storage.
 * 3. the registered Enclave matches the running one.
 *
 */
it(`can issue a verifiable-credential for Alice`, async () => {
    const alice = keyring.addFromUri('//Alice');
    const aliceIdentity = createLitentryIdentityType(api.registry, {
        addressOrHandle: alice.address,
        type: 'Substrate',
    });

    assert.strictEqual(u8aToHex(alice.addressRaw), aliceIdentity.asSubstrate.toHex());

    // Any Enclave operation requires fetching a nonce from `enclaveEndpoint` using the
    // shard registered in the `nodeEndpoint`.
    // it implicitly ensures that the Enclave is running and correctly registered in the Parachain.
    const { send, payloadToSign, txHash } = await request.requestBatchVC(api, {
        who: aliceIdentity,
        assertions: [
            // lit-holder
            api.createType('Assertion', {
                A4: '10.00',
            }),
        ],
    });

    assert.isString(payloadToSign);
    const signature = alice.sign(payloadToSign);

    const response = await send({
        signedPayload: u8aToHex(signature),
    });

    assert.strictEqual(response.txHash, txHash);
    assert.isArray(response.vcPayloads);
    assert.notInstanceOf(response.vcPayloads[0], Error);
    assert.instanceOf(response.vcPayloads[0], Uint8Array);

    const vcString = JSON.parse(u8aToString(response.vcPayloads[0] as Uint8Array));
    assert.isArray(vcString['@context']);
    assert.isString(vcString.id);
});

/**
 * This test implicitly checks that the state is readable and consistent
 *
 * Notice that this test is fixed to the historical state of tee-prod.
 */
it(`[PROD] should have consistent historical state for Alice`, async () => {
    const alice = keyring.addFromUri('//Alice');
    const aliceIdentity = createLitentryIdentityType(api.registry, {
        addressOrHandle: alice.address,
        type: 'Substrate',
    });

    assert.strictEqual(u8aToHex(alice.addressRaw), aliceIdentity.asSubstrate.toHex());

    const { send, payloadToSign } = await request.getIdGraph(api, {
        who: aliceIdentity,
    });

    const signature = alice.sign(payloadToSign);

    const { idGraph } = await send({
        signedPayload: u8aToHex(signature),
    });

    // Alice has at least, the following identities:
    const bob = keyring.addFromUri('//Bob');
    const eve = keyring.addFromUri('//Eve');

    assert.isTrue(
        idGraph.some(([identity]) => identity.asSubstrate.eq(alice.addressRaw)),
        'Alice is not in the graph'
    );
    assert.isTrue(
        idGraph.some(([identity]) => identity.asSubstrate.eq(bob.addressRaw)),
        'Bob is not in the graph'
    );
    assert.isTrue(
        idGraph.some(([identity]) => identity.asSubstrate.eq(eve.addressRaw)),
        'Eve is not in the graph'
    );
});
