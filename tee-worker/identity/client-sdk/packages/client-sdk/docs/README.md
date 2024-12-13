@litentry/client-sdk

# @litentry/client-sdk

## Table of contents

### References

- [getIdGraphHash](README.md#getidgraphhash)

### Namespaces

- [request](modules/request.md)

### Classes

- [Enclave](classes/Enclave.md)

### Type Aliases

- [DiscordOAuth2Proof](README.md#discordoauth2proof)
- [DiscordProof](README.md#discordproof)
- [EmailProof](README.md#emailproof)
- [IdGraph](README.md#idgraph)
- [TwitterOAuth2Proof](README.md#twitteroauth2proof)
- [TwitterProof](README.md#twitterproof)
- [ValidationResult](README.md#validationresult)
- [ValidationResultDetail](README.md#validationresultdetail)
- [VerifiableCredentialLike](README.md#verifiablecredentiallike)
- [Web3Proof](README.md#web3proof)

### Variables

- [ID\_GRAPH\_STRUCT](README.md#id_graph_struct)
- [enclave](README.md#enclave)

### Functions

- [calculateIdGraphHash](README.md#calculateidgraphhash)
- [createKeyAesOutputType](README.md#createkeyaesoutputtype)
- [createLitentryIdentityType](README.md#createlitentryidentitytype)
- [createLitentryValidationDataType](README.md#createlitentryvalidationdatatype)
- [createRequestType](README.md#createrequesttype)
- [createTCAuthenticationType](README.md#createtcauthenticationtype)
- [createTrustedCallType](README.md#createtrustedcalltype)
- [toPublicKey](README.md#topublickey)
- [validateVc](README.md#validatevc)

## References

### getIdGraphHash

Re-exports [getIdGraphHash](modules/request.md#getidgraphhash)

## Type Aliases

### DiscordOAuth2Proof

Ƭ **DiscordOAuth2Proof**: `Object`

Ownership proof for Discord accounts using oAuth2

**`See`**

createLitentryValidationDataType

#### Type declaration

| Name | Type |
| :------ | :------ |
| `code` | `string` |
| `redirectUri` | `string` |

#### Defined in

[lib/type-creators/validation-data.ts:58](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/validation-data.ts#L58)

___

### DiscordProof

Ƭ **DiscordProof**: `Object`

Ownership proof for Discord accounts

**`See`**

createLitentryValidationDataType

#### Type declaration

| Name | Type |
| :------ | :------ |
| `channelId` | `string` |
| `guildId` | `string` |
| `messageId` | `string` |

#### Defined in

[lib/type-creators/validation-data.ts:48](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/validation-data.ts#L48)

___

### EmailProof

Ƭ **EmailProof**: `Object`

Ownership proof for Email

**`See`**

createLitentryValidationDataType

#### Type declaration

| Name | Type |
| :------ | :------ |
| `email` | `string` |
| `verificationCode` | `string` |

#### Defined in

[lib/type-creators/validation-data.ts:68](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/validation-data.ts#L68)

___

### IdGraph

Ƭ **IdGraph**: `Vec`\<`ITuple`\<[`LitentryIdentity`, `IdentityContext`]\>\>

The Identity Graph type

#### Defined in

[lib/type-creators/id-graph.ts:13](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/id-graph.ts#L13)

___

### TwitterOAuth2Proof

Ƭ **TwitterOAuth2Proof**: `Object`

Ownership proof for Twitter accounts using oAuth2

**`See`**

createLitentryValidationDataType

#### Type declaration

| Name | Type |
| :------ | :------ |
| `code` | `string` |
| `redirectUri` | `string` |
| `state` | `string` |

#### Defined in

[lib/type-creators/validation-data.ts:37](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/validation-data.ts#L37)

___

### TwitterProof

Ƭ **TwitterProof**: `Object`

Ownership proof for Twitter accounts

**`See`**

createLitentryValidationDataType

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tweetId` | `string` |

#### Defined in

[lib/type-creators/validation-data.ts:30](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/validation-data.ts#L30)

___

### ValidationResult

Ƭ **ValidationResult**: `Object`

Represents the overall validation result for a Verifiable Credential (VC).

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `detail` | [`ValidationResultDetail`](README.md#validationresultdetail) | Represents the whole validation result detail. |
| `isValid` | `boolean` | Represents the whole validation result status. If is true, means all fields of the detail are true, otherwise any one of it is not true. The caller should use this field to determine whether the VC is valid. |

#### Defined in

[lib/vc-validator/validator.types.ts:28](https://github.com/litentry/client-sdk/blob/develop/lib/vc-validator/validator.types.ts#L28)

___

### ValidationResultDetail

Ƭ **ValidationResultDetail**: `Object`

Defines the details of the validation result for each component of the Verifiable Credential (VC).

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `enclaveRegistry?` | ``true`` \| `string` | Represents the validation result (vcPubkey and mrEnclave) for the Enclave registry. If validation succeeds, it's true; if validation fails, it's an error message. The vcPubkey from Enclave registry must be same as issuer.id in VC JSON. The mrEnclave from Enclave registry must be same as issuer.mrenclave in VC JSON. |
| `vcSignature?` | ``true`` \| `string` | Represents the validation result for the VC signature. If validation succeeds, it's true; if validation fails, it's an error message. Use issuer.id in VC JSON as vcPubkey, proof.proofValue in VC JSON as signature to verify VC JSON. |

#### Defined in

[lib/vc-validator/validator.types.ts:4](https://github.com/litentry/client-sdk/blob/develop/lib/vc-validator/validator.types.ts#L4)

___

### VerifiableCredentialLike

Ƭ **VerifiableCredentialLike**: `Record`\<`string`, `unknown`\> & \{ `@context`: `string` ; `credentialSubject`: `Record`\<`string`, `unknown`\> ; `id`: `string` ; `issuer`: \{ `id`: `string` ; `mrenclave`: `string` ; `name`: `string`  } ; `parachainBlockNumber?`: `number` ; `proof`: \{ `proofValue`: `string` ; `verificationMethod`: `string`  } ; `sidechainBlockNumber?`: `number` ; `type`: `string`[]  }

#### Defined in

[lib/vc-validator/validator.ts:11](https://github.com/litentry/client-sdk/blob/develop/lib/vc-validator/validator.ts#L11)

___

### Web3Proof

Ƭ **Web3Proof**: `Object`

Ownership proof for Web3 accounts (Substrate, EVM, Bitcoin).

Bitcoin signatures are base64-encoded strings. Substrate and EVM signatures are hex-encoded strings.

**`See`**

createLitentryIdentityType

#### Type declaration

| Name | Type |
| :------ | :------ |
| `message` | `string` |
| `signature` | \`0x$\{string}\` \| `string` |

#### Defined in

[lib/type-creators/validation-data.ts:20](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/validation-data.ts#L20)

## Variables

### ID\_GRAPH\_STRUCT

• `Const` **ID\_GRAPH\_STRUCT**: ``"Vec<(LitentryIdentity, IdentityContext)>"``

The Type Struct that represents an Identity Graph

#### Defined in

[lib/type-creators/id-graph.ts:18](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/id-graph.ts#L18)

___

### enclave

• `Const` **enclave**: [`Enclave`](classes/Enclave.md)

Enclave instance

This an instance of the Enclave class.

**`See`**

Enclave

**`Example`**

```ts
import { enclave } from '@litentry/identity-hub';

const shard = await enclave.getShard(api);
const key = await enclave.getKey(api);
const encrypted = await enclave.encrypt(api, { cleartext: new Uint8Array([1, 2, 3]) });
const response = await enclave.send({
 jsonrpc: '2.0',
 method: 'author_submitAndWatch',
 params: ['0x123']
});
```

#### Defined in

[lib/enclave.ts:294](https://github.com/litentry/client-sdk/blob/develop/lib/enclave.ts#L294)

## Functions

### calculateIdGraphHash

▸ **calculateIdGraphHash**(`idGraph`): \`0x$\{string}\`

Returns the hash of the given id graph. It matches the hash used in the Litentry Parachain.

#### Parameters

| Name | Type |
| :------ | :------ |
| `idGraph` | [`IdGraph`](README.md#idgraph) |

#### Returns

\`0x$\{string}\`

#### Defined in

[lib/util/calculate-id-graph-hash.ts:9](https://github.com/litentry/client-sdk/blob/develop/lib/util/calculate-id-graph-hash.ts#L9)

___

### createKeyAesOutputType

▸ **createKeyAesOutputType**(`registry`, `data`): `AesOutput`

Creates a KeyAesOutput sidechain type.

Heads-up: ensure data.ciphertext is in hex format. Using Uint may cause a bytes out range error.

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | \`0x$\{string}\` \| \{ `aad`: \`0x$\{string}\` \| `Uint8Array` ; `ciphertext`: \`0x$\{string}\` ; `nonce`: \`0x$\{string}\` \| `Uint8Array`  } |

#### Returns

`AesOutput`

**`Example`**

build from object
```ts
const identity = createKeyAesOutputType(registry, {
 ciphertext: '0x...',
 nonce: '0x...',
 aad: '0x...',
});
```

build from hex string
```ts
const identity = createKeyAesOutputType(registry, `0x...`);
```

#### Defined in

[lib/type-creators/key-aes-output.ts:27](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/key-aes-output.ts#L27)

___

### createLitentryIdentityType

▸ **createLitentryIdentityType**(`registry`, `data`): `LitentryIdentity`

Creates a LitentryIdentity chain type.

Notice that addresses and handles are not fully validated. This struct shouldn't be relied on for validation.

For Bitcoin, the compressed public key is expected (begins with 02 or 03).

For Solana, the address string is expected to be a base58-encoded or hex-encoded string.

For Substrate, the address is expected to be a SS58-encoded or hex-encoded address.

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | \`0x$\{string}\` \| `Uint8Array` \| \{ `addressOrHandle`: `string` ; `type`: ``"Solana"`` \| ``"Twitter"`` \| ``"Discord"`` \| ``"Github"`` \| ``"Substrate"`` \| ``"Evm"`` \| ``"Bitcoin"`` \| ``"Email"``  } |

#### Returns

`LitentryIdentity`

**`Example`**

```ts
const substrateIdentity = createLitentryIdentityType(registry, {
 addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
 type: 'Substrate',
});

const twitterIdentity = createLitentryIdentityType(registry, {
 addressOrHandle: 'my-twitter-handle',
 type: 'Twitter',
});
```

#### Defined in

[lib/type-creators/litentry-identity.ts:31](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/litentry-identity.ts#L31)

___

### createLitentryValidationDataType

▸ **createLitentryValidationDataType**\<`IIdentityType`\>(`registry`, `identityDescriptor`, `proof`): `LitentryValidationData`

Creates the LitentryValidationData given the identity network and its type.

The proof to pass depends on the identity network (IIdentityType):
- Web3: Web3Proof
- Twitter: TwitterProof
- Discord: DiscordProof

#### Type parameters

| Name | Type |
| :------ | :------ |
| `IIdentityType` | extends ``"Solana"`` \| ``"Twitter"`` \| ``"Discord"`` \| ``"Github"`` \| ``"Substrate"`` \| ``"Evm"`` \| ``"Bitcoin"`` \| ``"Email"`` |

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `registry` | `Registry` | Litentry Parachain API's type registry |
| `identityDescriptor` | `Object` | - |
| `identityDescriptor.addressOrHandle` | `string` | The address or handle of the identity |
| `identityDescriptor.type` | `IIdentityType` | The identity type |
| `proof` | `IIdentityType` extends ``"Discord"`` ? [`DiscordProof`](README.md#discordproof) \| [`DiscordOAuth2Proof`](README.md#discordoauth2proof) : `IIdentityType` extends ``"Twitter"`` ? [`TwitterProof`](README.md#twitterproof) \| [`TwitterOAuth2Proof`](README.md#twitteroauth2proof) : `IIdentityType` extends ``"Email"`` ? [`EmailProof`](README.md#emailproof) : [`Web3Proof`](README.md#web3proof) | The ownership proof |

#### Returns

`LitentryValidationData`

**`Example`**

Web3
```ts
import { createLitentryValidationDataType } from '@litentry/client-sdk';
import type { Web3Proof } from '@litentry/client-sdk';

const userAddress = '0x123';

const proof: Web3Proof = {
  signature: '0x123',
  message: '0x123',
}

const validationData = createLitentryValidationDataType(
  registry,
  {
    addressOrHandle: userAddress,
    type: 'Evm',
  },
  proof,
);
```

**`Example`**

Twitter
```ts
import { createLitentryValidationDataType } from '@litentry/client-sdk';
import type { TwitterProof } from '@litentry/client-sdk';

const userHandle = '@litentry';

const proof: TwitterProof = {
  // Both twitter.com and x.com are valid
  tweetId: 'https://twitter.com/0x123/status/123',
};

const validationData = createLitentryValidationDataType(
  registry,
  {
    addressOrHandle: userHandle,
    type: 'Twitter',
  },
  proof,
);
```

#### Defined in

[lib/type-creators/validation-data.ts:126](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/validation-data.ts#L126)

___

### createRequestType

▸ **createRequestType**(`api`, `data`): `Promise`\<`AesRequest`\>

Creates a Request struct type for the `TrustedCall` operation.

A shielding key is generated and used to encrypt the `TrustedCall` operation and communicated
to the enclave to protect the data for transportation.

The shielding key is encrypted using the Enclave's shielding key and attached in the Request.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | - |
| `data` | `Object` | - |
| `data.authentication` | `string` | signature or verification code based on the identity type |
| `data.call` | `TrustedCall` | - |
| `data.nonce` | `Index` | - |
| `data.sender` | `LitentryIdentity` | - |
| `data.shard` | `Uint8Array` | - |

#### Returns

`Promise`\<`AesRequest`\>

#### Defined in

[lib/type-creators/request.ts:32](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/request.ts#L32)

___

### createTCAuthenticationType

▸ **createTCAuthenticationType**(`registry`, `data`): `TCAuthentication`

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `AuthenticationData` |

#### Returns

`TCAuthentication`

#### Defined in

[lib/type-creators/tc-authentication.ts:16](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/tc-authentication.ts#L16)

___

### createTrustedCallType

▸ **createTrustedCallType**(`registry`, `data`): `Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

Creates the TrustedCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the TrustedCall

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `Object` |
| `data.method` | ``"link_identity"`` |
| `data.params` | `LinkIdentityParams` |

#### Returns

`Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

**`See`**

 - https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/identity/app-libs/stf/src/trusted_call.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/identity/client-api/parachain-api/prepare-build/interfaces/trusted_operations/definitions.ts

#### Defined in

[lib/type-creators/trusted-call.ts:90](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/trusted-call.ts#L90)

▸ **createTrustedCallType**(`registry`, `data`): `Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `Object` |
| `data.method` | ``"set_identity_networks"`` |
| `data.params` | `SetIdentityNetworksParams` |

#### Returns

`Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Defined in

[lib/type-creators/trusted-call.ts:97](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/trusted-call.ts#L97)

▸ **createTrustedCallType**(`registry`, `data`): `Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `Object` |
| `data.method` | ``"request_vc"`` |
| `data.params` | `RequestVcParams` |

#### Returns

`Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Defined in

[lib/type-creators/trusted-call.ts:104](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/trusted-call.ts#L104)

▸ **createTrustedCallType**(`registry`, `data`): `Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `Object` |
| `data.method` | ``"request_batch_vc"`` |
| `data.params` | `RequestBatchVcParams` |

#### Returns

`Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Defined in

[lib/type-creators/trusted-call.ts:111](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/trusted-call.ts#L111)

▸ **createTrustedCallType**(`registry`, `data`): `Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `Object` |
| `data.method` | ``"request_intent"`` |
| `data.params` | `RequestIntentParams` |

#### Returns

`Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Defined in

[lib/type-creators/trusted-call.ts:118](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/trusted-call.ts#L118)

▸ **createTrustedCallType**(`registry`, `data`): `Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `Object` |
| `data.method` | ``"create_account_store"`` |
| `data.params` | `RequestCreateAccountStoreParams` |

#### Returns

`Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Defined in

[lib/type-creators/trusted-call.ts:125](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/trusted-call.ts#L125)

▸ **createTrustedCallType**(`registry`, `data`): `Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `registry` | `Registry` |
| `data` | `Object` |
| `data.method` | ``"link_identity_callback"`` |
| `data.params` | `LinkIdentityCallbackParams` |

#### Returns

`Promise`\<\{ `call`: `TrustedCall` ; `key`: `CryptoKey`  }\>

#### Defined in

[lib/type-creators/trusted-call.ts:132](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/trusted-call.ts#L132)

___

### toPublicKey

▸ **toPublicKey**(`address`): \`0x$\{string}\`

Returns the public key from a given substrate address.

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

\`0x$\{string}\`

**`Throws`**

if the address is not a valid substrate address

#### Defined in

[lib/type-creators/litentry-identity.ts:85](https://github.com/litentry/client-sdk/blob/develop/lib/type-creators/litentry-identity.ts#L85)

___

### validateVc

▸ **validateVc**(`api`, `vc`): `Promise`\<[`ValidationResult`](README.md#validationresult)\>

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

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | The instance of ApiPromise. |
| `vc` | `string` | The VC JSON string that needs to be validated. |

#### Returns

`Promise`\<[`ValidationResult`](README.md#validationresult)\>

The validation result.

#### Defined in

[lib/vc-validator/validator.ts:86](https://github.com/litentry/client-sdk/blob/develop/lib/vc-validator/validator.ts#L86)
