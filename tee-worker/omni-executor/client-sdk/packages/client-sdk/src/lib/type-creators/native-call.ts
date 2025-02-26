import { Registry } from '@polkadot/types-codec/types';
import { LitentryIdentity, NativeCall, omniExecutor } from '@litentry/parachain-api';

const NativeCallEnum = omniExecutor.types.NativeCall._enum;

type NativeCallMethod = keyof typeof NativeCallEnum;

const nativeCallMethodKeys = Object.keys(NativeCallEnum) as Array<NativeCallMethod>;
const nativeCallMethodsMap = nativeCallMethodKeys.reduce(
  (acc, key) => ({ ...acc, [key]: key }),
  {} as Record<NativeCallMethod, NativeCallMethod>,
);

type RequestAuthTokenParams = {
  identity: LitentryIdentity;
  authOptions: {
    expiresAt: number;
  };
};

export function createNativeCallType(
  registry: Registry,
  data: {
    method: 'request_auth_token';
    params: RequestAuthTokenParams;
  },
): NativeCall;

export function createNativeCallType(
  registry: Registry,
  data: {
    method: NativeCallMethod;
    params: Record<string, unknown>;
  },
): NativeCall {
  const { method, params } = data;

  if (isRequestAuthTokenCall(method, params)) {
    const { identity, authOptions } = params;

    const call = registry.createType('NativeCall', {
      [nativeCallMethodsMap.request_auth_token]: registry.createType(NativeCallEnum.request_auth_token, [
        identity,
        registry.createType('AuthOptions', {
          expires_at: authOptions.expiresAt,
        }),
      ]),
    }) as unknown as NativeCall;

    return call;
  }

  throw new Error(`native call method: ${data.method} is not supported`);
}

// TypeScript type guards to get the param's types right
function isRequestAuthTokenCall(
  method: NativeCallMethod,
  params: Record<string, unknown>,
): params is RequestAuthTokenParams {
  return method === nativeCallMethodsMap.request_auth_token;
}
