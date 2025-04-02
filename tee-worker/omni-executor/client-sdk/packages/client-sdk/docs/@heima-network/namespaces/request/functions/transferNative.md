[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / transferNative

# Function: transferNative()

> **transferNative**(`api`, `data`): `Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/intents/transfer-native.request.ts:28](https://github.com/litentry/heima/blob/dev/requests/intents/transfer-native.request.ts#L28)

Intent Request: Transfers native tokens to another account on the  Parachain.

## Parameters

### api

`ApiPromise`

The  Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### amount

`bigint`

The amount to send.

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### to

`string`

The account destination in hex or ss58 formatted address.

## Returns

`Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to sign
(if applicable) and a send function.
