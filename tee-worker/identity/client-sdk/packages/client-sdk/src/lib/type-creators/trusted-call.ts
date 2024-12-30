import { compactAddLength } from '@polkadot/util';

import type { Registry } from '@polkadot/types-codec/types';

import { trusted_operations, type Intent } from '@litentry/parachain-api';
import type {
  TrustedCall,
  LitentryIdentity,
  LitentryValidationData,
  AuthOptions,
  Web3Network,
  Assertion,
} from '@litentry/parachain-api';

import * as shieldingKeyUtils from '../util/shielding-key';

// collect methods in a single place. so typescript can help if anything changes
type TrustedCallMethod =
  keyof typeof trusted_operations.types.TrustedCall._enum;

const trustedCallMethodKeys = Object.keys(
  trusted_operations.types.TrustedCall._enum
) as Array<TrustedCallMethod>;
const trustedCallMethodsMap = trustedCallMethodKeys.reduce(
  (acc, key) => ({ ...acc, [key]: key }),
  {} as Record<TrustedCallMethod, TrustedCallMethod>
);

// (LitentryIdentity, LitentryIdentity, LitentryIdentity, LitentryValidationData, Vec<Web3Network>, Option<RequestAesKey>, H256)
type LinkIdentityParams = {
  who: LitentryIdentity;
  identity: LitentryIdentity;
  validation: LitentryValidationData;
  networks: Array<Web3Network['type']>;
  hash: `0x${string}`;
};

// (LitentryIdentity, LitentryIdentity, LitentryIdentity, Vec<Web3Network>, Option<RequestAesKey>, H256)
type LinkIdentityCallbackParams = {
  signer: LitentryIdentity;
  who: LitentryIdentity;
  identity: LitentryIdentity;
  networks: Array<Web3Network['type']>;
  hash: `0x${string}`;
};

// LitentryIdentity, LitentryIdentity, Assertion, Option<RequestAesKey>, H256;
type RequestVcParams = {
  who: LitentryIdentity;
  assertion: Assertion;
  hash: `0x${string}`;
};

// LitentryIdentity, LitentryIdentity, Vec<Assertion>, Option<RequestAesKey>, H256;
type RequestBatchVcParams = {
  signer: LitentryIdentity;
  who: LitentryIdentity;
  assertions: Array<Assertion>;
  hash: `0x${string}`;
};

// (LitentryIdentity, LitentryIdentity, LitentryIdentity, Vec<Web3Network>, Option<RequestAesKey>, H256)
type SetIdentityNetworksParams = {
  who: LitentryIdentity;
  identity: LitentryIdentity;
  networks: Array<Web3Network['type']>;
  hash: `0x${string}`;
};

// LitentryIdentity, Intent
type RequestIntentParams = {
  who: LitentryIdentity;
  intent: Intent;
};

// LitentryIdentity
type RequestCreateAccountStoreParams = {
  who: LitentryIdentity;
};

// LitentryIdentity, LitentryIdentity, LitentryValidationData, boolean
type RequestAddAccountParams = {
  who: LitentryIdentity;
  identity: LitentryIdentity;
  validation: LitentryValidationData;
  isPublic: boolean;
};

// LitentryIdentity, Array<LitentryIdentity>
type RequestRemoveAccountsParams = {
  who: LitentryIdentity;
  identities: Array<LitentryIdentity>;
};

// LitentryIdentity, LitentryIdentity
type RequestPublicizeAccountParams = {
  who: LitentryIdentity;
  identity: LitentryIdentity;
};

// LitentryIdentity, AuthOptions
type RequestAuthTokenParams = {
  who: LitentryIdentity;
  options: AuthOptions;
};

/**
 * Creates the TrustedCall for the given method and provide the `param's` types expected for them.
 *
 * Heads-up:
 * This must match the Rust implementation of the TrustedCall
 * @see https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/identity/app-libs/stf/src/trusted_call.rs
 *
 * Similarly, our types definitions must match also.
 * @see https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/identity/client-api/parachain-api/prepare-build/interfaces/trusted_operations/definitions.ts
 */
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'link_identity';
    params: LinkIdentityParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'set_identity_networks';
    params: SetIdentityNetworksParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'request_vc';
    params: RequestVcParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'request_batch_vc';
    params: RequestBatchVcParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'request_intent';
    params: RequestIntentParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'create_account_store';
    params: RequestCreateAccountStoreParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'add_account';
    params: RequestAddAccountParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'remove_accounts';
    params: RequestRemoveAccountsParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'publicize_account';
    params: RequestPublicizeAccountParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'request_auth_token';
    params: RequestAuthTokenParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: 'link_identity_callback';
    params: LinkIdentityCallbackParams;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }>;
export async function createTrustedCallType(
  registry: Registry,
  data: {
    method: TrustedCallMethod;
    params: Record<string, unknown>;
  }
): Promise<{ call: TrustedCall; key: CryptoKey }> {
  const { method, params } = data;

  // generate ephemeral shielding key to encrypt the user-sensitive result in the DI response
  const key = await shieldingKeyUtils.generate();
  const keyU8 = await shieldingKeyUtils.exportKey(key);

  if (isLinkIdentityCall(method, params)) {
    const { who, identity, validation, networks, hash } = params;

    const networksVec = registry.createType('Vec<Web3Network>', networks);
    const optionAesKey = registry.createType(
      'Option<RequestAesKey>',
      compactAddLength(keyU8)
    );

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.link_identity]: registry.createType(
        trusted_operations.types.TrustedCall._enum.link_identity,
        [who, who, identity, validation, networksVec, optionAesKey, hash]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isLinkIdentityCallback(method, params)) {
    const { signer, who, identity, networks, hash } = params;

    const networksVec = registry.createType('Vec<Web3Network>', networks);
    const optionAesKey = registry.createType(
      'Option<RequestAesKey>',
      compactAddLength(keyU8)
    );

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.link_identity_callback]: registry.createType(
        trusted_operations.types.TrustedCall._enum.link_identity_callback,
        [signer, who, identity, networksVec, optionAesKey, hash]
      ),
    });

    return { call, key };
  }

  if (isRequestVcCall(method, params)) {
    const { who, assertion, hash } = params;

    const optionAesKey = registry.createType(
      'Option<RequestAesKey>',
      compactAddLength(keyU8)
    );

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.request_vc]: registry.createType(
        trusted_operations.types.TrustedCall._enum.request_vc,
        [who, who, assertion, optionAesKey, hash]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isRequestBatchVcCall(method, params)) {
    const { signer, who, assertions, hash } = params;

    const optionAesKey = registry.createType(
      'Option<RequestAesKey>',
      compactAddLength(keyU8)
    );

    const vecAssertions = registry.createType('Vec<Assertion>', assertions);

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.request_batch_vc]: registry.createType(
        trusted_operations.types.TrustedCall._enum.request_batch_vc,
        [signer, who, vecAssertions, optionAesKey, hash]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isSetIdentityNetworksCall(method, params)) {
    const { who, identity, networks, hash } = params;

    const networksVec = registry.createType('Vec<Web3Network>', networks);
    const optionAesKey = registry.createType(
      'Option<RequestAesKey>',
      compactAddLength(keyU8)
    );

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.set_identity_networks]: registry.createType(
        trusted_operations.types.TrustedCall._enum.set_identity_networks,
        [who, who, identity, networksVec, optionAesKey, hash]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isRequestIntentCall(method, params)) {
    const { who, intent } = params;

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.request_intent]: registry.createType(
        trusted_operations.types.TrustedCall._enum.request_intent,
        [who, intent]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isRequestCreateAccountStoreParams(method, params)) {
    const { who } = params;

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.create_account_store]: registry.createType(
        trusted_operations.types.TrustedCall._enum.create_account_store,
        who
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isRequestAddAccountParams(method, params)) {
    const { who, identity, validation, isPublic } = params;

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.add_account]: registry.createType(
        trusted_operations.types.TrustedCall._enum.add_account,
        [who, identity, validation, isPublic]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isRequestRemoveAccountsParams(method, params)) {
    const { who, identities } = params;

    const identitiesVec = registry.createType(
      'Vec<LitentryIdentity>',
      identities
    );
    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.remove_accounts]: registry.createType(
        trusted_operations.types.TrustedCall._enum.remove_accounts,
        [who, identitiesVec]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isRequestPublicizeAccountParams(method, params)) {
    const { who, identity } = params;

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.publicize_account]: registry.createType(
        trusted_operations.types.TrustedCall._enum.publicize_account,
        [who, identity]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  if (isRequestAuthTokenParams(method, params)) {
    const { who, options } = params;

    const call = registry.createType('TrustedCall', {
      [trustedCallMethodsMap.request_auth_token]: registry.createType(
        trusted_operations.types.TrustedCall._enum.request_auth_token,
        [who, options]
      ),
    }) as TrustedCall;

    return { call, key };
  }

  throw new Error(`trusted call method: ${data.method} is not supported`);
}

// TypeScript type guards to get the param's types right
function isLinkIdentityCall(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is LinkIdentityParams {
  return method === 'link_identity';
}
function isLinkIdentityCallback(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is LinkIdentityCallbackParams {
  return method === 'link_identity_callback';
}
function isSetIdentityNetworksCall(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is SetIdentityNetworksParams {
  return method === 'set_identity_networks';
}
function isRequestVcCall(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestVcParams {
  return method === 'request_vc';
}
function isRequestBatchVcCall(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestBatchVcParams {
  return method === 'request_batch_vc';
}
function isRequestIntentCall(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestIntentParams {
  return method === 'request_intent';
}
function isRequestCreateAccountStoreParams(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestCreateAccountStoreParams {
  return method === 'create_account_store';
}
function isRequestAddAccountParams(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestAddAccountParams {
  return method === 'add_account';
}

function isRequestRemoveAccountsParams(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestRemoveAccountsParams {
  return method === 'remove_accounts';
}

function isRequestPublicizeAccountParams(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestPublicizeAccountParams {
  return method === 'publicize_account';
}

function isRequestAuthTokenParams(
  method: TrustedCallMethod,
  params: Record<string, unknown>
): params is RequestAuthTokenParams {
  return method === 'request_auth_token';
}
