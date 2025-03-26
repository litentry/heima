[**@heima/client-sdk**](../../../../README.md)

***

[@heima/client-sdk](../../../../README.md) / [request](../README.md) / removeAccounts

# Function: removeAccounts()

> **removeAccounts**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/remove-accounts.request.ts:27](https://github.com/litentry/heima/blob/dev/requests/remove-accounts.request.ts#L27)

Removes accounts from the Heima Parachain.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### membersToRemove

`Identity`[]

The member accounts for removing from the OmniAccount. Use the `createCorePrimitivesIdentityType` helper to create this structure.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
