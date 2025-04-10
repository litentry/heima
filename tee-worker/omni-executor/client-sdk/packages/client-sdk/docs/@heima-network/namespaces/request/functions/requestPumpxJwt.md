[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / requestPumpxJwt

# Function: requestPumpxJwt()

> **requestPumpxJwt**(`api`, `data`, `enclaveInstance`): `Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `jwt`: `PumpxRequestJwt`; \}\>; \}\>

Defined in: [requests/pumpx-request-jwt.request.ts:35](https://github.com/litentry/heima/blob/dev/requests/pumpx-request-jwt.request.ts#L35)

Requests a pumpx JWT from the Enclave.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### googleCode?

`string`

Optional Google authentication code.

#### inviteCode?

`string`

Optional invite code for PumpX registration.

#### lang?

`string`

Optional language preference for PumpX.

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

### enclaveInstance

[`Enclave`](../../../../classes/Enclave.md) = `enclave`

The enclave instance use to interact with Enclave.

## Returns

`Promise`\<\{ `getPayloadToSign`: () => `Promise`\<`string`\>; `send`: (`args`) => `Promise`\<\{ `jwt`: `PumpxRequestJwt`; \}\>; \}\>

A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
