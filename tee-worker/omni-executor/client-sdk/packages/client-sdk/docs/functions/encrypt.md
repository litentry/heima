[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / encrypt

# Function: encrypt()

> **encrypt**(`args`, `shieldingKey`): `Promise`\<\{ `ciphertext`: `Uint8Array`; \}\>

Defined in: [utils/shielding-key.ts:51](https://github.com/litentry/heima/blob/dev/utils/shielding-key.ts#L51)

Encrypts the given cleartext with the given nonce and shielding key.

## Parameters

### args

#### cleartext

`Uint8Array`

#### nonce

`Uint8Array`

### shieldingKey

`CryptoKey`

## Returns

`Promise`\<\{ `ciphertext`: `Uint8Array`; \}\>
