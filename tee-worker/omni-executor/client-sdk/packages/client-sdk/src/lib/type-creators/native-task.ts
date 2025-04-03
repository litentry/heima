import { Registry } from '@polkadot/types-codec/types';
import {
  Intent,
  Identity,
  ValidationData,
  NativeTask,
  OmniAccountPermission,
  omniExecutor,
} from '@heima-network/parachain-api';

const NativeTaskEnum = omniExecutor.types.NativeTask._enum;

// Collect methods in a single place. so typescript can help if anything changes
type NativeTaskMethod = keyof typeof NativeTaskEnum;

const NativeTaskMethodKeys = Object.keys(NativeTaskEnum) as Array<NativeTaskMethod>;
const NativeTaskMethodsMap = NativeTaskMethodKeys.reduce(
  (acc, key) => ({ ...acc, [key]: key }),
  {} as Record<NativeTaskMethod, NativeTaskMethod>,
);

// Identity
type RequestCreateAccountStoreParams = {
  member: Identity;
};

// Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>
type RequestAddAccountParams = {
  member: Identity;
  memberToAdd: Identity;
  validation: ValidationData;
  isPublic: boolean;
  permissions?: Array<OmniAccountPermission>;
};

// Identity, Array<Identity>
type RequestRemoveAccountsParams = {
  member: Identity;
  membersToRemove: Array<Identity>;
};

// Identity, Identity
type RequestPublicizeAccountParams = {
  member: Identity;
  memberToPublicize: Identity;
};

// Identity, Identity, Vec<OmniAccountPermission>
type RequestSetPermissionsParams = {
  member: Identity;
  memberToSetPermissions: Identity;
  permissions: Array<OmniAccountPermission>;
};

// Identity, Intent
type RequestIntentParams = {
  member: Identity;
  intent: Intent;
};

// Identity
type RequestAuthTokenParams = {
  member: Identity;
};

// Identity, Option<String>, Option<String>, Option<String>
type RequestPumpxRequestJwtParams = {
  member: Identity;
  inviteCode?: string;
  googleCode?: string;
  lang?: string;
};

/**
 * Creates the NativeTask for the given method and provide the `param's` types expected for them.
 *
 * Heads-up:
 * This must match the Rust implementation of the NativeTask
 * @see https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_task.rs
 *
 * Similarly, our types definitions must match also.
 * @see https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts
 */
export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'CreateAccountStore';
    params: RequestCreateAccountStoreParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'AddAccount';
    params: RequestAddAccountParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'RemoveAccounts';
    params: RequestRemoveAccountsParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'PublicizeAccount';
    params: RequestPublicizeAccountParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'SetPermissions';
    params: RequestSetPermissionsParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'RequestIntent';
    params: RequestIntentParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'RequestAuthToken';
    params: RequestAuthTokenParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: 'PumpxRequestJwt';
    params: RequestPumpxRequestJwtParams;
  },
): { task: NativeTask };

export function createNativeTaskType(
  registry: Registry,
  data: {
    method: NativeTaskMethod;
    params: Record<string, unknown>;
  },
): { task: NativeTask } {
  const { method, params } = data;

  if (isRequestCreateAccountStoreTask(method, params)) {
    const { member } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.CreateAccountStore]: member,
    });

    return { task };
  }

  if (isRequestAddAccountTask(method, params)) {
    const { member, memberToAdd, validation, isPublic, permissions } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.AddAccount]: registry.createType(NativeTaskEnum.AddAccount, [
        member,
        memberToAdd,
        validation,
        isPublic,
        permissions,
      ]),
    });

    return { task };
  }

  if (isRequestRemoveAccountsTask(method, params)) {
    const { member, membersToRemove } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.RemoveAccounts]: registry.createType(NativeTaskEnum.RemoveAccounts, [
        member,
        membersToRemove,
      ]),
    });

    return { task };
  }

  if (isRequestPublicizeAccountTask(method, params)) {
    const { member, memberToPublicize } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.PublicizeAccount]: registry.createType(NativeTaskEnum.PublicizeAccount, [
        member,
        memberToPublicize,
      ]),
    });

    return { task };
  }

  if (isRequestSetPermissionsTask(method, params)) {
    const { member, memberToSetPermissions, permissions } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.SetPermissions]: registry.createType(NativeTaskEnum.SetPermissions, [
        member,
        memberToSetPermissions,
        permissions,
      ]),
    });

    return { task };
  }

  if (isRequestIntentTask(method, params)) {
    const { member, intent } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.RequestIntent]: registry.createType(NativeTaskEnum.RequestIntent, [member, intent]),
    });

    return { task };
  }

  if (isRequestAuthTokenTask(method, params)) {
    const { member } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.RequestAuthToken]: member,
    });

    return { task };
  }

  if (isRequestPumpxRequestJwtTask(method, params)) {
    const { member, inviteCode, googleCode, lang } = params;

    const task = registry.createType<NativeTask>('NativeTask', {
      [NativeTaskMethodsMap.PumpxRequestJwt]: [
        member,
        registry.createType('Option<String>', inviteCode),
        registry.createType('Option<String>', googleCode),
        registry.createType('Option<String>', lang),
      ],
    });

    return { task };
  }

  throw new Error(`native task method: ${data.method} is not supported`);
}

// TypeScript type guards to get the param's types right
function isRequestCreateAccountStoreTask(
  method: NativeTaskMethod,
  params: Record<string, unknown>,
): params is RequestCreateAccountStoreParams {
  return method === NativeTaskMethodsMap.CreateAccountStore;
}

function isRequestAddAccountTask(
  method: NativeTaskMethod,
  params: Record<string, unknown>,
): params is RequestAddAccountParams {
  return method === NativeTaskMethodsMap.AddAccount;
}

function isRequestRemoveAccountsTask(
  method: NativeTaskMethod,
  params: Record<string, unknown>,
): params is RequestRemoveAccountsParams {
  return method === NativeTaskMethodsMap.RemoveAccounts;
}

function isRequestPublicizeAccountTask(
  method: NativeTaskMethod,
  params: Record<string, unknown>,
): params is RequestPublicizeAccountParams {
  return method === NativeTaskMethodsMap.PublicizeAccount;
}

function isRequestSetPermissionsTask(
  method: NativeTaskMethod,
  params: Record<string, unknown>,
): params is RequestSetPermissionsParams {
  return method === NativeTaskMethodsMap.SetPermissions;
}

function isRequestIntentTask(method: NativeTaskMethod, params: Record<string, unknown>): params is RequestIntentParams {
  return method === NativeTaskMethodsMap.RequestIntent;
}

function isRequestAuthTokenTask(
  method: NativeTaskMethod,
  params: Record<string, unknown>,
): params is RequestAuthTokenParams {
  return method === NativeTaskMethodsMap.RequestAuthToken;
}

function isRequestPumpxRequestJwtTask(
  method: NativeTaskMethod,
  params: Record<string, unknown>,
): params is RequestPumpxRequestJwtParams {
  return method === NativeTaskMethodsMap.PumpxRequestJwt;
}
