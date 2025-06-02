import { hexToU8a, u8aToHex } from '@polkadot/util';
import { assert } from 'chai';
import * as ed from '@noble/ed25519';
import { parseIdGraph } from './identity-helper';
import { HeimaPrimitivesIdentity, WorkerRpcReturnValue, StfError } from '@heima-network/api-argument/identity';
import type { IntegrationTestContext } from '../common-types';
import { getIdGraphHash } from '../di-utils';
import type { HexString } from '@polkadot/util/types';
import { nextRequestId } from '../helpers';
import { aesKey, sendRequest } from '../call';
import colors from 'colors';
import { Bytes } from '@polkadot/types-codec';
import { decryptWithAes } from './crypto';
import { base58Encode, blake2AsHex } from '@polkadot/util-crypto';
import { validateVcSchema } from '@litentry/vc-schema-validator';
import { PalletIdentityManagementTeeIdentityContext } from '@heima-network/api-argument/sidechain';
import { KeyObject } from 'crypto';

export function assertIdGraph(
    actual: [HeimaPrimitivesIdentity, PalletIdentityManagementTeeIdentityContext][],
    expected: [HeimaPrimitivesIdentity, boolean][]
) {
    assert.equal(actual.length, expected.length);
    expected.forEach((expected, i) => {
        assert.deepEqual(
            actual[i][0].toJSON(),
            expected[0].toJSON(),
            'event idGraph identity should be equal expectedIdentity'
        );

        const idGraphContext = actual[0][1];
        assert.isTrue(idGraphContext.linkBlock.toNumber() > 0, 'link_block should be greater than 0');
        assert.equal(idGraphContext.status.isActive, expected[1], 'isActive should be ' + expected[1]);
    });
}

export async function assertIsInSidechainBlock(callType: string, res: WorkerRpcReturnValue) {
    assert.isTrue(
        res.status.isTrustedOperationStatus,
        `${callType} should be trusted operation status, but is ${res.status.type}`
    );
    const status = res.status.asTrustedOperationStatus;
    console.log(res.toHuman());

    assert.isTrue(
        status[0].isSubmitted || status[0].isInSidechainBlock,
        `${callType} should be submitted or in sidechain block, but is ${status[0].type}`
    );
}

export function assertWorkerError(
    context: IntegrationTestContext,
    check: (returnValue: StfError) => void,
    returnValue: WorkerRpcReturnValue
) {
    const errValueDecoded = context.api.createType('StfError', returnValue.value);
    check(errValueDecoded);
}

// a common assertion for all DI requests that might mutate the IdGraph
// returns the `id_graph_hash` in the `returnValue`
export async function assertIdGraphMutationResult(
    context: IntegrationTestContext,
    teeShieldingKey: KeyObject,
    identity: HeimaPrimitivesIdentity,
    returnValue: WorkerRpcReturnValue,
    resultType:
        | 'LinkIdentityResult'
        | 'DeactivateIdentityResult'
        | 'ActivateIdentityResult'
        | 'SetIdentityNetworksResult',
    expectedIdGraph: [HeimaPrimitivesIdentity, boolean][]
): Promise<HexString> {
    const decodedResult = context.api.createType(resultType, returnValue.value);
    assert.isNotNull(decodedResult.mutated_id_graph);
    const idGraph = parseIdGraph(context.sidechainRegistry, decodedResult.mutated_id_graph, aesKey);
    assertIdGraph(idGraph, expectedIdGraph);
    const queriedIdGraphHash = (await getIdGraphHash(context, teeShieldingKey, identity)).toHex();
    assert.equal(u8aToHex(decodedResult.id_graph_hash), queriedIdGraphHash);

    console.log(colors.green('assertIdGraphMutationResult passed'));
    return u8aToHex(decodedResult.id_graph_hash);
}

export async function assertVc(context: IntegrationTestContext, subject: HeimaPrimitivesIdentity, data: Bytes) {
    const results = context.api.createType('RequestVCResult', data);
    // step 1
    // decryptWithAes function added 0x prefix
    const vcPayload = results.vc_payload;
    const decryptVcPayload = decryptWithAes(aesKey, vcPayload, 'utf-8').replace('0x', '');

    /* DID format
    did:litentry:substrate:0x12345...
    did:litentry:evm:0x123456...
    did:litentry:twitter:my_twitter_handle
    */

    // step 2
    // check credential subject's DID
    const credentialSubjectId = JSON.parse(decryptVcPayload).credentialSubject.id;
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
    const vcPayloadJson = JSON.parse(decryptVcPayload);
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

export async function assertIdGraphHash(
    context: IntegrationTestContext,
    teeShieldingKey: KeyObject,
    identity: HeimaPrimitivesIdentity,
    idGraph: [HeimaPrimitivesIdentity, PalletIdentityManagementTeeIdentityContext][]
) {
    const idGraphType = context.sidechainRegistry.createType(
        'Vec<(HeimaPrimitivesIdentity, PalletIdentityManagementTeeIdentityContext)>',
        idGraph
    );
    const computedIdGraphHash = blake2AsHex(idGraphType.toU8a());
    console.log('computed id graph hash: ', computedIdGraphHash);

    const queriedIdGraphHash = (await getIdGraphHash(context, teeShieldingKey, identity)).toHex();
    console.log('queried id graph hash: ', queriedIdGraphHash);
    assert.equal(computedIdGraphHash, queriedIdGraphHash);
}

function trimPrefix(str: string, prefix: string): string {
    if (str.startsWith(prefix)) {
        return str.substring(prefix.length);
    }
    return str;
}
