import { randomBytes, KeyObject } from 'crypto';
import { step } from 'mocha-steps';
import { assert } from 'chai';
import { decryptWithAes, initIntegrationTestContext, PolkadotSigner } from './common/utils';
import { randomSubstrateWallet } from './common/helpers';
import { assertIsInSidechainBlock, assertVc } from './common/utils/assertion';
import {
    getSidechainNonce,
    getTeeShieldingKey,
    sendRequestFromTrustedCall,
    createSignedTrustedCallRequestVc,
    createSignedTrustedGetterIdGraph,
    sendAesRequestFromGetter,
    decodeIdGraph,
} from './common/di-utils'; // @fixme move to a better place
import type { IntegrationTestContext } from './common/common-types';
import { HeimaPrimitivesIdentity, WorkerRpcReturnValue } from '@heima-network/api-argument/identity';
import { aesKey } from './common/call';
import { $ as zx } from 'zx';
import { KeyringPair } from '@polkadot/keyring/types';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { CredentialDefinition, credentialsJson } from './common/credential-json';
import { byId } from '@litentry/chaindata';
import { sleep } from './common/utils';
// Change this to the environment you want to test
const chain = byId['litentry-dev'];

describe('Test Vc (direct invocation)', function () {
    let context: IntegrationTestContext = undefined as any;
    let teeShieldingKey: KeyObject = undefined as any;
    const substrateIdentities: HeimaPrimitivesIdentity[] = [];

    const clientDir = process.env.LITENTRY_CLI_DIR;

    const keyringPairs: KeyringPair[] = [];
    let argvId = '';

    const nodeEndpoint: string = chain.rpcs[0].url;
    const enclaveEndpoint: string = chain.enclaveRpcs[0].url;
    console.log(`[node] ${nodeEndpoint}`);
    console.log(`[worker] ${enclaveEndpoint}`);

    const teeDevNodePort = 443;
    const teeDevWorkerPort = 443;
    const errorArray: { id: string; index: number; assertion: any; error: any }[] = [];
    this.timeout(6000000);
    before(async () => {
        context = await initIntegrationTestContext(nodeEndpoint, enclaveEndpoint);
        teeShieldingKey = await getTeeShieldingKey(context);
    });

    // usage example:
    // `pnpm run test-data-providers:local --id=vip3-membership-card-gold` for single test
    // `pnpm run test-data-providers:local` for all tests
    const idIndex = process.argv.indexOf('--id');
    argvId = process.argv[idIndex + 1];
    const { protocol: workerProtocal, hostname: workerHostname } = new URL(enclaveEndpoint);
    const { protocol: nodeProtocal, hostname: nodeHostname } = new URL(nodeEndpoint);

    async function linkIdentityViaCli(id: string) {
        const credentialDefinitions = credentialsJson.find((item) => item.id === id) as CredentialDefinition;
        console.log(`linking identity-${credentialDefinitions.mockDid} via cli`);

        const keyringPair = randomSubstrateWallet();
        keyringPairs.push(keyringPair);
        const formatAddress = u8aToHex(keyringPair.publicKey);
        const substrateIdentity = await new PolkadotSigner(keyringPair).getIdentity(context);
        substrateIdentities.push(substrateIdentity);

        try {
            // CLIENT = "$CLIENT_BIN -p $NPORT -P $WORKER1PORT -u $NODEURL -U $WORKER1URL"
            const commandPromise = zx`${clientDir} -p ${teeDevNodePort} -P ${teeDevWorkerPort} -u ${
                nodeProtocal + nodeHostname
            } -U ${workerProtocal + workerHostname}\
                  trusted -d link-identity did:litentry:substrate:${formatAddress}\
                  did:${credentialDefinitions.mockDid}\
                  ${credentialDefinitions.mockWeb3Network}`;

            await commandPromise;

            const idGraphGetter = await createSignedTrustedGetterIdGraph(
                context.api,
                new PolkadotSigner(keyringPair),
                substrateIdentity
            );
            const res = await sendAesRequestFromGetter(context, teeShieldingKey, hexToU8a(aesKey), idGraphGetter);

            const idGraph = decodeIdGraph(context.sidechainRegistry, res.value);

            assert.lengthOf(idGraph, 2, 'idGraph length should be 2');
        } catch (error: any) {
            console.log(`Exit code: ${error.exitCode}`);
            console.log(`Error: ${error.stderr}`);

            throw error;
        }
    }

    async function requestVc(id: string, index: number) {
        try {
            const credentialDefinitions = credentialsJson.find((item) => item.id === id) as CredentialDefinition;
            const assertion = {
                [credentialDefinitions.assertion.id]: credentialDefinitions.assertion.payload,
            };
            console.log('vc description: ', credentialDefinitions.description);

            console.log('assertion: ', assertion);

            let currentNonce = (await getSidechainNonce(context, substrateIdentities[index])).toNumber();
            const getNextNonce = () => currentNonce++;
            const nonce = getNextNonce();

            const requestIdentifier = `0x${randomBytes(32).toString('hex')}`;
            const requestVcCall = await createSignedTrustedCallRequestVc(
                context.api,
                context.mrEnclave,
                context.api.createType('Index', nonce),
                new PolkadotSigner(keyringPairs[index]),
                substrateIdentities[index],
                context.api.createType('Assertion', assertion).toHex(),
                context.api.createType('Option<RequestAesKey>', aesKey).toHex(),
                requestIdentifier
            );

            const onMessageReceived = async (res: WorkerRpcReturnValue) => {
                const vcresponse = context.api.createType('RequestVcResultOrError', res.value);
                console.log(`vcresponse len: ${vcresponse.len}, idx: ${vcresponse.idx}`);
                if (vcresponse.result.isOk) await assertVc(context, substrateIdentities[index], vcresponse.result.asOk);
                const decryptVcPayload = decryptWithAes(aesKey, vcresponse.vc_payload, 'utf-8').replace('0x', '');
                const vcPayloadJson = JSON.parse(decryptVcPayload);
                assert.equal(
                    vcPayloadJson.credentialSubject.values[0],
                    credentialDefinitions.expectedCredentialValue,
                    "credential value doesn't match, please check the credential json expectedCredentialValue"
                );
            };
            const callResults = await sendRequestFromTrustedCall(
                context,
                teeShieldingKey,
                requestVcCall,
                onMessageReceived
            );
            await assertIsInSidechainBlock(`${Object.keys(assertion)[0]} requestVcCall`, callResults);
        } catch (error) {
            // Sometimes unstable dataprovider can cause interruptions in the testing process. We expect errors in the test to be stored and specific error information to be thrown out after the end.
            const credentialDefinitions = credentialsJson.find((item) => item.id === id) as CredentialDefinition;

            errorArray.push({
                id: id,
                index: index,
                assertion: JSON.stringify(credentialDefinitions.assertion),
                error: error,
            });
            console.error(`Error in requestVc for id ${id} at index ${index}:`, error);
        }
        //  sometime the previous nonce change hasn't been persisted in the sidechain state while next request is already coming
        await sleep(12);
    }

    if (argvId && credentialsJson.find((item) => item.id === argvId)) {
        step(`link identity && request vc with specific credentials for ${argvId}`, async function () {
            await linkIdentityViaCli(argvId);
            await requestVc(argvId, 0);
        });
    } else {
        credentialsJson.forEach(({ id }, index) => {
            step(`link identity && request vc with all credentials for ${id}`, async function () {
                await linkIdentityViaCli(id);
                await requestVc(id, index);
            });
        });
    }
    after(async function () {
        if (errorArray.length > 0) {
            console.log('errorArray:', errorArray);
            throw new Error(`${errorArray.length} tests failed. See above for details.`);
        }
    });
});
