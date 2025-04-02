[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / publicizeAccount

# Function: publicizeAccount()

> **publicizeAccount**(`api`, `data`): `Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/publicize-account.request.ts:27](https://github.com/litentry/heima/blob/dev/requests/publicize-account.request.ts#L27)

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

## Returns

`Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
