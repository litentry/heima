[**@heima/client-sdk**](../../../README.md)

***

[@heima/client-sdk](../../../README.md) / [request](../README.md) / getAccountStore

# Function: getAccountStore()

> **getAccountStore**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<`Identity`[]\>; \}\>

Defined in: [requests/get-account-store.request.ts:24](https://github.com/litentry/heima/blob/dev/requests/get-account-store.request.ts#L24)

Gets an account store from the Enclave.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<`Identity`[]\>; \}\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.

payloadToSign - The payload to sign if the identity is a Web3 identity.

send - A function to send the request to the Enclave.

send.args The arguments required to send the request.

send.args.authentication - The authentication data.

send.return - The account store.
