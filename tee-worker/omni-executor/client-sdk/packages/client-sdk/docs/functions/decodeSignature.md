[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / decodeSignature

# Function: decodeSignature()

> **decodeSignature**(`signature`, `identity`): `Uint8Array`

Defined in: [utils/decode-signature.ts:13](https://github.com/litentry/heima/blob/dev/utils/decode-signature.ts#L13)

Decodes a signature string into a Uint8Array based on the identity type.

## Parameters

### signature

`string`

The signature string to decode

### identity

`Identity`

The identity object containing the type information

## Returns

`Uint8Array`

A Uint8Array containing the decoded signature

## Throws

When signature is empty or format is unsupported
