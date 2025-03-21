[**@heima/client-sdk**](../../../README.md)

***

[@heima/client-sdk](../../../README.md) / [request](../README.md) / transferSolana

# Function: transferSolana()

> **transferSolana**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/intents/transfer-solana.request.ts:29](https://github.com/litentry/heima/blob/dev/requests/intents/transfer-solana.request.ts#L29)

Intent Request: Transfers SOL to another account on Solana.

## Parameters

### api

`ApiPromise`

The  Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### amount

`bigint`

The amount to send in lamports.

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### to

`string`

The Solana address destination.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to signature
(if applicable) and a send function.

payloadToSign - The payload to sign if the identity is a Web3 identity.

send - A function to send the request to the Enclave.

send.args - The arguments required to send the request.

send.args.authentication - The authentication data.

send.return.blockHash - Block hash of the transaction

send.return.extrinsicHash - Extrinsic hash of the transaction

send.return.status - Status of the transaction
