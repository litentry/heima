[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / getAndWaitForAccountStoreCreation

# Function: getAndWaitForAccountStoreCreation()

> **getAndWaitForAccountStoreCreation**(`api`, `account`): `Promise`\<`MemberAccount`[]\>

Defined in: [test-utils/helpers.ts:17](https://github.com/litentry/heima/blob/dev/test-utils/helpers.ts#L17)

Retrieves and waits for the account store to be created for the specified account.

This function polls the blockchain at 1-second intervals until it confirms
that an account store has been created for the given account.

## Parameters

### api

`ApiPromise`

The Polkadot API instance used to query the blockchain

### account

The account to check for an associated account store

`string` | `Uint8Array` | `AccountId32`

## Returns

`Promise`\<`MemberAccount`[]\>

A Promise that resolves to the value of the account store when it is confirmed to exist
