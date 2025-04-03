[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / u8aToBase64Url

# Function: u8aToBase64Url()

> **u8aToBase64Url**(`value`): `string`

Defined in: [utils/u8aToBase64Url.ts:21](https://github.com/litentry/heima/blob/dev/utils/u8aToBase64Url.ts#L21)

Creates a base64-URL value. Padding is omitted.

## Parameters

### value

`U8aLike`

## Returns

`string`

## See

https://en.wikipedia.org/wiki/Base64#RFC_4648

## Example

```ts
import { stringToU8a } from '@polkadot/util';
import { base64Encode } from '@polkadot/util-crypto';

const input = stringToU8a('fo ob');
const base64Output = base64Encode(input);
const base64urlOutput = u8aToBase64Url(input);

console.log(`base64: ${base64Output}`);
console.log(`base64url: ${base64urlOutput}`);
// base64: Zm8gYm8=
// base64url: Zm8gYm8
```
