[**@heima/client-sdk**](../README.md)

***

[@heima/client-sdk](../README.md) / createValidationDataType

# Function: createValidationDataType()

> **createValidationDataType**\<`IIdentityType`\>(`registry`, `identityDescriptor`, `proof`): `ValidationData`

Defined in: [type-creators/validation-data.ts:123](https://github.com/litentry/heima/blob/dev/type-creators/validation-data.ts#L123)

Creates the ValidationData given the identity network and its type.

The proof to pass depends on the identity network (IdentityType):
- Web3: Web3Proof
- Twitter: TwitterProof
- Discord: DiscordProof

## Type Parameters

• **IIdentityType** *extends* `"Twitter"` \| `"Discord"` \| `"Github"` \| `"Substrate"` \| `"Evm"` \| `"Bitcoin"` \| `"Solana"` \| `"Email"`

## Parameters

### registry

`Registry`

Parachain API's type registry

### identityDescriptor

#### addressOrHandle

`string`

The address or handle of the identity

#### type

`IIdentityType`

The identity type

### proof

`IIdentityType` *extends* `"Discord"` ? [`DiscordProof`](../type-aliases/DiscordProof.md) \| [`DiscordOAuth2Proof`](../type-aliases/DiscordOAuth2Proof.md) : `IIdentityType` *extends* `"Twitter"` ? [`TwitterProof`](../type-aliases/TwitterProof.md) \| [`TwitterOAuth2Proof`](../type-aliases/TwitterOAuth2Proof.md) : `IIdentityType` *extends* `"Email"` ? [`EmailProof`](../type-aliases/EmailProof.md) : [`Web3Proof`](../type-aliases/Web3Proof.md)

The ownership proof

## Returns

`ValidationData`

## Examples

```ts
import { createValidationDataType } from '@heima/client-sdk';
import type { Web3Proof } from '@heima/client-sdk';

const userAddress = '0x123';

const proof: Web3Proof = {
  signature: '0x123',
  message: '0x123',
}

const validationData = createValidationDataType(
  registry,
  {
    addressOrHandle: userAddress,
    type: 'Evm',
  },
  proof,
);
```

```ts
import { createValidationDataType } from '@heima/client-sdk';
import type { TwitterProof } from '@heima/client-sdk';

const userHandle = '@heima';

const proof: TwitterProof = {
  // Both twitter.com and x.com are valid
  tweetId: 'https://twitter.com/0x123/status/123',
};

const validationData = createValidationDataType(
  registry,
  {
    addressOrHandle: userHandle,
    type: 'Twitter',
  },
  proof,
);
```
