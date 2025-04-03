[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / systemRemark

# Function: systemRemark()

> **systemRemark**(`api`, `data`): `Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/intents/system-remark.request.ts:28](https://github.com/litentry/heima/blob/dev/requests/intents/system-remark.request.ts#L28)

Intent Request: Sends a system remark to the Heima Parachain.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### message

`string`

The message to be sent.

## Returns

`Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to sign
(if applicable) and a send function.
