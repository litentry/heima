import { assert } from 'chai';
import { step } from 'mocha-steps';
import { KeyObject, randomBytes } from 'crypto';
import * as ed from '@noble/ed25519';
import { base58Encode } from '@polkadot/util-crypto';
import { hexToU8a } from '@polkadot/util';
import { validateVcSchema } from '@litentry/vc-schema-validator';
import {
    CorePrimitivesIdentity,
    RequestVcResultOrError,
    RequestVcOk,
    TrustedCallResult,
    WorkerRpcReturnValue,
} from 'parachain-api';
import { IntegrationTestContext, SubstrateSigner } from './common/common-types';
import {
    Web2ValidationConfig,
    buildIdentityHelper,
    buildWeb2Validation,
    decryptWithAes,
    initIntegrationTestContext,
    mockBatchAssertion,
} from './common/utils';
import { subscribeToEventsWithExtHash, subscribeToEvents } from './common/transactions';
import { getTeeShieldingKey } from './common/di-utils';
import { aesKey, sendRequest } from './common/call';
import { nextRequestId } from './common/helpers';
import {
    buildWeb3ValidationData,
    createAuthenticatedTrustedCallAddAccount,
    createAuthenticatedTrustedCallRequestBatchVc,
    createAuthenticatedTrustedCallRequestVc,
    createOmniAccountPermission,
    getOmniAccount,
    getOmniAccountNonce,
    sendRequestFromTrustedCall,
} from './common/utils/native-request-helpers';

describe('Test native vc_request', function () {
    this.timeout(6000000);
    let teeShieldingKey: KeyObject;
    let context: IntegrationTestContext;
    let aliceSubstrateWallet: SubstrateSigner;
    let aliceSubstrateIdentity: CorePrimitivesIdentity;
    let omniAccount: string;

    before(async function () {
        const parachainEndpoint = process.env.PARACHAIN_ENDPOINT;
        if (!parachainEndpoint) {
            throw new Error('PARACHAIN_ENDPOINT environment variable is missing.');
        }
        context = await initIntegrationTestContext(parachainEndpoint);
        teeShieldingKey = await getTeeShieldingKey(context);
        const wallet = context.web3Wallets['substrate'];
        aliceSubstrateWallet = wallet['Alice'] as SubstrateSigner;
        aliceSubstrateIdentity = await aliceSubstrateWallet.getIdentity(context);
        omniAccount = await getOmniAccount(context.api, aliceSubstrateIdentity);
    });

    step('Request VC with no AccountStore created before', async function () {
        const a1Assertion = {
            description: 'Have identified at least one account/address in both Web2 and Web3.',
            assertion: {
                A1: 'A1',
            },
        };
        console.log(`request vc A1 for Alice ... Assertion description: ${a1Assertion.description}`);
        const requestIdentifier = `0x${randomBytes(32).toString('hex')}`;
        const requestVcCall = await createAuthenticatedTrustedCallRequestVc(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', 0),
            aliceSubstrateWallet,
            aliceSubstrateIdentity,
            context.api.createType('Assertion', a1Assertion.assertion).toHex(),
            context.api.createType('Option<RequestAesKey>', aesKey).toHex(),
            requestIdentifier
        );
        const onMessageReceived = async (res: WorkerRpcReturnValue) => {
            const trustedCallResult: TrustedCallResult = context.api.createType('TrustedCallResult', res.value);
            if (trustedCallResult.isOk && trustedCallResult.asOk.isRequestVcResult) {
                const requestVcResultOrError: RequestVcResultOrError = trustedCallResult.asOk.asRequestVcResult;
                if (requestVcResultOrError.result.isOk) {
                    const requestVcOk: RequestVcOk = context.api.createType(
                        'RequestVcOk',
                        requestVcResultOrError.result.asOk
                    );
                    await assertVc(context, aliceSubstrateIdentity, requestVcOk);
                }
            }
        };
        const vcEventsPromise = subscribeToEventsWithExtHash(requestIdentifier, context);
        const omniAccountEventsPromise = subscribeToEvents('omniAccount', 'AccountStoreCreated', context.api);

        await sendRequestFromTrustedCall(context, teeShieldingKey, requestVcCall, onMessageReceived);

        console.log('Waiting for events...');
        const vcEvents = await vcEventsPromise;
        assert.equal(vcEvents.length, 1);
        const omniAccountEvents = await omniAccountEventsPromise;
        assert.equal(omniAccountEvents.length, 1);
    });

    step('Add accounts to the AccountStore', async function () {
        let accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        assert.isTrue(accountStore.isSome, 'account store not found');
        let membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 1, 'account store members count should be 1');

        let currentNonce = (await getOmniAccountNonce(context.api, aliceSubstrateIdentity)).toNumber();
        // twitter
        const twitterIdentity = await buildIdentityHelper('mock_user', 'Twitter', context);
        const validationConfig: Web2ValidationConfig = {
            identityType: 'Twitter',
            context,
            signerIdentitity: aliceSubstrateIdentity,
            linkIdentity: twitterIdentity,
            verificationType: 'PublicTweet',
            validationNonce: currentNonce,
        };
        const twitterValidationData = await buildWeb2Validation(validationConfig);
        const addTwitterAccountCall = await createAuthenticatedTrustedCallAddAccount(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceSubstrateWallet,
            aliceSubstrateIdentity,
            twitterIdentity,
            twitterValidationData.toHex(),
            true, // public account
            [createOmniAccountPermission(context.api, 'All')]
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, addTwitterAccountCall);
        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 2, 'account store members count should be 2');

        // EVM
        currentNonce++;
        const evmSigner = context.web3Wallets.evm.Alice;
        const evmIdentity = await evmSigner.getIdentity(context);
        const evmValidationData = await buildWeb3ValidationData(
            context,
            aliceSubstrateIdentity,
            evmIdentity,
            currentNonce,
            'evm',
            evmSigner
        );
        const addEvmAccountCall = await createAuthenticatedTrustedCallAddAccount(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceSubstrateWallet,
            aliceSubstrateIdentity,
            evmIdentity,
            evmValidationData.toHex(),
            true, // public account
            [createOmniAccountPermission(context.api, 'All')]
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, addEvmAccountCall);
        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 3, 'account store members count should be 3');

        // Bitcoin
        currentNonce++;
        const bitcoinSigner = context.web3Wallets.bitcoin.Alice;
        const bitcoinIdentity = await bitcoinSigner.getIdentity(context);
        const bitcoinValidationData = await buildWeb3ValidationData(
            context,
            aliceSubstrateIdentity,
            bitcoinIdentity,
            currentNonce,
            'bitcoin',
            bitcoinSigner
        );
        const addBitcoinAccountCall = await createAuthenticatedTrustedCallAddAccount(
            context.api,
            context.mrEnclave,
            context.api.createType('Index', currentNonce),
            aliceSubstrateWallet,
            aliceSubstrateIdentity,
            bitcoinIdentity,
            bitcoinValidationData.toHex(),
            true, // public account
            [createOmniAccountPermission(context.api, 'All')]
        );
        await sendRequestFromTrustedCall(context, teeShieldingKey, addBitcoinAccountCall);
        accountStore = await context.api.query.omniAccount.accountStore(omniAccount);
        membersCount = accountStore.unwrap().length;
        assert.equal(membersCount, 4, 'account store members count should be 4');
    });

    // Request VC with AccountStore created before
    mockBatchAssertion.forEach(({ description, assertion }) => {
        step(`Request VC payload: ${JSON.stringify(assertion)} (alice)`, async function () {
            const currentNonce = (await getOmniAccountNonce(context.api, aliceSubstrateIdentity)).toNumber();
            const requestIdentifier = `0x${randomBytes(32).toString('hex')}`;
            console.log(
                `request vc direct ${Object.keys(assertion)[0]} for Alice ... Assertion description: ${description}`
            );

            let requestVcCall;
            if (Array.isArray(assertion)) {
                console.log('>>>> Batch Assertions');
                requestVcCall = await createAuthenticatedTrustedCallRequestBatchVc(
                    context.api,
                    context.mrEnclave,
                    context.api.createType('Index', currentNonce),
                    aliceSubstrateWallet,
                    aliceSubstrateIdentity,
                    context.api.createType('Vec<Assertion>', assertion).toHex(),
                    context.api.createType('Option<RequestAesKey>', aesKey).toHex(),
                    requestIdentifier
                );
            } else {
                requestVcCall = await createAuthenticatedTrustedCallRequestVc(
                    context.api,
                    context.mrEnclave,
                    context.api.createType('Index', currentNonce),
                    aliceSubstrateWallet,
                    aliceSubstrateIdentity,
                    context.api.createType('Assertion', assertion).toHex(),
                    context.api.createType('Option<RequestAesKey>', aesKey).toHex(),
                    requestIdentifier
                );
            }

            // Instead of waiting for final response we will listen all responses from the call
            const onMessageReceived = async (res: WorkerRpcReturnValue) => {
                const trustedCallResult: TrustedCallResult = context.api.createType('TrustedCallResult', res.value);
                if (trustedCallResult.isOk && trustedCallResult.asOk.isRequestVcResult) {
                    const requestVcResultOrError: RequestVcResultOrError = trustedCallResult.asOk.asRequestVcResult;
                    if (requestVcResultOrError.result.isOk) {
                        const requestVcOk: RequestVcOk = context.api.createType(
                            'RequestVcOk',
                            requestVcResultOrError.result.asOk
                        );
                        await assertVc(context, aliceSubstrateIdentity, requestVcOk);
                    }
                }
            };

            const vcEventsPromise = subscribeToEventsWithExtHash(requestIdentifier, context);

            await sendRequestFromTrustedCall(context, teeShieldingKey, requestVcCall, onMessageReceived);

            console.log('Waiting for events...');
            const vcEvents = await vcEventsPromise;
            assert.equal(vcEvents.length, Array.isArray(assertion) ? assertion.length : 1);
        });
    });
});

async function assertVc(context: IntegrationTestContext, subject: CorePrimitivesIdentity, result: RequestVcOk) {
    // step 1
    // decryptWithAes function added 0x prefix
    const decryptedVcPayload = decryptWithAes(aesKey, result.vc_payload, 'utf-8').replace('0x', '');

    // step 2
    // check credential subject's DID
    const credentialSubjectId = JSON.parse(decryptedVcPayload).credentialSubject.id;
    const expectSubject = Object.entries(JSON.parse(subject.toString()));

    // step 3
    // convert to DID format
    const expectDid = 'did:litentry:' + expectSubject[0][0] + ':' + expectSubject[0][1];
    assert.equal(
        expectDid,
        credentialSubjectId,
        'Check credentialSubject error: expectDid should be equal to credentialSubject id'
    );

    // step 4
    // extrac proof and vc without proof json
    const vcPayloadJson = JSON.parse(decryptedVcPayload);
    console.log('credential: ', JSON.stringify(vcPayloadJson, null, 2));
    const { proof, ...vcWithoutProof } = vcPayloadJson;

    // step 5
    // check vc signature
    const signature = Buffer.from(hexToU8a(`0x${proof.proofValue}`));
    const message = Buffer.from(JSON.stringify(vcWithoutProof));
    const vcPubkey = Buffer.from(hexToU8a(proof.verificationMethod));
    const signatureStatus = await ed.verify(signature, message, vcPubkey);
    assert.isTrue(signatureStatus, 'Check Vc signature error: signature should be valid');

    // step 6
    // lookup the teebag enclave regsitry to check mrenclave and vcPubkey
    const parachainBlockHash = await context.api.query.system.blockHash(vcPayloadJson.parachainBlockNumber);
    const apiAtVcIssuedBlock = await context.api.at(parachainBlockHash);
    const enclaveAccount = trimPrefix(vcPayloadJson.issuer.id, 'did:litentry:substrate:');
    const registeredEnclave = (await apiAtVcIssuedBlock.query.teebag.enclaveRegistry(enclaveAccount)).unwrap();

    assert.equal(
        vcPayloadJson.issuer.mrenclave,
        base58Encode(registeredEnclave.mrenclave),
        "Check VC mrenclave: it should equal enclave's mrenclave from parachains enclave registry"
    );

    assert.equal(
        proof.verificationMethod,
        registeredEnclave.vcPubkey,
        "Check VC pubkey: it should equal enclave's vcPubkey from parachains enclave registry"
    );

    // step 7
    // check runtime version is present
    const [parachainSpecVersion, sidechainSpecVersion] = await Promise.all([
        context.api.rpc.state.getRuntimeVersion(),
        sendRequest(
            context.tee,
            {
                jsonrpc: '2.0',
                id: nextRequestId(context),
                method: 'state_getRuntimeVersion',
                params: [],
            },
            context.api
        ),
    ]).then(([parachainRuntime, sidechainReturnValue]) => {
        const sidechainRuntime = context.api.createType('RuntimeVersion', sidechainReturnValue.value);

        return [parachainRuntime.specVersion.toNumber(), sidechainRuntime.specVersion.toNumber()];
    });

    assert.deepEqual(
        vcPayloadJson.issuer.runtimeVersion,
        { parachain: parachainSpecVersion, sidechain: sidechainSpecVersion },
        'Check VC runtime version: it should equal the current defined versions'
    );

    // step 8
    // validate VC against schema
    const schemaResult = await validateVcSchema(vcPayloadJson);

    if (schemaResult.errors) console.log('Schema Validation errors: ', schemaResult.errors);

    assert.isTrue(schemaResult.isValid, 'Check Vc payload error: vcPayload should be valid');

    assert.equal(
        vcWithoutProof.type[0],
        'VerifiableCredential',
        'Check Vc payload type error: vcPayload type should be VerifiableCredential'
    );
    assert.equal(
        proof.type,
        'Ed25519Signature2020',
        'Check Vc proof type error: proof type should be Ed25519Signature2020'
    );
}

function trimPrefix(str: string, prefix: string): string {
    if (str.startsWith(prefix)) {
        return str.substring(prefix.length);
    }
    return str;
}
