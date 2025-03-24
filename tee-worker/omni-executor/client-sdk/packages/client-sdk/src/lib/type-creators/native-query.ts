import { Registry } from '@polkadot/types-codec/types';
import { Identity, NativeQuery, omniExecutor } from '@heima/parachain-api';

const NativeQueryEnum = omniExecutor.types.NativeQuery._enum;

// Collect methods in a single place. so typescript can help if anything changes
type NativeQueryMethod = keyof typeof NativeQueryEnum;

const nativeQueryMethodKeys = Object.keys(NativeQueryEnum) as Array<NativeQueryMethod>;
const nativeQueryMethodsMap = nativeQueryMethodKeys.reduce(
  (acc, key) => ({ ...acc, [key]: key }),
  {} as Record<NativeQueryMethod, NativeQueryMethod>,
);

// Identity
type RequestGetAccountStoreParams = {
  member: Identity;
};

/**
 * Creates the NativeQuery for the given method and provide the `param's` types expected for them.
 *
 * Heads-up:
 * This must match the Rust implementation of the NativeQuery
 * @see https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/executor-core/src/native_operation.rs
 *
 * Similarly, our types definitions must match also.
 * @see https://github.com/litentry/heima/blob/dev/tee-worker/client-api/parachain-api/prepare-build/interfaces/omniExecutor/definitions.ts
 */
export function createNativeQueryType(
  registry: Registry,
  data: {
    method: 'get_account_store';
    params: RequestGetAccountStoreParams;
  },
): { operation: NativeQuery };

export function createNativeQueryType(
  registry: Registry,
  data: {
    method: NativeQueryMethod;
    params: Record<string, unknown>;
  },
): { operation: NativeQuery } {
  const { method, params } = data;

  if (isRequestGetAccountStoreQuery(method, params)) {
    const { member } = params;

    const operation = registry.createType<NativeQuery>('NativeQuery', {
      [nativeQueryMethodsMap.get_account_store]: member,
    });

    return { operation };
  }

  throw new Error(`native query method: ${data.method} is not supported`);
}

// TypeScript type guards to get the param's types right
function isRequestGetAccountStoreQuery(
  method: NativeQueryMethod,
  params: Record<string, unknown>,
): params is RequestGetAccountStoreParams {
  return method === nativeQueryMethodsMap.get_account_store;
}
