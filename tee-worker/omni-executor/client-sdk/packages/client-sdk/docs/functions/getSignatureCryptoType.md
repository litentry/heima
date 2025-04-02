[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / getSignatureCryptoType

# Function: getSignatureCryptoType()

> **getSignatureCryptoType**(`args`): `"Ed25519"` \| `"Sr25519"` \| `"Ecdsa"` \| `"Ethereum"` \| `"None"`

Defined in: [utils/get-signature-crypto-type.ts:16](https://github.com/litentry/heima/blob/dev/utils/get-signature-crypto-type.ts#L16)

Returns the crypto type of the signature in capitalized format.

## Parameters

### args

The signature verification parameters

#### address

`string` \| `Uint8Array`

The address that signed the message (string or Uint8Array)

#### message

`string` \| `Uint8Array`

The message that was signed (string or Uint8Array)

#### signature

`string` \| `Uint8Array`

The signature to verify (string or Uint8Array)

## Returns

`"Ed25519"` \| `"Sr25519"` \| `"Ecdsa"` \| `"Ethereum"` \| `"None"`

The capitalized crypto type used for the signature:
         'None' | 'Ed25519' | 'Sr25519' | 'Ecdsa' | 'Ethereum'

## Throws

If the signature is invalid
