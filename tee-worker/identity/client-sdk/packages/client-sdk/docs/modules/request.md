[@litentry/client-sdk](../README.md) / request

# Namespace: request

requests

## Table of contents

### Functions

- [addAccount](request.md#addaccount)
- [callEthereum](request.md#callethereum)
- [createAccountStore](request.md#createaccountstore)
- [createChallengeCode](request.md#createchallengecode)
- [getIdGraph](request.md#getidgraph)
- [getIdGraphHash](request.md#getidgraphhash)
- [getLastRegisteredEnclave](request.md#getlastregisteredenclave)
- [getOmniGraph](request.md#getomnigraph)
- [linkIdentity](request.md#linkidentity)
- [linkIdentityCallback](request.md#linkidentitycallback)
- [publicizeAccount](request.md#publicizeaccount)
- [remark](request.md#remark)
- [removeAccounts](request.md#removeaccounts)
- [requestAuthToken](request.md#requestauthtoken)
- [requestBatchVC](request.md#requestbatchvc)
- [requestVerificationCode](request.md#requestverificationcode)
- [setIdentityNetworks](request.md#setidentitynetworks)
- [transferEthereum](request.md#transferethereum)
- [transferNative](request.md#transfernative)
- [transferSolana](request.md#transfersolana)

## Functions

### addAccount

▸ **addAccount**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Adds an account to the Litentry Parachain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.identity` | `LitentryIdentity` | Account to be added. Use `createCorePrimitivesIdentityType` helper to create this struct |
| `data.isPublic` | `boolean` | Whether the account is public |
| `data.validation` | `LitentryValidationData` | The ownership proof. Use `createLitentryValidationDataType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to sign (if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is
an email identity, this is the email verification code. If who is not an email identity, this is the
signed payload.

#### Defined in

[lib/requests/add-account.request.ts:32](https://github.com/litentry/client-sdk/blob/develop/lib/requests/add-account.request.ts#L32)

___

### callEthereum

▸ **callEthereum**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

OmniAccount: Call an Ethereum contract.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.address` | `string` | Ethereum contract address |
| `data.input` | `U8aLike` | Contract input data |
| `data.omniAccount` | `LitentryIdentity` | The user's omniAccount. Use `createLitentryIdentityType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to signature
(if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is an
email identity, this is the email verification code. If the who is not an email identity, this
is the signed payload.

#### Defined in

[lib/requests/call-ethereum.request.ts:32](https://github.com/litentry/client-sdk/blob/develop/lib/requests/call-ethereum.request.ts#L32)

___

### createAccountStore

▸ **createAccountStore**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Creates an account store on the Litentry Parachain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.omniAccount` | `LitentryIdentity` | The user's OmniAccount. Use `createLitentryIdentityType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to sign (if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is
an email identity, this is the email verification code. If the who is not an email identity, this is the
signed payload.

#### Defined in

[lib/requests/create-account-store.request.ts:30](https://github.com/litentry/client-sdk/blob/develop/lib/requests/create-account-store.request.ts#L30)

___

### createChallengeCode

▸ **createChallengeCode**(`api`, `args`, `options?`): `Promise`\<`string`\>

Generates the challenge code to link an identity.

The challenge code is calculated from:

```
blake2_256(<enclaveNonce> + <primaryAccount> + <identityToLink>)
```

When `options.prettify` is set to true, the challenge code will be prefixed
with `Token: ` for utf-8 signatures support.
Otherwise, it will be returned as a hex string.

`options.prettify` feature is web3-specific. Ignored for web2.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `args` | `Object` | - |
| `args.identity` | `LitentryIdentity` | Identity to be linked. Use `createCorePrimitivesIdentityType` helper to create this struct |
| `args.who` | `LitentryIdentity` | The user's account. Use `createCorePrimitivesIdentityType` helper to create this struct |
| `options` | `Object` | - |
| `options.prettify?` | `boolean` | - |

#### Returns

`Promise`\<`string`\>

#### Defined in

[lib/requests/link-identity.request.ts:39](https://github.com/litentry/client-sdk/blob/develop/lib/requests/link-identity.request.ts#L39)

___

### getIdGraph

▸ **getIdGraph**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraph`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue`  }\>  }\>

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | - |
| `data` | `Object` | - |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |

#### Returns

`Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraph`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue`  }\>  }\>

#### Defined in

[lib/requests/get-id-graph.request.ts:14](https://github.com/litentry/client-sdk/blob/develop/lib/requests/get-id-graph.request.ts#L14)

___

### getIdGraphHash

▸ **getIdGraphHash**(`api`, `«destructured»`): `Promise`\<`H256` \| ``null``\>

Retrieve the idGraphHash for a given identity.

#### Parameters

| Name | Type |
| :------ | :------ |
| `api` | `ApiPromise` |
| `«destructured»` | `Object` |
| › `who` | `LitentryIdentity` |

#### Returns

`Promise`\<`H256` \| ``null``\>

#### Defined in

[lib/requests/get-id-graph-hash.ts:13](https://github.com/litentry/client-sdk/blob/develop/lib/requests/get-id-graph-hash.ts#L13)

___

### getLastRegisteredEnclave

▸ **getLastRegisteredEnclave**(`api`, `workerType?`): `Promise`\<\{ `account`: `AccountId32` ; `enclave`: `CorePrimitivesTeebagTypesEnclave`  }\>

Return the Enclave registry information of the latest registered TEE worker.

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | `undefined` |
| `workerType` | ``"Identity"`` \| ``"OmniExecutor"`` | `'Identity'` |

#### Returns

`Promise`\<\{ `account`: `AccountId32` ; `enclave`: `CorePrimitivesTeebagTypesEnclave`  }\>

#### Defined in

[lib/requests/get-last-registered-enclave.ts:11](https://github.com/litentry/client-sdk/blob/develop/lib/requests/get-last-registered-enclave.ts#L11)

___

### getOmniGraph

▸ **getOmniGraph**(`api`, `who`): `Promise`\<`MemberAccount`[] \| ``null``\>

Retrieve the omni account graph for a given identity.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Polkadot API instance |
| `who` | `Object` | Litentry identity to query |
| `who.who` | `LitentryIdentity` | - |

#### Returns

`Promise`\<`MemberAccount`[] \| ``null``\>

Array of member accounts or null if not found

**`Throws`**

Error if API calls fail

#### Defined in

[lib/requests/get-omni-graph.ts:13](https://github.com/litentry/client-sdk/blob/develop/lib/requests/get-omni-graph.ts#L13)

___

### linkIdentity

▸ **linkIdentity**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraphHash`: \`0x$\{string}\` ; `mutatedIdentities`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue` ; `txHash`: `string`  }\> ; `txHash`: `string`  }\>

Link an identity to the user's account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.identity` | `LitentryIdentity` | Identity to be linked. Use `createCorePrimitivesIdentityType` helper to create this struct |
| `data.networks` | (``"Polkadot"`` \| ``"Kusama"`` \| ``"Litentry"`` \| ``"Litmus"`` \| ``"LitentryRococo"`` \| ``"Khala"`` \| ``"SubstrateTestnet"`` \| ``"Ethereum"`` \| ``"Bsc"`` \| ``"BitcoinP2tr"`` \| ``"BitcoinP2pkh"`` \| ``"BitcoinP2sh"`` \| ``"BitcoinP2wpkh"`` \| ``"BitcoinP2wsh"`` \| ``"Polygon"`` \| ``"Arbitrum"`` \| ``"Solana"`` \| ``"Combo"``)[] | The networks to link the identity to, for web3 accounts |
| `data.validation` | `LitentryValidationData` | The ownership proof. Use `createLitentryValidationDataType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The prime identity. Use `createCorePrimitivesIdentityType` helper to create this struct |

#### Returns

`Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraphHash`: \`0x$\{string}\` ; `mutatedIdentities`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue` ; `txHash`: `string`  }\> ; `txHash`: `string`  }\>

#### Defined in

[lib/requests/link-identity.request.ts:75](https://github.com/litentry/client-sdk/blob/develop/lib/requests/link-identity.request.ts#L75)

___

### linkIdentityCallback

▸ **linkIdentityCallback**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraphHash`: \`0x$\{string}\` ; `mutatedIdentities`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue` ; `txHash`: `string`  }\> ; `txHash`: `string`  }\>

(internal) Link an identity to the user's account.

This function is only meant to be used in development networks where root or enclave_signer_account
are used as the signer.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.identity` | `LitentryIdentity` | Identity to be linked. Use `createCorePrimitivesIdentityType` helper to create this struct |
| `data.networks?` | (``"Polkadot"`` \| ``"Kusama"`` \| ``"Litentry"`` \| ``"Litmus"`` \| ``"LitentryRococo"`` \| ``"Khala"`` \| ``"SubstrateTestnet"`` \| ``"Ethereum"`` \| ``"Bsc"`` \| ``"BitcoinP2tr"`` \| ``"BitcoinP2pkh"`` \| ``"BitcoinP2sh"`` \| ``"BitcoinP2wpkh"`` \| ``"BitcoinP2wsh"`` \| ``"Polygon"`` \| ``"Arbitrum"`` \| ``"Solana"`` \| ``"Combo"``)[] | The networks to link the identity to, for web3 accounts |
| `data.signer` | `LitentryIdentity` | The signer. Use `createCorePrimitivesIdentityType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The prime identity. Use `createCorePrimitivesIdentityType` helper to create this struct |

#### Returns

`Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraphHash`: \`0x$\{string}\` ; `mutatedIdentities`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue` ; `txHash`: `string`  }\> ; `txHash`: `string`  }\>

#### Defined in

[lib/requests/link-identity-callback.request.ts:28](https://github.com/litentry/client-sdk/blob/develop/lib/requests/link-identity-callback.request.ts#L28)

___

### publicizeAccount

▸ **publicizeAccount**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Publicizes a member account in the AccountStore on the Litentry Parachain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.identity` | `LitentryIdentity` | The member account for publicizing. Use `createLitentryIdentityType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.

[payloadToSign] The payload to sign if the identity is not an email.

send A function to send the request to the Enclave.

send.args The arguments required to send the request.

send.args.authentication The authentication string. For email identities, this is the verification code. For non-email identities, this is the signed payload.

#### Defined in

[lib/requests/publicize-account.request.ts:29](https://github.com/litentry/client-sdk/blob/develop/lib/requests/publicize-account.request.ts#L29)

___

### remark

▸ **remark**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Sends a remark to the Litentry Parachain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.message` | `string` | the message to be sent |
| `data.omniAccount` | `LitentryIdentity` | The user's omniAccount. Use `createLitentryIdentityType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to sign
(if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is
an email identity, this is the email verification code. If the who is not an email identity,
this is the signed payload.

#### Defined in

[lib/requests/remark.request.ts:31](https://github.com/litentry/client-sdk/blob/develop/lib/requests/remark.request.ts#L31)

___

### removeAccounts

▸ **removeAccounts**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Removes accounts from the Litentry Parachain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.identities` | `LitentryIdentity`[] | Accounts for removing |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to sign (if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is
an email identity, this is the email verification code. If who is not an email identity, this is the
signed payload.

#### Defined in

[lib/requests/remove-accounts.request.ts:31](https://github.com/litentry/client-sdk/blob/develop/lib/requests/remove-accounts.request.ts#L31)

___

### requestAuthToken

▸ **requestAuthToken**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `token`: `string`  }\>  }\>

Requests an authentication token from the Enclave.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.expiresAt` | `number` | The block number at which the token expires |
| `data.omniAccount` | `LitentryIdentity` | The user's omniAccount. Use `createLitentryIdentityType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `token`: `string`  }\>  }\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.

[payloadToSign] The payload to sign if the identity is not an email.

send A function to send the request to the Enclave.

send.args The arguments required to send the request.

send.args.authentication The authentication string. For email identities, this is the verification code. For non-email identities, this is the signed payload.

#### Defined in

[lib/requests/request_auth_token.request.ts:25](https://github.com/litentry/client-sdk/blob/develop/lib/requests/request_auth_token.request.ts#L25)

___

### requestBatchVC

▸ **requestBatchVC**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }, `subscribeFn?`: (`error`: `Error` \| ``null``, `data`: \{ `index`: `number` ; `partialResult`: `WorkerRpcReturnValue`[] ; `vcPayload`: `Uint8Array`  }) => `void`) => `Promise`\<\{ `response`: `WorkerRpcReturnValue`[] ; `txHash`: `string` ; `vcPayloads`: (`Uint8Array` \| `Error`)[]  }\> ; `txHash`: `string`  }\>

Request a Batch of Verifiable Credential (VC) from the Litentry Protocol.

The send's subscribeFn is optional and is used to process the VC payload as it comes in.

The final response array, contains WorkerRpcReturnValue as they come in from the Enclave.
Notice that the response array is not ordered. Decoding the `WorkerRpcReturnValue.value`
into `RequestVcResultOrError` will contain the index of the request and the payload or error.

The information about available assertions and their payload can be found in the
`Assertion` (`Assertion`) type.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.assertions` | `Assertion`[] | the assertions to be claimed. See `Assertion` type |
| `data.signer?` | `LitentryIdentity` | The signer's account. Use `createLitentryIdentityType` helper to create this struct |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |

#### Returns

`Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }, `subscribeFn?`: (`error`: `Error` \| ``null``, `data`: \{ `index`: `number` ; `partialResult`: `WorkerRpcReturnValue`[] ; `vcPayload`: `Uint8Array`  }) => `void`) => `Promise`\<\{ `response`: `WorkerRpcReturnValue`[] ; `txHash`: `string` ; `vcPayloads`: (`Uint8Array` \| `Error`)[]  }\> ; `txHash`: `string`  }\>

#### Defined in

[lib/requests/request-batch-vc.request.ts:35](https://github.com/litentry/client-sdk/blob/develop/lib/requests/request-batch-vc.request.ts#L35)

___

### requestVerificationCode

▸ **requestVerificationCode**(`api`, `«destructured»`): `Promise`\<\{ `success`: `boolean`  }\>

Request email verification code.

#### Parameters

| Name | Type |
| :------ | :------ |
| `api` | `ApiPromise` |
| `«destructured»` | `Object` |
| › `email` | `string` |
| › `omniAccount` | \`0x$\{string}\` |

#### Returns

`Promise`\<\{ `success`: `boolean`  }\>

#### Defined in

[lib/requests/request-verification-code.request.ts:9](https://github.com/litentry/client-sdk/blob/develop/lib/requests/request-verification-code.request.ts#L9)

___

### setIdentityNetworks

▸ **setIdentityNetworks**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraphHash`: \`0x$\{string}\` ; `mutatedIdentities`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue` ; `txHash`: `string`  }\> ; `txHash`: `string`  }\>

Set the networks for a Web3 Identity.

It allows to change the list of `networks` for an already linked web3 identity.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.identity` | `LitentryIdentity` | Identity to be modified. Use `createLitentryIdentityType` helper to create this struct |
| `data.networks` | (``"Polkadot"`` \| ``"Kusama"`` \| ``"Litentry"`` \| ``"Litmus"`` \| ``"LitentryRococo"`` \| ``"Khala"`` \| ``"SubstrateTestnet"`` \| ``"Ethereum"`` \| ``"Bsc"`` \| ``"BitcoinP2tr"`` \| ``"BitcoinP2pkh"`` \| ``"BitcoinP2sh"`` \| ``"BitcoinP2wpkh"`` \| ``"BitcoinP2wsh"`` \| ``"Polygon"`` \| ``"Arbitrum"`` \| ``"Solana"`` \| ``"Combo"``)[] | Networks to be set |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |

#### Returns

`Promise`\<\{ `payloadToSign`: `string` ; `send`: (`args`: \{ `signedPayload`: `string`  }) => `Promise`\<\{ `idGraphHash`: \`0x$\{string}\` ; `mutatedIdentities`: [`IdGraph`](../README.md#idgraph) ; `response`: `WorkerRpcReturnValue` ; `txHash`: `string`  }\> ; `txHash`: `string`  }\>

#### Defined in

[lib/requests/set-identity-networks.request.ts:26](https://github.com/litentry/client-sdk/blob/develop/lib/requests/set-identity-networks.request.ts#L26)

___

### transferEthereum

▸ **transferEthereum**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Transfers ETH to another account on Ethereum.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.amount` | `bigint` | Amount to send |
| `data.omniAccount` | `LitentryIdentity` | The user's omniAccount. Use `createLitentryIdentityType` helper to create this struct |
| `data.to` | `string` | Ethereum address destination |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to signature
(if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is
an email identity, this is the email verification code. If the who is not an email identity,
this is the signed payload.

#### Defined in

[lib/requests/transfer-ethereum.request.ts:31](https://github.com/litentry/client-sdk/blob/develop/lib/requests/transfer-ethereum.request.ts#L31)

___

### transferNative

▸ **transferNative**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Transfers native tokens to another account on the Litentry Parachain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.amount` | `bigint` | Amount to send |
| `data.omniAccount` | `LitentryIdentity` | The user's omniAccount. Use `createLitentryIdentityType` helper to create this struct |
| `data.to` | `string` | Account destination in hex or ss58 formatted address |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to sign
(if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is
an email identity, this is the email verification code. If the who is not an email identity,
this is the signed payload.

#### Defined in

[lib/requests/transfer-native.request.ts:31](https://github.com/litentry/client-sdk/blob/develop/lib/requests/transfer-native.request.ts#L31)

___

### transferSolana

▸ **transferSolana**(`api`, `data`, `isWeb3Auth`): `Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

Transfers SOL to another account on Solana.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `api` | `ApiPromise` | Litentry Parachain API instance from Polkadot.js |
| `data` | `Object` | - |
| `data.amount` | `bigint` | Amount to send in lamports |
| `data.omniAccount` | `LitentryIdentity` | The user's omniAccount. Use `createLitentryIdentityType` helper to create this struct |
| `data.to` | `string` | Solana address destination |
| `data.who` | `LitentryIdentity` | The user's account. Use `createLitentryIdentityType` helper to create this struct |
| `isWeb3Auth` | `boolean` | Whether the user is using Web3 authentication |

#### Returns

`Promise`\<\{ `payloadToSign?`: `string` ; `send`: (`args`: \{ `authentication`: [`AuthenticationData`](../README.md#authenticationdata)  }) => `Promise`\<\{ `blockHash`: `string` ; `extrinsicHash`: `string` ; `response`: `WorkerRpcReturnValue`  }\>  }\>

- A promise that resolves to an object containing the payload to signature
(if applicable) and a send function.

[payloadToSign] - The payload to sign if who is not an email identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication string. If who is
an email identity, this is the email verification code. If the who is not an email identity,
this is the signed payload.

#### Defined in

[lib/requests/transfer-solana.request.ts:31](https://github.com/litentry/client-sdk/blob/develop/lib/requests/transfer-solana.request.ts#L31)
