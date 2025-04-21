[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / JsonRpcRequest

# Type Alias: JsonRpcRequest

> **JsonRpcRequest** = `object`

Defined in: [utils/types.ts:3](https://github.com/litentry/heima/blob/dev/utils/types.ts#L3)

## Properties

### id?

> `optional` **id**: `number`

Defined in: [utils/types.ts:11](https://github.com/litentry/heima/blob/dev/utils/types.ts#L11)

Use sequential numbers starting from 1 for consecutive requests.
For one-time request that closes connections right away, using `1` is ok.

***

### jsonrpc

> **jsonrpc**: `string`

Defined in: [utils/types.ts:4](https://github.com/litentry/heima/blob/dev/utils/types.ts#L4)

***

### method

> **method**: `string`

Defined in: [utils/types.ts:5](https://github.com/litentry/heima/blob/dev/utils/types.ts#L5)

***

### params

> **params**: `any`

Defined in: [utils/types.ts:6](https://github.com/litentry/heima/blob/dev/utils/types.ts#L6)
