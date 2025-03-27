[**@heima/client-sdk**](../../../../README.md)

***

[@heima/client-sdk](../../../../README.md) / [request](../README.md) / transferEthereum

# Function: transferEthereum()

> **transferEthereum**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/intents/transfer-ethereum.request.ts:29](https://github.com/litentry/heima/blob/dev/requests/intents/transfer-ethereum.request.ts#L29)

Intent Request: Transfers ETH to another account on Ethereum.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### amount

`` `0x${string}` ``

The amount to send.

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### to

`string`

The Ethereum address destination.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to signature
(if applicable) and a send function.
