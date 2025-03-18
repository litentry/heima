[**@heima/client-sdk**](../../../README.md)

***

[@heima/client-sdk](../../../README.md) / [request](../README.md) / requestAuthToken

# Function: requestAuthToken()

> **requestAuthToken**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `token`: `string`; \}\>; \}\>

Defined in: [requests/request-auth-token.request.ts:33](https://github.com/litentry/heima/blob/dev/requests/request-auth-token.request.ts#L33)

Requests an authentication token from the Enclave.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### expiresAt

`number`

The block number at which the token expires.

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `token`: `string`; \}\>; \}\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.

payloadToSign - The payload to sign if the identity is a Web3 identity.

send - A function to send the request to the Enclave.

send.args The arguments required to send the request.

send.args.authentication - The authentication data.

send.return.blockHash - Block hash of the transaction

send.return.extrinsicHash - Extrinsic hash of the transaction

send.return.status - Status of the transaction
