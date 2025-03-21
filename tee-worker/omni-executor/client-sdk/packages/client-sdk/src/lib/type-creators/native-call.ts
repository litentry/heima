import { Registry } from '@polkadot/types-codec/types';
import {
  Intent,
  Identity,
  ValidationData,
  NativeCall,
  OmniAccountPermission,
  omniExecutor,
} from '@heima/parachain-api';

const NativeCallEnum = omniExecutor.types.NativeCall._enum;

// Collect methods in a single place. so typescript can help if anything changes
type NativeCallMethod = keyof typeof NativeCallEnum;

const nativeCallMethodKeys = Object.keys(NativeCallEnum) as Array<NativeCallMethod>;
const nativeCallMethodsMap = nativeCallMethodKeys.reduce(
  (acc, key) => ({ ...acc, [key]: key }),
  {} as Record<NativeCallMethod, NativeCallMethod>,
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

// Identity, AuthOptions
type RequestAuthTokenParams = {
  member: Identity;
  authOptions: {
    expiresAt: number;
  };
};

/**
 * Creates the NativeCall for the given method and provide the `param's` types expected for them.
 *
 * Heads-up:
 * This must match the Rust implementation of the NativeCall
 * @see https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs
 *
 * Similarly, our types definitions must match also.
 * @see https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts
 */
export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'create_account_store';
    params: RequestCreateAccountStoreParams;
  },
): { operation: NativeCall };

export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'add_account';
    params: RequestAddAccountParams;
  },
): { operation: NativeCall };

export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'remove_accounts';
    params: RequestRemoveAccountsParams;
  },
): { operation: NativeCall };

export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'publicize_account';
    params: RequestPublicizeAccountParams;
  },
): { operation: NativeCall };

export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'set_permissions';
    params: RequestSetPermissionsParams;
  },
): { operation: NativeCall };

export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'request_intent';
    params: RequestIntentParams;
  },
): { operation: NativeCall };

export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'request_auth_token';
    params: RequestAuthTokenParams;
  },
): { operation: NativeCall };

export function createNativeCallType(
  registry: Registry,
  data: {
    method: NativeCallMethod;
    params: Record<string, unknown>;
  },
): { operation: NativeCall } {
  const { method, params } = data;

  if (isRequestCreateAccountStoreCall(method, params)) {
    const { member } = params;

    const operation = registry.createType<NativeCall>('NativeCall', {
      [nativeCallMethodsMap.create_account_store]: member,
    });

    return { operation };
  }

  if (isRequestAddAccountCall(method, params)) {
    const { member, memberToAdd, validation, isPublic, permissions } = params;

    const operation = registry.createType<NativeCall>('NativeCall', {
      [nativeCallMethodsMap.add_account]: registry.createType(NativeCallEnum.add_account, [
        member,
        memberToAdd,
        validation,
        isPublic,
        permissions,
      ]),
    });

    return { operation };
  }

  if (isRequestRemoveAccountsCall(method, params)) {
    const { member, membersToRemove } = params;

    const operation = registry.createType<NativeCall>('NativeCall', {
      [nativeCallMethodsMap.remove_accounts]: registry.createType(NativeCallEnum.remove_accounts, [
        member,
        membersToRemove,
      ]),
    });

    return { operation };
  }

  if (isRequestPublicizeAccountCall(method, params)) {
    const { member, memberToPublicize } = params;

    const operation = registry.createType<NativeCall>('NativeCall', {
      [nativeCallMethodsMap.publicize_account]: registry.createType(NativeCallEnum.publicize_account, [
        member,
        memberToPublicize,
      ]),
    });

    return { operation };
  }

  if (isRequestSetPermissionsCall(method, params)) {
    const { member, memberToSetPermissions, permissions } = params;

    const operation = registry.createType<NativeCall>('NativeCall', {
      [nativeCallMethodsMap.set_permissions]: registry.createType(NativeCallEnum.set_permissions, [
        member,
        memberToSetPermissions,
        permissions,
      ]),
    });

    return { operation };
  }

  if (isRequestIntentCall(method, params)) {
    const { member, intent } = params;

    const operation = registry.createType<NativeCall>('NativeCall', {
      [nativeCallMethodsMap.request_intent]: registry.createType(NativeCallEnum.request_intent, [member, intent]),
    });

    return { operation };
  }

  if (isRequestAuthTokenCall(method, params)) {
    const { member, authOptions } = params;

    const operation = registry.createType<NativeCall>('NativeCall', {
      [nativeCallMethodsMap.request_auth_token]: registry.createType(NativeCallEnum.request_auth_token, [
        member,
        registry.createType('AuthOptions', {
          expires_at: authOptions.expiresAt,
        }),
      ]),
    });

    return { operation };
  }

  throw new Error(`native call method: ${data.method} is not supported`);
}

// TypeScript type guards to get the param's types right
function isRequestCreateAccountStoreCall(
  method: NativeCallMethod,
  params: Record<string, unknown>,
): params is RequestCreateAccountStoreParams {
  return method === nativeCallMethodsMap.create_account_store;
}

function isRequestAddAccountCall(
  method: NativeCallMethod,
  params: Record<string, unknown>,
): params is RequestAddAccountParams {
  return method === nativeCallMethodsMap.add_account;
}

function isRequestRemoveAccountsCall(
  method: NativeCallMethod,
  params: Record<string, unknown>,
): params is RequestRemoveAccountsParams {
  return method === nativeCallMethodsMap.remove_accounts;
}

function isRequestPublicizeAccountCall(
  method: NativeCallMethod,
  params: Record<string, unknown>,
): params is RequestPublicizeAccountParams {
  return method === nativeCallMethodsMap.publicize_account;
}

function isRequestSetPermissionsCall(
  method: NativeCallMethod,
  params: Record<string, unknown>,
): params is RequestSetPermissionsParams {
  return method === nativeCallMethodsMap.set_permissions;
}

function isRequestIntentCall(method: NativeCallMethod, params: Record<string, unknown>): params is RequestIntentParams {
  return method === nativeCallMethodsMap.request_intent;
}

function isRequestAuthTokenCall(
  method: NativeCallMethod,
  params: Record<string, unknown>,
): params is RequestAuthTokenParams {
  return method === nativeCallMethodsMap.request_auth_token;
}
