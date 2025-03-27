[**@heima/client-sdk**](../../../../README.md)

***

[@heima/client-sdk](../../../../README.md) / [request](../README.md) / setPermissions

# Function: setPermissions()

> **setPermissions**(`api`, `data`): `Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

Defined in: [requests/set-permissions.request.ts:28](https://github.com/litentry/heima/blob/dev/requests/set-permissions.request.ts#L28)

Set the permissions for a specified account within the Heima Parachain.

## Parameters

### api

`ApiPromise`

The Heima Parachain API instance from Polkadot.js.

### data

The data object containing the following properties:

#### member

`Identity`

The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.

#### memberToSetPermissions

`Identity`

The account to be updated. Use the `createCorePrimitivesIdentityType` helper to create this structure.

#### permissions

`OmniAccountPermission`[]

The permissions to be assigned to the account.

## Returns

`Promise`\<\{ `payloadToSign`: `string`; `send`: (`args`) => `Promise`\<\{ `blockHash`: `` `0x${string}` ``; `extrinsicHash`: `` `0x${string}` ``; `status`: `` `0x${string}` ``; \}\>; \}\>

- A promise that resolves to an object containing the payload to sign (if applicable) and a function to send the request.
