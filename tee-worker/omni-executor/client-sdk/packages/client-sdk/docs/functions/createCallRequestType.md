[**@heima/client-sdk**](../README.md)

***

[@heima/client-sdk](../README.md) / createCallRequestType

# Function: createCallRequestType()

> **createCallRequestType**(`api`, `data`): `Promise`\<`OmniAesRequest` \| `PlainRequest`\>

Defined in: [type-creators/request.ts:19](https://github.com/litentry/heima/blob/dev/type-creators/request.ts#L19)

Creates a Request struct type for the `NativeCall` operation.

A shielding key is generated and used to encrypt the `TrustedCall` operation and communicated
to the enclave to protect the data for transportation.

The shielding key is encrypted using the Enclave's shielding key and attached in the Request.

## Parameters

### api

`ApiPromise`

### data

#### authentication

[`AuthenticationData`](../type-aliases/AuthenticationData.md)

#### mrEnclave

`Uint8Array`

#### nonce

`Index`

#### operation

`NativeCall`

#### plain?

`boolean`

## Returns

`Promise`\<`OmniAesRequest` \| `PlainRequest`\>
