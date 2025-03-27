[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createQueryRequestType

# Function: createQueryRequestType()

> **createQueryRequestType**(`api`, `data`): `Promise`\<`OmniAesRequest` \| `PlainRequest`\>

Defined in: [type-creators/request.ts:49](https://github.com/litentry/heima/blob/dev/type-creators/request.ts#L49)

Creates a Request struct type for the `NativeQuery` operation.

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

`NativeQuery`

#### plain?

`boolean`

## Returns

`Promise`\<`OmniAesRequest` \| `PlainRequest`\>
