[**@heima/client-sdk**](../README.md)

***

[@heima/client-sdk](../README.md) / createNativeCallType

# Function: createNativeCallType()

## Call Signature

> **createNativeCallType**(`registry`, `data`): `object`

Defined in: [type-creators/native-call.ts:66](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L66)

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

Defined in: [type-creators/native-call.ts:74](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L74)

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

Defined in: [type-creators/native-call.ts:82](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L82)

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

Defined in: [type-creators/native-call.ts:90](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L90)

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

Defined in: [type-creators/native-call.ts:98](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L98)

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

Defined in: [type-creators/native-call.ts:106](https://github.com/litentry/heima/blob/dev/type-creators/native-call.ts#L106)

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
