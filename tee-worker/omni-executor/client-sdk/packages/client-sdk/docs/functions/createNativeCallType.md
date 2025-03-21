[**@heima/client-sdk**](../README.md)

***

[@heima/client-sdk](../README.md) / createNativeCallType

# Function: createNativeCallType()

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:79](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L79)

Creates the NativeCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeCall

### Parameters

#### registry

`Registry`

#### data

##### method

`"create_account_store"`

##### params

`RequestCreateAccountStoreParams`

### Returns

`object`

#### operation

> **operation**: `NativeCall`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:87](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L87)

Creates the NativeCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeCall

### Parameters

#### registry

`Registry`

#### data

##### method

`"add_account"`

##### params

`RequestAddAccountParams`

### Returns

`object`

#### operation

> **operation**: `NativeCall`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:95](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L95)

Creates the NativeCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeCall

### Parameters

#### registry

`Registry`

#### data

##### method

`"remove_accounts"`

##### params

`RequestRemoveAccountsParams`

### Returns

`object`

#### operation

> **operation**: `NativeCall`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:103](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L103)

Creates the NativeCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeCall

### Parameters

#### registry

`Registry`

#### data

##### method

`"publicize_account"`

##### params

`RequestPublicizeAccountParams`

### Returns

`object`

#### operation

> **operation**: `NativeCall`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:111](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L111)

Creates the NativeCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeCall

### Parameters

#### registry

`Registry`

#### data

##### method

`"set_permissions"`

##### params

`RequestSetPermissionsParams`

### Returns

`object`

#### operation

> **operation**: `NativeCall`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:119](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L119)

Creates the NativeCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeCall

### Parameters

#### registry

`Registry`

#### data

##### method

`"request_intent"`

##### params

`RequestIntentParams`

### Returns

`object`

#### operation

> **operation**: `NativeCall`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:127](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L127)

Creates the NativeCall for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeCall

### Parameters

#### registry

`Registry`

#### data

##### method

`"request_auth_token"`

##### params

`RequestAuthTokenParams`

### Returns

`object`

#### operation

> **operation**: `NativeCall`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts
