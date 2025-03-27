[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / getAccountNonce

# Function: getAccountNonce()

> **getAccountNonce**(`api`, `account`): `Promise`\<`Index`\>

Defined in: [requests/get-nonce.request.ts:32](https://github.com/litentry/heima/blob/dev/requests/get-nonce.request.ts#L32)

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
