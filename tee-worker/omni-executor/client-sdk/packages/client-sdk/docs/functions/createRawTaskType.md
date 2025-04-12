[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createRawTaskType

# Function: createRawTaskType()

Creates a RawTask wrapper for NativeTask with optional encryption.

When plain=true:
- Returns unencrypted RawTask directly

When plain=false or omitted:
1. Generates ephemeral shielding key
2. Encrypts the task using client shielding key
3. Encrypts the shielding key using Enclave's public key
4. Packages as AES task

## Param

Polkadot API instance

## Param

Task configuration

## Param

Native task to be wrapped

## Param

Optional nonce value

## Param

Optional authentication data

## Param

Whether to skip encryption, defaults to false

## Param

The enclave instance use to interact with Enclave.

## Examples

```ts
// Get unencrypted raw task
const rawTask = await createRawTaskType(api, { task, plain: true });
```

```ts
// Get encrypted raw task with key
const { rawTask, encryptionKey } = await createRawTaskType(api, { task });
```

## Call Signature

> **createRawTaskType**(`api`, `data`, `enclaveInstance?`): `Promise`\<`RawTask`\>

Defined in: [type-creators/raw-task.ts:23](https://github.com/litentry/heima/blob/dev/type-creators/raw-task.ts#L23)

Creates a raw task type for unencrypted NativeTask.

### Parameters

#### api

`ApiPromise`

Polkadot API instance

#### data

Task configuration

##### authData?

[`OmniAuthData`](../type-aliases/OmniAuthData.md)

Optional authentication data

##### nonce?

`Index`

Optional nonce value

##### plain

`true`

Flag to return unencrypted raw task

##### task

`NativeTask`

Native task to be wrapped

#### enclaveInstance?

[`Enclave`](../classes/Enclave.md)

The enclave instance use to interact with Enclave.

### Returns

`Promise`\<`RawTask`\>

Either RawTask or object with encrypted RawTask and encryption key

### Param

Polkadot API instance

### Param

Task configuration

### Param

Native task to be wrapped

### Param

Optional nonce value

### Param

Optional authentication data

### Param

Whether to skip encryption, defaults to false

### Param

The enclave instance use to interact with Enclave.

### Examples

```ts
// Get unencrypted raw task
const rawTask = await createRawTaskType(api, { task, plain: true });
```

```ts
// Get encrypted raw task with key
const { rawTask, encryptionKey } = await createRawTaskType(api, { task });
```

## Call Signature

> **createRawTaskType**(`api`, `data`, `enclaveInstance?`): `Promise`\<\{ `encryptionKey`: `CryptoKey`; `rawTask`: `RawTask`; \}\>

Defined in: [type-creators/raw-task.ts:46](https://github.com/litentry/heima/blob/dev/type-creators/raw-task.ts#L46)

Creates an encrypted raw task type.

### Parameters

#### api

`ApiPromise`

Polkadot API instance

#### data

Task configuration

##### authData?

[`OmniAuthData`](../type-aliases/OmniAuthData.md)

Optional authentication data

##### nonce?

`Index`

Optional nonce value

##### plain?

`false`

Flag to return encrypted raw task (default)

##### task

`NativeTask`

Native task to be wrapped

#### enclaveInstance?

[`Enclave`](../classes/Enclave.md)

The enclave instance use to interact with Enclave.

### Returns

`Promise`\<\{ `encryptionKey`: `CryptoKey`; `rawTask`: `RawTask`; \}\>

Either RawTask or object with encrypted RawTask and encryption key

### Param

Polkadot API instance

### Param

Task configuration

### Param

Native task to be wrapped

### Param

Optional nonce value

### Param

Optional authentication data

### Param

Whether to skip encryption, defaults to false

### Param

The enclave instance use to interact with Enclave.

### Examples

```ts
// Get unencrypted raw task
const rawTask = await createRawTaskType(api, { task, plain: true });
```

```ts
// Get encrypted raw task with key
const { rawTask, encryptionKey } = await createRawTaskType(api, { task });
```
