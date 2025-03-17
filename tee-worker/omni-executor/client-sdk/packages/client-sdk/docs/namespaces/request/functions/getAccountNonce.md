[**@heima/client-sdk**](../../../README.md)

***

[@heima/client-sdk](../../../README.md) / [request](../README.md) / getAccountNonce

# Function: getAccountNonce()

> **getAccountNonce**(`api`, `account`): `Promise`\<`Index`\>

Defined in: [requests/get-nonce.request.ts:33](https://github.com/litentry/heima/blob/dev/requests/get-nonce.request.ts#L33)

Retrieves the nonce for a given account.

## Parameters

### api

`ApiPromise`

Polkadot.js API instance

### account

The account to get the nonce for

`string` | `Uint8Array` | `AccountId`

## Returns

`Promise`\<`Index`\>

Promise resolving to the nonce value
