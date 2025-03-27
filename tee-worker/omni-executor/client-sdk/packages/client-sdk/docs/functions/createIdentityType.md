[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createIdentityType

# Function: createIdentityType()

> **createIdentityType**(`registry`, `data`): `Identity`

Defined in: [type-creators/identity.ts:31](https://github.com/litentry/heima/blob/dev/type-creators/identity.ts#L31)

Creates a Identity chain type.

Notice that addresses and handles are not fully validated. This struct shouldn't be relied on for validation.

For Bitcoin, the compressed public key is expected (begins with 02 or 03).

For Solana, the address string is expected to be a base58-encoded or hex-encoded string.

For Substrate, the address is expected to be a SS58-encoded or hex-encoded address.

## Parameters

### registry

`Registry`

### data

`` `0x${string}` `` | `Uint8Array` | \{ `addressOrHandle`: `string`; `type`: `"Twitter"` \| `"Discord"` \| `"Github"` \| `"Substrate"` \| `"Evm"` \| `"Bitcoin"` \| `"Solana"` \| `"Email"`; \}

## Returns

`Identity`

## Example

```ts
const substrateIdentity = createIdentityType(registry, {
 addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
 type: 'Substrate',
});

const twitterIdentity = createIdentityType(registry, {
 addressOrHandle: 'my-twitter-handle',
 type: 'Twitter',
});
```
