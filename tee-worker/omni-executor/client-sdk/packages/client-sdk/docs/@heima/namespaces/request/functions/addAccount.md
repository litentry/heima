[**@heima/client-sdk**](../../../../README.md)

***

[@heima/client-sdk](../../../../README.md) / [request](../README.md) / addAccount

# Function: addAccount()

> **addAccount**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/add-account.request.ts:30](https://github.com/litentry/heima/blob/dev/requests/add-account.request.ts#L30)

Adds an account to the Heima Parachain.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### isPublic

`boolean`

Whether the account is public.

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### memberToAdd

`Identity`

The member account for adding to the OmniAccount. Use the `createCorePrimitivesIdentityType` helper to create this structure.

#### permissions?

`OmniAccountPermission`[]

The permissions for the account.

#### validation

`ValidationData`

The ownership proof. Use the `createValidationDataType` helper to create this structure.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
