import { ApiPromise } from '@polkadot/api';
import { TypeRegistry, Metadata } from '@polkadot/types';

import metadataRpc from '@litentry/parachain-api/prepare-build/litentry-parachain-metadata.json';

import {
  getIssuerAccount,
  validateVc,
  validateVcWithTrustedTeeDevEnclave,
  type VerifiableCredentialLike,
} from './validator';

const registry = new TypeRegistry();
// PalletTeeBag is in the chain types metadata
const metadata = new Metadata(registry, metadataRpc.result as `0x${string}`);
registry.setMetadata(metadata);

// Tests will be run against the following constants
const VC_STRING = `{"@context":["https://www.w3.org/2018/credentials/v1","https://w3id.org/security/suites/ed25519-2020/v1"],"id":"0xd9add088c9c601a3a5ada44aecd0652f3f7ade535415ff38b87ad4772aac9c35","type":["VerifiableCredential"],"credentialSubject":{"id":"did:litentry:substrate:0x32e8dde674d49c60c766f350e65bef59be60df6b5bbd82d212d74f72483a2d58","description":"The user is an early bird user of the IdentityHub EVM version and has generated at least 1 credential during 2023 Aug 14th ~ Aug 21st.","type":"IDHub EVM Version Early Bird","assertionText":"A20","assertions":[{"src":"$has_joined","op":"==","dst":"false"}],"values":[false],"endpoint":"http://localhost:9933"},"issuer":{"id":"did:litentry:substrate:0x679b46ff616adb70536f240c339a3f18d529600e3032057363e06d3bb259311c","name":"Litentry TEE Worker","mrenclave":"5ring5YbtScEBX9G379E1GGArrUa7aDAcgpuhBthDyyg"},"issuanceDate":"2024-02-09T21:35:00.092920175+00:00","parachainBlockNumber":874,"sidechainBlockNumber":1738,"proof":{"created":"2024-02-09T21:35:00.093431770+00:00","type":"Ed25519Signature2020","proofPurpose":"assertionMethod","proofValue":"955cc41d4eba19742e4c1767277434695b79f88a3881fc2cf4e58cf19ad793b8f4209902c041366fe1118c84a3f255f51cb893335d92291bfcec852efd37b408","verificationMethod":"0x679b46ff616adb70536f240c339a3f18d529600e3032057363e06d3bb259311c"}}`;

const palletTeebagEnclaveJson = {
  workerType: 'Identity',
  mrenclave:
    '0x482b74e3ed4faab7675c54d72cca9a2e9016e0d0b02416553d470a5c81806077',
  lastSeenTimestamp: '1707504072000',
  url: 'wss://127.0.0.1:2130',
  shieldingPubkey:
    '{n:[141,96,206,167,106,208,16,177,54,166,114,140,239,178,72,146,59,119,66,206,243,148,12,52,93,2,116,190,194,69,118,224,88,155,161,41,135,49,32,122,215,193,214,67,121,161,52,23,62,146,172,6,167,154,67,128,42,170,12,52,138,216,42,148,59,251,253,45,105,255,125,139,249,40,234,41,214,81,206,22,116,148,15,200,158,92,135,142,168,59,48,66,86,199,204,253,115,132,67,123,26,88,82,157,176,240,239,106,200,97,13,200,226,130,222,8,26,131,222,155,128,36,212,162,152,18,117,98,194,210,147,213,243,193,154,155,223,146,116,66,201,154,5,35,1,173,42,11,67,15,124,96,108,137,82,42,28,37,71,151,83,6,19,197,137,165,238,227,251,25,93,144,100,240,14,205,93,50,105,227,193,191,179,69,88,174,53,109,122,87,210,166,69,173,5,193,210,95,22,17,79,187,3,165,193,195,238,36,181,206,197,255,29,172,175,64,41,73,91,158,136,105,226,101,20,196,173,158,187,132,221,204,253,65,238,59,93,204,93,178,123,57,16,172,225,37,127,50,226,52,242,215,204,21,67,179,56,136,126,189,56,20,13,163,54,199,47,215,161,196,56,79,152,154,95,52,44,206,80,222,140,229,139,229,255,199,79,137,240,212,237,235,29,181,153,89,183,20,68,168,2,26,18,135,69,205,32,176,13,10,33,185,180,32,129,169,102,250,62,146,255,61,86,255,207,238,139,198,161,170,239,57,69,209,92,220,52,178,36,4,180,116,51,177,226,228,160,112,198,246,122,190,96,37,148,72,120,126,90,135,136,52,25,120,188,156,52,24,222,219,181,236,54,158,204,119,158,21,234,21,239,22,155,130],e:[1,0,0,1]}',
  vcPubkey:
    '0x679b46ff616adb70536f240c339a3f18d529600e3032057363e06d3bb259311c',
  sgxBuildMode: 'Production',
  attestationType: 'Ignore',
};

let api: ApiPromise;

beforeAll(() => {
  const optionEnclave = registry.createType(
    'Option<CorePrimitivesTeebagTypesEnclave>',
    registry.createType(
      'CorePrimitivesTeebagTypesEnclave',
      palletTeebagEnclaveJson
    )
  );

  // define api manually to mock query methods
  api = {
    createType: registry.createType.bind(registry),
    query: {
      teebag: {
        enclaveRegistry: jest.fn(() => optionEnclave),
        scheduledEnclave: {
          entries: jest.fn(() => []),
        },
      },
    },
  } as unknown as ApiPromise;

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  api.query.teebag.enclaveRegistry.entries = jest.fn(() => [
    [null, optionEnclave],
  ]);
});

describe('VcValidator', () => {
  const vc = JSON.parse(VC_STRING);

  it('should work', async () => {
    const result = await validateVc(api, VC_STRING);

    expect(result).toEqual({
      isValid: true,
      detail: {
        vcSignature: true,
        enclaveRegistry: true,
      },
    });
    expect(api.query.teebag.enclaveRegistry.entries).toHaveBeenCalled();
    expect(api.query.teebag.scheduledEnclave.entries).not.toHaveBeenCalled();
  });

  it('should parse issuer id from did', () => {
    // prior to dec 2023
    expect(
      getIssuerAccount({
        issuer: {
          id: '776a9b30535f6f318cbb6151a454f13b34ad6028cedc212d553d5385528995b3',
          name: 'Litentry TEE Worker',
          mrenclave: 'DHz8RLnPJk5c5RoPqMjJKXMJ5rLu7EjZKZ6sRGCGc1sn',
        },
      } as VerifiableCredentialLike)
    ).toStrictEqual(
      '0x776a9b30535f6f318cbb6151a454f13b34ad6028cedc212d553d5385528995b3'
    );

    // dec 2023 onwards
    expect(
      getIssuerAccount({
        issuer: {
          id: 'did:litentry:substrate:0xd9373befadaf40f4f974200edaa9751ded0ab22203746154927c720af38bcff9',
          name: 'Litentry TEE Worker',
          mrenclave: '2oCP63BREBwWUYaVrYcLuUZiDfDKRjVdCnqPHn8R9pY3',
        },
      } as VerifiableCredentialLike)
    ).toStrictEqual(
      '0xd9373befadaf40f4f974200edaa9751ded0ab22203746154927c720af38bcff9'
    );
  });

  it('should validate vc content legitimacy using contained proof', async () => {
    const result = await validateVc(
      api,
      JSON.stringify({
        ...vc,
        proof: {
          ...vc.proof,
          // mess-up proof value
          proofValue:
            '94dc5db7befa8e1ff7288d97dc0d9eec86257de327a701bc9e85d6caf4a9c97a8cebd3eb0cc9cd74395fd89d24a5eccc735ddc473fbd651711b426c4a2f99f07',
        },
      })
    );
    expect(result.isValid).toBeFalsy();
    expect(result.detail.vcSignature).not.toEqual(true);
    expect(result.detail.enclaveRegistry).toEqual('invalid');
  });

  it('should validate legitimacy of issuer signing key', async () => {
    (
      api.query['teebag']['enclaveRegistry']['entries'] as unknown as jest.Mock
    ).mockImplementationOnce(() => {
      const enclave = registry.createType('CorePrimitivesTeebagTypesEnclave', {
        ...palletTeebagEnclaveJson,
        // mess-up vcPubkey and mrEnclave
        vcPubkey:
          '0xbf6ae6b420f26aae8717dcd7ccc7d0caf543a27dbd5f622ea666e2eff6d1ec78',
        mrEnclave:
          '0xf23ef3036a0725ea502450753edb46a18636de8cac4cafc2d31e00af225d0c65',
      });

      const returnedValue = registry.createType(
        'Option<CorePrimitivesTeebagTypesEnclave>',
        enclave
      );

      return [[null, returnedValue]];
    });
    const result = await validateVc(api, VC_STRING);

    expect(result.isValid).toBeFalsy();
    expect(result.detail.vcSignature).toEqual(true);
    expect(typeof result.detail.enclaveRegistry).toEqual('string');
    expect(result.detail.enclaveRegistry?.toString()).toEqual(
      'enclave registry is invalid'
    );
  });
});

describe('validateVcWithTrustedTeeDevEnclave', () => {
  it('should not work with VC not issued by trusted past enclave', async () => {
    const result = await validateVcWithTrustedTeeDevEnclave(
      api,
      JSON.parse(VC_STRING)
    );

    expect(result).toEqual('mrenclave is invalid');
  });

  it('should work with VC issued by trusted past enclave', async () => {
    // this vc was generated on the running tee-dev from 2024-04-22 (trusted)
    const vcContent = `{"@context":["https://www.w3.org/2018/credentials/v1","https://w3id.org/security/suites/ed25519-2020/v1"],"id":"0x1aefd704d62ce216cc2147bb34f4af7637074d6770c95bb8d6a412ccd91ea63a","type":["VerifiableCredential"],"credentialSubject":{"id":"did:litentry:evm:0x0ace67628bd43213c1c41ca1daef47e63923c75c","description":"You are VIP3 Gold Card Holder","type":"VIP3 Gold Card Holder","assertionText":"VIP3MembershipCard(Gold)","assertions":[{"src":"$is_gold_card","op":"==","dst":"true"}],"values":[false],"endpoint":"wss://rpc.litentry-parachain.litentry.io"},"issuer":{"id":"did:litentry:substrate:0x56eb57c05914da669e494299476bb21b377f231951d6f9a846cde2c4d078eb3a","name":"Litentry TEE Worker","mrenclave":"6wxPcTxnt52hLJd8VxAuLJi5az5NJiMXBcaJdW6qvTJS"},"issuanceDate":"2024-04-22T13:35:28.678115066+00:00","parachainBlockNumber":1076,"sidechainBlockNumber":2154,"proof":{"created":"2024-04-22T13:35:28.678688591+00:00","type":"Ed25519Signature2020","proofPurpose":"assertionMethod","proofValue":"ad4bfe8a9ee8f29f08ed312fd02b6ffd05f97fbd8f09991356377d83f81b6ea0f19d9dd1160d8c2f28be66575b9b39e973b8c587b24fbef79531af416f76530e","verificationMethod":"0x56eb57c05914da669e494299476bb21b377f231951d6f9a846cde2c4d078eb3a"},"credentialSchema":{"id":"https://raw.githubusercontent.com/litentry/vc-jsonschema/main/dist/schemas/19-vip3-card-holder/1-0-0.json","type":"JsonSchemaValidator2018"}}`;

    const result = await validateVcWithTrustedTeeDevEnclave(
      api,
      JSON.parse(vcContent)
    );

    expect(result).toEqual(true);
  });
});
