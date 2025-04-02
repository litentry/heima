[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createRawTaskType

# Function: createRawTaskType()

> **createRawTaskType**(`api`, `data`): `Promise`\<`RawTask`\>

Defined in: [type-creators/raw-task.ts:19](https://github.com/litentry/heima/blob/dev/type-creators/raw-task.ts#L19)

Creates a RawTask struct type for the `NativeTask`.

A shielding key is generated and used to encrypt the `NativeTask` and communicated
to the enclave to protect the data for transportation.

The shielding key is encrypted using the Enclave's shielding key and attached in the Task.

## Parameters

### api

`ApiPromise`

### data

#### authData?

[`OmniAuthData`](../type-aliases/OmniAuthData.md)

#### nonce?

`Index`

#### plain?

`boolean`

#### task

`NativeTask`

## Returns

`Promise`\<`RawTask`\>
