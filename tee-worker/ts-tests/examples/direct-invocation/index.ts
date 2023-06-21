import { cryptoWaitReady } from '@polkadot/util-crypto';
import { KeyringPair } from '@polkadot/keyring/types';
import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import { default as teeTypes } from '../../parachain-interfaces/identity/definitions';
import { HexString } from '@polkadot/util/types';
import {
    createSignedTrustedCallSetUserShieldingKey,
    sendRequestFromTrustedCall,
    getTEEShieldingKey,
    createSignedTrustedCallLinkIdentity,
    createSignedTrustedGetterUserShieldingKey,
    sendRequestFromGetter,
    getSidechainNonce,
} from './util';
import { getEnclave, sleep, buildIdentityHelper, initIntegrationTestContext } from '../../common/utils';
import { aesKey, keyNonce } from '../../common/call';
import { Metadata, TypeRegistry } from '@polkadot/types';
import sidechainMetaData from '../../litentry-sidechain-metadata.json';
import { hexToU8a } from '@polkadot/util';
import { assert } from 'chai';

// in order to handle self-signed certificates we need to turn off the validation
// TODO add self signed certificate
process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

const webSocketAsPromised = require('websocket-as-promised');
const webSocket = require('ws');
const keyring = new Keyring({ type: 'sr25519' });

const PARACHAIN_WS_ENDPINT = 'ws://localhost:9944';
const WORKER_TRUSTED_WS_ENDPOINT = 'wss://localhost:2000';

async function runDirectCall() {
    const parachainWs = new WsProvider(PARACHAIN_WS_ENDPINT);
    const sidechainRegistry = new TypeRegistry();
    const metaData = new Metadata(sidechainRegistry, sidechainMetaData.result as HexString);
    sidechainRegistry.setMetadata(metaData);
    const { types } = teeTypes;
    const parachainApi = await ApiPromise.create({
        provider: parachainWs,
        types,
    });
    const context = await initIntegrationTestContext(WORKER_TRUSTED_WS_ENDPOINT, PARACHAIN_WS_ENDPINT, 0);

    await cryptoWaitReady();
    const wsp = new webSocketAsPromised(WORKER_TRUSTED_WS_ENDPOINT, {
        createWebSocket: (url: any) => new webSocket(url),
        extractMessageData: (event: any) => event,
        packMessage: (data: any) => JSON.stringify(data),
        unpackMessage: (data: string) => JSON.parse(data),
        attachRequestId: (data: any, requestId: string | number) => Object.assign({ id: requestId }, data),
        extractRequestId: (data: any) => data && data.id,
    });
    await wsp.open();

    const key = await getTEEShieldingKey(wsp, parachainApi);

    const alice: KeyringPair = keyring.addFromUri('//Alice', { name: 'Alice' });
    const bob: KeyringPair = keyring.addFromUri('//Bob', { name: 'Bob' });
    const mrenclave = (await getEnclave(parachainApi)).mrEnclave;

    let nonce = await getSidechainNonce(wsp, parachainApi, mrenclave, key, alice.address);

    // a hardcoded AES key which is used overall in tests - maybe we need to put it in a common place
    // const key_alice = '0x22fc82db5b606998ad45099b7978b5b4f9dd4ea6017e57370ac56141caaabd12';


    // similar to `reqExtHash` in indirect calls, we need some "identifiers" to pair the response
    // with the request. Ideally it's the hash of the trusted operation, but we need it before constructing
    // a trusted call, hence a random number is used here - better ideas are welcome
    let hash = `0x${require('crypto').randomBytes(32).toString('hex')}`;
    console.log('Send direct setUserShieldingKey call for alice ... hash:', hash);
    let setUserShieldingKeyCall = createSignedTrustedCallSetUserShieldingKey(
        parachainApi,
        mrenclave,
        nonce,
        alice,
        aesKey,
        hash
    );
    let res = await sendRequestFromTrustedCall(wsp, parachainApi, mrenclave, key, setUserShieldingKeyCall);
    console.log('setUserShieldingKey call returned', res.toHuman());

    sleep(10);

    hash = `0x${require('crypto').randomBytes(32).toString('hex')}`;

    nonce = await getSidechainNonce(wsp, parachainApi, mrenclave, key, alice.address);

    console.log('Send direct createIdentity call... hash:', hash);
    const twitterIdentity = await buildIdentityHelper('mock_user', 'Twitter', 'Web2', context);
    const linkIdentityCall = createSignedTrustedCallLinkIdentity(
        parachainApi,
        mrenclave,
        nonce,
        alice,
        sidechainRegistry.createType('LitentryPrimitivesIdentity', twitterIdentity).toHex(),
        parachainApi
            .createType('LitentryValidationData', {
                Web2Validation: {
                    Twitter: {
                        tweet_id: `0x${Buffer.from('100', 'utf8').toString('hex')}`,
                    },
                },
            })
            .toHex(),
        keyNonce,
        hash
    );
    res = await sendRequestFromTrustedCall(wsp, parachainApi, mrenclave, key, linkIdentityCall);
    console.log('linkIdentity call returned', res.toHuman());

    sleep(10);

    console.log('Send UserShieldingKey getter for alice ...');
    let UserShieldingKeyGetter = createSignedTrustedGetterUserShieldingKey(parachainApi, alice);
    res = await sendRequestFromGetter(wsp, parachainApi, mrenclave, key, UserShieldingKeyGetter);
    console.log('UserShieldingKey getter returned', res.toHuman());
    // the returned res.value of the trustedGetter is of Option<> type
    // res.value should be `0x018022fc82db5b606998ad45099b7978b5b4f9dd4ea6017e57370ac56141caaabd12`
    // TODO: why `createType` must accept an Uint8Array here? The following still prints the unwrapped value
    //       let k = parachainApi.createType('Option<Bytes>', res.value.toHex());
    //       console.log("k.isSome", k.isSome); // true
    //       console.log("k.unwrap", k.unwrap().toHex()); // still 0x018022fc82db5b606998ad45099b7978b5b4f9dd4ea6017e57370ac56141caaabd12
    let k = parachainApi.createType('Option<Bytes>', hexToU8a(res.value.toHex()));
    assert.isTrue(k.isSome);
    assert.equal(k.unwrap().toHex(), aesKey);

    // bob's shielding key should be none
    console.log('Send UserShieldingKey getter for bob ...');
    UserShieldingKeyGetter = createSignedTrustedGetterUserShieldingKey(parachainApi, bob);
    res = await sendRequestFromGetter(wsp, parachainApi, mrenclave, key, UserShieldingKeyGetter);
    console.log('UserShieldingKey getter returned', res.toHuman());
    k = parachainApi.createType('Option<Bytes>', hexToU8a(res.value.toHex()));
    assert.isTrue(k.isNone);

    sleep(10);

    nonce = await getSidechainNonce(wsp, parachainApi, mrenclave, key, bob.address);

    // set bob's shielding key, with wrapped bytes
    const keyBob = '0x8378193a4ce64180814bd60591d1054a04dbc4da02afde453799cd6888ee0c6c';
    hash = `0x${require('crypto').randomBytes(32).toString('hex')}`;
    console.log('Send direct setUserShieldingKey call for bob, with wrapped bytes... hash:', hash);
    setUserShieldingKeyCall = createSignedTrustedCallSetUserShieldingKey(
        parachainApi,
        mrenclave,
        nonce,
        bob,
        keyBob,
        hash,
        true // with wrapped bytes
    );
    res = await sendRequestFromTrustedCall(wsp, parachainApi, mrenclave, key, setUserShieldingKeyCall);
    console.log('setUserShieldingKey call returned', res.toHuman());

    // verify that bob's key is set
    console.log('Send UserShieldingKey getter for bob ...');
    UserShieldingKeyGetter = createSignedTrustedGetterUserShieldingKey(parachainApi, bob);
    res = await sendRequestFromGetter(wsp, parachainApi, mrenclave, key, UserShieldingKeyGetter);
    console.log('UserShieldingKey getter returned', res.toHuman());
    k = parachainApi.createType('Option<Bytes>', hexToU8a(res.value.toHex()));
    assert.isTrue(k.isSome);
    assert.equal(k.unwrap().toHex(), keyBob);
}

(async () => {
    await runDirectCall().catch((e) => {
        console.error(e);
    });
    process.exit(0);
})();
