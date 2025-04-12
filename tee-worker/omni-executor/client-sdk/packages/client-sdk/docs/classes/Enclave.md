[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / Enclave

# Class: Enclave

Defined in: [enclave.ts:68](https://github.com/litentry/heima/blob/dev/enclave.ts#L68)

This is a singleton class to mainly hold the Enclave's Shielding Key and MrEnclave.

With this class you can:
- Retrieve the Enclave's Shielding Key. (1)
- Retrieve the Enclave's MrEnclave value which is used as the mrEnclave value. (1)
- Encrypt data using the Enclave's Shielding Key.
- Send request to the Enclave.

(1) Querying from the Parachain, instead of directly from the Enclave Worker itself helps
ensuring clients are connected to a trusted worker.

1. Using the global enclave instance:

## Examples

```ts
import { enclave } from '@heima-network/client-sdk';

const mrEnclave = await enclave.getMrEnclave(api);
const key = await enclave.getShieldingKey();

console.log({ mrEnclave, key });

// Encrypt data using the Enclave's Shielding Key
const encrypted = await enclave.encrypt({ cleartext: new Uint8Array([1, 2, 3]) });

// Send request to the Enclave.
const response = await enclave.send({
 jsonrpc: '2.0',
 method: 'native_submitAesRequest',
 params: ['0x123']
});
```

2. Create your own enclave instance:

```ts
import { Enclave } from '@heima-network/client-sdk';

const enclave = new Enclave('ws://tee-dev.litentry.io');

// you can also set debug mode for logging debug logs
enclave.setEnableDebug(true);
```

## Constructors

### Constructor

> **new Enclave**(`endpoint`, `config`): `Enclave`

Defined in: [enclave.ts:101](https://github.com/litentry/heima/blob/dev/enclave.ts#L101)

Creates a new Omni client instance

#### Parameters

##### endpoint

`string`

WebSocket endpoint URL

##### config

`Partial`\<`EnclaveConfig`\> = `{}`

Optional configuration overrides

#### Returns

`Enclave`

## Methods

### disconnect()

> **disconnect**(): `void`

Defined in: [enclave.ts:270](https://github.com/litentry/heima/blob/dev/enclave.ts#L270)

Closes the WebSocket connection and performs cleanup
Use this method to properly terminate the client connection

#### Returns

`void`

***

### encrypt()

> **encrypt**(`__namedParameters`): `Promise`\<\{ `ciphertext`: `Uint8Array`; \}\>

Defined in: [enclave.ts:207](https://github.com/litentry/heima/blob/dev/enclave.ts#L207)

#### Parameters

##### \_\_namedParameters

###### cleartext

`Uint8Array`

#### Returns

`Promise`\<\{ `ciphertext`: `Uint8Array`; \}\>

***

### getConnectionState()

> **getConnectionState**(): [`ConnectionState`](../enumerations/ConnectionState.md)

Defined in: [enclave.ts:123](https://github.com/litentry/heima/blob/dev/enclave.ts#L123)

Returns the current connection state

#### Returns

[`ConnectionState`](../enumerations/ConnectionState.md)

***

### getMrEnclave()

> **getMrEnclave**(`api`): `Promise`\<`` `0x${string}` ``\>

Defined in: [enclave.ts:137](https://github.com/litentry/heima/blob/dev/enclave.ts#L137)

Retrieve the Enclave's mrEnclave from the Parachain.

The Enclave registry contains the information of the registered TEE workers. These TEE Workers share the
same Enclave's mrEnclave value.

The value will be held in memory for the duration of the session.

#### Parameters

##### api

`ApiPromise`

#### Returns

`Promise`\<`` `0x${string}` ``\>

#### See

Test it by yourself https://polkadot.js.org/apps/?rpc=wss://tee-dev.litentry.io#/chainstate

***

### getShieldingKey()

> **getShieldingKey**(): `Promise`\<`CryptoKey`\>

Defined in: [enclave.ts:172](https://github.com/litentry/heima/blob/dev/enclave.ts#L172)

Get the Enclave's Shielding Key.

#### Returns

`Promise`\<`CryptoKey`\>

Promise that resolves with the crypto key, the value will be held in memory for the duration of the session.

***

### send()

> **send**(`payload`, `options?`): `Promise`\<`string`\>

Defined in: [enclave.ts:227](https://github.com/litentry/heima/blob/dev/enclave.ts#L227)

Sends a JSON-RPC request over the WebSocket connection

#### Parameters

##### payload

[`JsonRpcRequest`](../type-aliases/JsonRpcRequest.md)

The JSON-RPC request to send

##### options?

Optional settings including custom timeout

###### timeout?

`number`

#### Returns

`Promise`\<`string`\>

Promise that resolves with the response

***

### setEnableDebug()

> **setEnableDebug**(`enable`): `void`

Defined in: [enclave.ts:116](https://github.com/litentry/heima/blob/dev/enclave.ts#L116)

Set debug mode.

#### Parameters

##### enable

`boolean`

Enable debug logs

#### Returns

`void`

***

### getInstance()

> `static` **getInstance**(): `Enclave`

Defined in: [enclave.ts:89](https://github.com/litentry/heima/blob/dev/enclave.ts#L89)

#### Returns

`Enclave`
