[**@heima/client-sdk**](../../../README.md)

***

[@heima/client-sdk](../../../README.md) / [request](../README.md) / getOmniAccountNonceWithIdentity

# Function: getOmniAccountNonceWithIdentity()

> **getOmniAccountNonceWithIdentity**(`api`, `identity`): `Promise`\<`Index`\>

Defined in: [requests/get-nonce.request.ts:15](https://github.com/litentry/heima/blob/dev/requests/get-nonce.request.ts#L15)

Retrieves the omni account nonce for a given identity.

## Parameters

### api

`ApiPromise`

Polkadot.js API instance

### identity

`Identity`

The identity to get the omni account nonce for

## Returns

`Promise`\<`Index`\>

Promise resolving to the nonce value
