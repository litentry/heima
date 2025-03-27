[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createKeyAesOutputType

# Function: createKeyAesOutputType()

> **createKeyAesOutputType**(`registry`, `data`): `AesOutput`

Defined in: [type-creators/key-aes-output.ts:27](https://github.com/litentry/heima/blob/dev/type-creators/key-aes-output.ts#L27)

Creates a KeyAesOutput sidechain type.

Heads-up: ensure data.ciphertext is in hex format. Using Uint may cause a bytes out range error.

## Parameters

### registry

`Registry`

### data

`` `0x${string}` `` | \{ `aad`: `` `0x${string}` `` \| `Uint8Array`; `ciphertext`: `` `0x${string}` ``; `nonce`: `` `0x${string}` `` \| `Uint8Array`; \}

## Returns

`AesOutput`

## Example

build from object
```ts
const identity = createKeyAesOutputType(registry, {
 ciphertext: '0x...',
 nonce: '0x...',
 aad: '0x...',
});
```

build from hex string
```ts
const identity = createKeyAesOutputType(registry, `0x...`);
```
