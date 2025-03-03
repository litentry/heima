import { Registry } from '@polkadot/types-codec/types';
import { Identity, NativeCall, omniExecutor } from '@heima/parachain-api';

const NativeCallEnum = omniExecutor.types.NativeCall._enum;

// Collect methods in a single place. so typescript can help if anything changes
type NativeCallMethod = keyof typeof NativeCallEnum;

const nativeCallMethodKeys = Object.keys(NativeCallEnum) as Array<NativeCallMethod>;
const nativeCallMethodsMap = nativeCallMethodKeys.reduce(
  (acc, key) => ({ ...acc, [key]: key }),
  {} as Record<NativeCallMethod, NativeCallMethod>,
);

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

  if (isRequestAuthTokenCall(method, params)) {
    const { member, authOptions } = params;

    const operation = registry.createType('NativeCall', {
      [nativeCallMethodsMap.request_auth_token]: registry.createType(NativeCallEnum.request_auth_token, [
        member,
        registry.createType('AuthOptions', {
          expires_at: authOptions.expiresAt,
        }),
      ]),
    }) as unknown as NativeCall;

    return { operation };
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
