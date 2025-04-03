[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / verifySignature

# Function: verifySignature()

> **verifySignature**(`args`): `VerifyResult`

Defined in: [utils/verify-signature.ts:16](https://github.com/litentry/heima/blob/dev/utils/verify-signature.ts#L16)

Verifies a cryptographic signature against a message and an address.

## Parameters

### args

The verification parameters

#### address

`string` \| `Uint8Array`

The signer's address (either as a string or Uint8Array)

#### message

`string` \| `Uint8Array`

The message to verify (either as a string or Uint8Array)

#### signature

`string` \| `Uint8Array`

The signature to verify (either as a string or Uint8Array)

## Returns

`VerifyResult`

Object containing verification result with properties:
                        - isValid: boolean indicating if signature is valid
                        - crypto: string indicating the crypto type used
