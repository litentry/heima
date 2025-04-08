[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createPayloadToSign

# Function: createPayloadToSign()

> **createPayloadToSign**(`args`): `string`

Defined in: [utils/create-payload-to-sign.ts:18](https://github.com/litentry/heima/blob/dev/utils/create-payload-to-sign.ts#L18)

Constructs a message that users need to sign to authorize Enclave's tasks.
The message is created by concatenating the operation, nonce and mrEnclave,
then hashing it with blake2 and adding a prefix.

## Parameters

### args

#### mrEnclave

`Uint8Array`

The mrEnclave value

#### nonce

`Index`

Transaction nonce to prevent replay attacks

#### task

`NativeTask`

The task to be authorized

#### who

`Identity`

The identity of the signer

## Returns

`string`

A formatted message string ready for signing
