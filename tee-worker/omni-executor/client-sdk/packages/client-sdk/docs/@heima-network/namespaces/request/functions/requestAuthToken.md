[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / requestAuthToken

# Function: requestAuthToken()

> **requestAuthToken**(`api`, `data`): `Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `token`: `string`; \}\>; \}\>

Defined in: [requests/request-auth-token.request.ts:32](https://github.com/litentry/heima/blob/dev/requests/request-auth-token.request.ts#L32)

Requests an auth token from the Enclave.

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

`Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `token`: `string`; \}\>; \}\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
