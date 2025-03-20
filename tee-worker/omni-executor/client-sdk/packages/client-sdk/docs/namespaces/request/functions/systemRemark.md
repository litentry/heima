[**@heima/client-sdk**](../../../README.md)

***

[@heima/client-sdk](../../../README.md) / [request](../README.md) / systemRemark

# Function: systemRemark()

> **systemRemark**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

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
