[**@heima/client-sdk**](../../../../README.md)

***

[@heima/client-sdk](../../../../README.md) / [request](../README.md) / getAccountStore

# Function: getAccountStore()

> **getAccountStore**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<`Identity`[]\>; \}\>

Defined in: [requests/get-account-store.request.ts:23](https://github.com/litentry/heima/blob/dev/requests/get-account-store.request.ts#L23)

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
