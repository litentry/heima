[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createNativeTaskType

# Function: createNativeTaskType()

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:84](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L84)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"CreateAccountStore"`

##### params

`RequestCreateAccountStoreParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:92](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L92)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"AddAccount"`

##### params

`RequestAddAccountParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:100](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L100)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"RemoveAccounts"`

##### params

`RequestRemoveAccountsParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:108](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L108)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"PublicizeAccount"`

##### params

`RequestPublicizeAccountParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:116](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L116)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"SetPermissions"`

##### params

`RequestSetPermissionsParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:124](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L124)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"RequestIntent"`

##### params

`RequestIntentParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:132](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L132)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"RequestAuthToken"`

##### params

`RequestAuthTokenParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts

## Call Signature

> **createNativeTaskType**(`registry`, `data`): `object`

Defined in: [type-creators/native-task.ts:140](https://github.com/litentry/heima/blob/dev/type-creators/native-task.ts#L140)

Creates the NativeTask for the given method and provide the `param's` types expected for them.

Heads-up:
This must match the Rust implementation of the NativeTask

### Parameters

#### registry

`Registry`

#### data

##### method

`"PumpxRequestJwt"`

##### params

`RequestPumpxRequestJwtParams`

### Returns

`object`

#### task

> **task**: `NativeTask`

### See

 - https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs

Similarly, our types definitions must match also.
 - https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts
