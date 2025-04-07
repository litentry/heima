[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / publicizeAccount

# Function: publicizeAccount()

> **publicizeAccount**(`api`, `data`, `enclaveInstance`): `Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/publicize-account.request.ts:29](https://github.com/litentry/heima/blob/dev/requests/publicize-account.request.ts#L29)

Publicizes a member account in the AccountStore on the Heima Parachain.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### memberToPublicize

`Identity`

The member account for publicizing. Use the `createIdentityType` helper to create this structure.

### enclaveInstance

[`Enclave`](../../../../classes/Enclave.md) = `enclave`

The enclave instance use to interact with Enclave.

## Returns

`Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
