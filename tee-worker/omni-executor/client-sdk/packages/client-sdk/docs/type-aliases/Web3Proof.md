[**@heima/client-sdk**](../README.md)

***

[@heima/client-sdk](../README.md) / Web3Proof

# Type Alias: Web3Proof

> **Web3Proof**: `object`

Defined in: [type-creators/validation-data.ts:17](https://github.com/litentry/heima/blob/dev/type-creators/validation-data.ts#L17)

Ownership proof for Web3 accounts (Substrate, EVM, Bitcoin).

Bitcoin signatures are base64-encoded strings. Substrate and EVM signatures are hex-encoded strings.

## Type declaration

### message

> **message**: `string`

### signature

> **signature**: `` `0x${string}` `` \| `string`

## See

createIdentityType
