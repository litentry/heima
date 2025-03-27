[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / callEthereum

# Function: callEthereum()

> **callEthereum**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/intents/call-ethereum.request.ts:28](https://github.com/litentry/heima/blob/dev/requests/intents/call-ethereum.request.ts#L28)

Intent Request: Call an Ethereum contract.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### address

`string`

The Ethereum contract address.

#### input

`U8aLike`

The contract input data.

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to signature
(if applicable) and a send function.
