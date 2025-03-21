[**@heima/client-sdk**](../../../README.md)

***

[@heima/client-sdk](../../../README.md) / [request](../README.md) / transferNative

# Function: transferNative()

> **transferNative**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

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

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to sign
(if applicable) and a send function.

payloadToSign - The payload to sign if the identity is a Web3 identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication data.

send.return.blockHash - Block hash of the transaction

send.return.extrinsicHash - Extrinsic hash of the transaction

send.return.status - Status of the transaction
