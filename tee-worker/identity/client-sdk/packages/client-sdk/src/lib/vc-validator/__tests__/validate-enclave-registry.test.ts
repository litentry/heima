/**
 * Context: on 06.2024, issuer.id was fixed to be the enclave account and not the vc-pubkey
 *
 * The test should check issuer.id is validated as the enclave account and test that
 * old versioned Vc, where issuer.id was the vc-pubkey, work too
 */

import { ApiPromise } from '@polkadot/api';
import { Metadata, TypeRegistry } from '@polkadot/types';

import metadataRpc from '@litentry/parachain-api/prepare-build/litentry-parachain-metadata.json';

import {
  validateEnclaveRegistry,
  type VerifiableCredentialLike,
} from '../validator';

// VC where issuer.id is an enclave account. vc-pubkey is not the issuer.id
const VC_SAMPLE =
  '{"@context":["https://www.w3.org/2018/credentials/v1","https://w3id.org/security/suites/ed25519-2020/v1"],"id":"0xec36f40a79092bf0030a902daf35a293cceb8dcf52dcefb1ad1312b79efb61bc","type":["VerifiableCredentialLike"],"credentialSubject":{"id":"did:litentry:evm:0x0ace67628bd43213c1c41ca1daef47e63923c75c","description":"The amount of a particular token you are holding","type":"Token Holding Amount","assertionText":"TokenHoldingAmount(Eth)","assertions":[{"and":[{"src":"$token","op":"==","dst":"ETH"},{"or":[{"and":[{"src":"$network","op":"==","dst":"bsc"},{"src":"$address","op":"==","dst":"0x2170ed0880ac9a755fd29b2688956bd959f933f8"}]},{"and":[{"src":"$network","op":"==","dst":"ethereum"}]}]},{"src":"$holding_amount","op":">=","dst":"0.6"},{"src":"$holding_amount","op":"<","dst":"1.2"}]}],"values":[true],"endpoint":"http://localhost:9933"},"issuer":{"id":"did:litentry:substrate:0x0c575468882ece2b3f10c532a1acae8ad126cdb739d175949a85187d820cc5f3","name":"Litentry TEE Worker","mrenclave":"GNqMHe4bdwR4HSkBVTUQ9uMwP5fcpAnu1NGGw3wgrSrg","runtimeVersion":{"parachain":9181,"sidechain":107}},"issuanceDate":"2024-06-25T19:57:50.337888744+00:00","parachainBlockNumber":30,"sidechainBlockNumber":55,"proof":{"created":"2024-06-25T19:57:50.338101922+00:00","type":"Ed25519Signature2020","proofPurpose":"assertionMethod","proofValue":"5c770757f43ca977665f529d36b84c34c0fa15268673d49506814bf863cf88b4a0bbe927ca3cf72473e2eebf26167e8e0d8ed809e0a01f5f32f01bf8240b0707","verificationMethod":"0xbf31981dbb9673a3a0fd662ed716a2a172fa897202ad9b43ce914a6c73916c6b"},"credentialSchema":{"id":"https://raw.githubusercontent.com/litentry/vc-jsonschema/main/dist/schemas/25-token-holding-amount/1-1-2.json","type":"JsonSchemaValidator2018"}}';
const VC = JSON.parse(VC_SAMPLE) as VerifiableCredentialLike;

const registry = new TypeRegistry();
registry.register({});
const metadata = new Metadata(registry, metadataRpc.result as `0x${string}`);
registry.setMetadata(metadata);

const palletTeebagEnclaveJson = {
  workerType: 'Identity',
  mrenclave:
    '0x482b74e3ed4faab7675c54d72cca9a2e9016e0d0b02416553d470a5c81806077',
  lastSeenTimestamp: '1707504072000',
  url: 'wss://127.0.0.1:2130',
  shieldingPubkey:
    '{n:[141,96,206,167,106,208,16,177,54,166,114,140,239,178,72,146,59,119,66,206,243,148,12,52,93,2,116,190,194,69,118,224,88,155,161,41,135,49,32,122,215,193,214,67,121,161,52,23,62,146,172,6,167,154,67,128,42,170,12,52,138,216,42,148,59,251,253,45,105,255,125,139,249,40,234,41,214,81,206,22,116,148,15,200,158,92,135,142,168,59,48,66,86,199,204,253,115,132,67,123,26,88,82,157,176,240,239,106,200,97,13,200,226,130,222,8,26,131,222,155,128,36,212,162,152,18,117,98,194,210,147,213,243,193,154,155,223,146,116,66,201,154,5,35,1,173,42,11,67,15,124,96,108,137,82,42,28,37,71,151,83,6,19,197,137,165,238,227,251,25,93,144,100,240,14,205,93,50,105,227,193,191,179,69,88,174,53,109,122,87,210,166,69,173,5,193,210,95,22,17,79,187,3,165,193,195,238,36,181,206,197,255,29,172,175,64,41,73,91,158,136,105,226,101,20,196,173,158,187,132,221,204,253,65,238,59,93,204,93,178,123,57,16,172,225,37,127,50,226,52,242,215,204,21,67,179,56,136,126,189,56,20,13,163,54,199,47,215,161,196,56,79,152,154,95,52,44,206,80,222,140,229,139,229,255,199,79,137,240,212,237,235,29,181,153,89,183,20,68,168,2,26,18,135,69,205,32,176,13,10,33,185,180,32,129,169,102,250,62,146,255,61,86,255,207,238,139,198,161,170,239,57,69,209,92,220,52,178,36,4,180,116,51,177,226,228,160,112,198,246,122,190,96,37,148,72,120,126,90,135,136,52,25,120,188,156,52,24,222,219,181,236,54,158,204,119,158,21,234,21,239,22,155,130],e:[1,0,0,1]}',
  vcPubkey:
    '0xbf31981dbb9673a3a0fd662ed716a2a172fa897202ad9b43ce914a6c73916c6b',
  sgxBuildMode: 'Production',
  attestationType: 'Ignore',
};

let api: ApiPromise;

beforeAll(() => {
  const registry = new TypeRegistry();
  // PalletTeeBag is in the chain types metadata
  const metadata = new Metadata(registry, metadataRpc.result as `0x${string}`);
  registry.setMetadata(metadata);

  const optionEnclave = registry.createType(
    'Option<CorePrimitivesTeebagTypesEnclave>',
    registry.createType(
      'CorePrimitivesTeebagTypesEnclave',
      palletTeebagEnclaveJson
    )
  );

  // define api manually to mock query methods
  api = {
    at: jest.fn(() => Promise.resolve(api)),
    createType: registry.createType.bind(registry),
    query: {
      system: {
        blockHash: jest.fn(() => Promise.resolve('0x1234')),
      },
      teebag: {
        enclaveRegistry: jest.fn(() => optionEnclave),
        scheduledEnclave: {
          entries: jest.fn(() => []),
        },
      },
    },
    rpc: {
      chain: {
        getBlockHash: jest.fn(
          () =>
            '0x8a9bc5971c3a6d23ba4743134c0caf12dca775a872fea5c27516175142bd4e1b'
        ),
      },
    },
  } as unknown as ApiPromise;

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  api.query.teebag.enclaveRegistry.entries = jest.fn(() => [
    [null, optionEnclave],
  ]);
});

describe('[regular VC] `issuer.id` is the enclave account', () => {
  it('issuer.id and enclave account should not be the same', () => {
    expect(`did:litentry:substrate:${VC.proof.verificationMethod}`).not.toEqual(
      VC.issuer.id
    );
  });

  it('should call query.teebag.enclaveRegistry(enclaveAccount)', async () => {
    const result = await validateEnclaveRegistry(api, VC);

    expect(result).toBeTruthy();

    expect(api.rpc.chain.getBlockHash).toHaveBeenCalledWith(
      VC.parachainBlockNumber
    );
    expect(api.at).toHaveBeenCalled();

    expect(api.query.teebag.enclaveRegistry).toHaveBeenCalledWith(
      VC.issuer.id.substring('did:litentry:substrate:'.length)
    );
  });
});

describe('[old VC] `issuer.id` is vc-pubkey', () => {
  let VC_OLD: VerifiableCredentialLike;

  beforeAll(() => {
    const VC_SAMPLE_OLD = JSON.stringify(
      Object.assign({}, VC, {
        issuer: {
          ...VC.issuer,
          id: `did:litentry:substrate:${VC.proof.verificationMethod}`,
        },
      })
    );

    VC_OLD = JSON.parse(VC_SAMPLE_OLD) as VerifiableCredentialLike;
  });

  it('issuer.id and enclave account are the same', () => {
    expect(`did:litentry:substrate:${VC_OLD.proof.verificationMethod}`).toEqual(
      VC_OLD.issuer.id
    );
  });

  it('should call query.teebag.enclaveRegistry()', async () => {
    const result = await validateEnclaveRegistry(api, VC_OLD);

    expect(result).toBeTruthy();

    expect(api.query.teebag.enclaveRegistry).not.toHaveBeenCalledWith(
      VC_OLD.issuer.id
    );
    expect(api.query.teebag.enclaveRegistry.entries).toHaveBeenCalled();
  });
});
