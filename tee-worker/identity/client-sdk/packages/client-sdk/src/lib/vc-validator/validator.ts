import { ApiPromise } from '@polkadot/api';
import { hexToU8a, stringToU8a } from '@polkadot/util';
import { base58Decode, signatureVerify } from '@polkadot/util-crypto';

import type { CorePrimitivesTeebagTypesEnclave } from '@litentry/parachain-api';

import { RUNTIME_ENCLAVE_REGISTRY } from './runtime-enclave-registry';

import type { ValidationResult } from './validator.types';

export type VerifiableCredentialLike = Record<string, unknown> & {
  '@context': string;
  type: Array<string>;
  id: string;
  issuer: {
    id: string;
    name: string;
    mrenclave: string;
  };
  proof: {
    proofValue: string;
    verificationMethod: string;
  };
  credentialSubject: Record<string, unknown>;
  parachainBlockNumber?: number;
  sidechainBlockNumber?: number;
};

/**
Validate Verifiable Credential (VC)

### How to find the wallet account address in VC

The id of VC's credentialSubject is the encoded wallet account address, using code below to encode:

```typescript
import { decodeAddress } from '@polkadot/util-crypto';
import { u8aToHex } from '@polkadot/util';

const address = u8aToHex(decodeAddress(walletAccountAddress));
const credentialSubjectId = address.substring(2);
```

With the code above:
- If your Polkadot account address is `5CwPfqmormPx9wJ4ASq7ikwdJeRonoUZX9SxwUtm1px9L72W`, the credentialSubjectId will be `26a84b380d8c3226d69f9ae6e482aa6669ed34c6371c52c4dfb48596913d6f28`.
- If your Metamask account address is `0xC620b3e5BEBedA952A8AD18b83Dc4Cf3Dc9CAF4b`, the credentialSubjectId will be `c620b3e5bebeda952a8ad18b83dc4cf3dc9caf4b`.

### What the validation function do

- The validation function can only verify that the VC was issued by Litentry.
- The VC's credentialSubject can be Substrate or EVM account that is support by Litentry.

### What the validation function can't do

- The validation function cannot validate that the VC's credentialSubject is the current wallet account. It's SDK's consumer's responsibility to validate the id of VC's credentialSubject is equal to the wallet address.

### How to use

```typescript
import { WsProvider, ApiPromise } from '@polkadot/api';
import { validateVc, NETWORKS } from '@litentry/vc-sdk';

const api: ApiPromise = await ApiPromise.create({
  provider: new WsProvider(NETWORKS['litentry-staging'])
});
const vcJson = '{"@context": "https://www.w3.org/2018/credentials/v1", "type": "VerifiableCredential", "issuer": "https://example.com/issuer", "subject": "did:example:123", "credentialStatus": "https://example.com/status"}';
const result = await validateVc(api, vcJson);

// isValid is false if any field value of the result.detail is not true
if (!result.isValid) {
  // true or error message
  console.log('vcJson: ', result.detail.vcJson);
  // true or error message
  console.log('vcRegistry: ', result.detail.vcRegistry);
  // true or error message
  console.log('vcSignature: ', result.detail.vcSignature);
  // true or error message
  console.log('enclaveRegistry: ', result.detail.enclaveRegistry);
}
```

@param {ApiPromise} api - The instance of ApiPromise.
@param {ApiPromise} vc - The VC JSON string that needs to be validated.
@returns The validation result.
 */
export async function validateVc(
  api: ApiPromise,
  vc: string
): Promise<ValidationResult> {
  // 1. Self-contained proof: Validate the VC signature
  const vcSignature = validateVcSignature(vc);

  // early exit
  if (vcSignature !== true) {
    return {
      isValid: false,
      detail: {
        vcSignature,
        enclaveRegistry: 'invalid',
      },
    };
  }

  const parsedVc = JSON.parse(vc) as VerifiableCredentialLike;

  // 2. Verify Enclave registry
  // 2-a Validate against tee-dev enclaves used in production.
  const isValid = validateVcWithTrustedTeeDevEnclave(api, parsedVc);

  if (isValid === true) {
    return {
      isValid: true,
      detail: {
        vcSignature: true,
        enclaveRegistry: true,
      },
    };
  }

  // 2-b Validate against the current running Enclave
  const [enclaveRegistry] = await Promise.all([
    validateEnclaveRegistry(api, parsedVc),
  ]);

  return {
    isValid: enclaveRegistry === true,
    detail: {
      vcSignature: true,
      enclaveRegistry,
    },
  };
}

function isDid(did: string): boolean {
  return did.startsWith('did:litentry:');
}

function parseLitentryIdentityDid(
  did: `did:litentry:${string}:${string}` | string
): { network: string; account: string } {
  if (typeof did !== 'string') {
    throw new Error('Invalid DID');
  }

  if (!did.startsWith('did:litentry:')) {
    throw new Error('Invalid DID');
  }

  const [network, account] = did.split(':').slice(2);

  if (!network || !account) {
    throw new Error('Invalid DID');
  }

  return {
    network,
    account,
  };
}

/**
 * Get the issuer account from a VerifiableCredential-like object.
 *
 * For compatibility, it supports both DID format and non-prefixed hex format of legacy VCs.
 *
 * @example
 * ```ts
 *  getIssuerAccount({
 *    issuer: {
 *      id: '776a9b30535f6f318cbb6151a454f13b34ad6028cedc212d553d5385528995b3',
 *      name: 'Litentry TEE Worker',
 *      mrenclave: 'DHz8RLnPJk5c5RoPqMjJKXMJ5rLu7EjZKZ6sRGCGc1sn',
 *    },
 *  });
 *
 * // returns '0x776a9b30535f6f318cbb6151a454f13b34ad6028cedc212d553d5385528995b3'
 *
 * ```ts
 * getIssuerAccount({
 *  issuer: {
 *      id: 'did:litentry:substrate:0xd9373befadaf40f4f974200edaa9751ded0ab22203746154927c720af38bcff9',
 *      name: 'Litentry TEE Worker',
 *      mrenclave: '2oCP63BREBwWUYaVrYcLuUZiDfDKRjVdCnqPHn8R9pY3',
 *    },
 *  });
 *
 * // returns '0xd9373befadaf40f4f974200edaa9751ded0ab22203746154927c720af38bcff9'
 *
 * @ignore
 */
export function getIssuerAccount(vc: VerifiableCredentialLike): `0x${string}` {
  const value = vc?.issuer?.id;

  if (typeof value !== 'string') {
    throw new Error('issuer id is missing');
  }

  // DID format introduced on Dec 2023
  if (isDid(value)) {
    return parseLitentryIdentityDid(value).account as `0x${string}`;
  }

  // non-prefixed hex format before Dec 2023
  return `0x${value}`;
}

function validateVcSignature(vcJson: string): true | string {
  try {
    const { proof, ...content } = JSON.parse(
      vcJson
    ) as VerifiableCredentialLike;

    const signature = hexToU8a(`0x${proof.proofValue}`);
    const message = stringToU8a(JSON.stringify(content));
    const vcPubkey = proof.verificationMethod;

    const { isValid } = signatureVerify(message, signature, vcPubkey);

    if (!isValid) {
      return 'vc signature is invalid';
    }
    return true;
  } catch (e) {
    return (e as Error).message;
  }
}

/**
 * Validates that the VC was valid at the time it was issued.
 */
export async function validateEnclaveRegistry(
  api: ApiPromise,
  vc: VerifiableCredentialLike
): Promise<true | string> {
  const issuerId = getIssuerAccount(vc);
  const vcPubkey = vc.proof.verificationMethod;

  // 1. For VCs considered old (*),  we position the api at end of june 2024
  // and assert the vc-pubkey and mrenclave info.
  //
  // (*) VC with no parachainBlockNumber field and VC where issuer.id is vc-pubkey
  //
  // @TODO position api at block height after P-892 gets released
  if (!vc.parachainBlockNumber || vc.proof.verificationMethod === issuerId) {
    const registry = await api.query.teebag.enclaveRegistry.entries();
    const enclave = registry.some(([, maybeEnclave]) => {
      if (maybeEnclave.isEmpty) return false;

      const entry = maybeEnclave.unwrap();

      return (
        entry.vcPubkey.eq(vcPubkey) &&
        entry.mrenclave.eq(base58Decode(vc.issuer.mrenclave))
      );
    });

    if (!enclave) {
      console.log(
        `[vc-sdk::validator] No Enclave Worker with vcPubkey "${vcPubkey}" and mrenclave "${vc.issuer.mrenclave}" found`
      );
      return 'enclave registry is invalid';
    }

    return true;
  }

  // 2. For any other VC, use the `parachainBlockNumber` and `issuer.id` to determine the enclave.
  const blockHash = await api.rpc.chain.getBlockHash(vc.parachainBlockNumber);
  const apiAtIssuedBlock = await api.at(blockHash);
  const maybeEnclave = await apiAtIssuedBlock.query.teebag.enclaveRegistry(
    issuerId
  );
  if (maybeEnclave.isEmpty) {
    console.log(`[vc-sdk::validator] No Enclave Worker with ${issuerId} found`);
    return 'enclave registry is invalid';
  }

  const enclave = maybeEnclave.unwrap();

  if (!enclave.vcPubkey.eq(vcPubkey)) {
    return `vc pubkey is invalid. Got ${vcPubkey}, expected ${enclave.vcPubkey.toHex()}.`;
  }

  if (!enclave.mrenclave.eq(base58Decode(vc.issuer.mrenclave))) {
    return 'mrenclave is invalid';
  }

  return true;
}

/**
 * Validates the VC information against the past enclave registry.
 * These Enclave may no longer be online but these are Enclave that issued the VC and we trust.
 *
 * exported for testing purposes.
 */
export function validateVcWithTrustedTeeDevEnclave(
  api: ApiPromise,
  vc: VerifiableCredentialLike
): true | string {
  const enclave: CorePrimitivesTeebagTypesEnclave = api.createType(
    'CorePrimitivesTeebagTypesEnclave',
    RUNTIME_ENCLAVE_REGISTRY
  );

  // 1. check mrenclave
  const vcMrEnclave = base58Decode(vc.issuer.mrenclave);
  const isMrEnclaveValid = enclave.mrenclave.eq(vcMrEnclave);

  if (!isMrEnclaveValid) {
    return 'mrenclave is invalid';
  }

  // 2. Check pubkey
  const vcPubkey = vc.proof.verificationMethod;
  const isVcPubkeyValid = enclave.vcPubkey.eq(vcPubkey);

  if (!isVcPubkeyValid) {
    return `vc pubkey is invalid. Got ${vcPubkey}, expected ${enclave.vcPubkey.toHex()}.`;
  }

  // it checks-up
  return true;
}
