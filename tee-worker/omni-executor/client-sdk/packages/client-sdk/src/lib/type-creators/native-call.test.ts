import { TypeRegistry } from '@polkadot/types';

import { identity, type LitentryIdentity, omniAccount, omniExecutor } from '@litentry/parachain-api';

import { createLitentryIdentityType } from './litentry-identity';
import { createNativeCallType } from './native-call';

const types = {
  ...identity.types, // LitentryIdentity is defined here
  ...omniAccount.types, // AuthOptions is defined here
  ...omniExecutor.types, // NativeCall is defined here
};

let registry: TypeRegistry;
let aliceIdentity: LitentryIdentity;

beforeAll(() => {
  registry = new TypeRegistry();
  registry.register(types);

  // set up data
  aliceIdentity = createLitentryIdentityType(registry, {
    addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    type: 'Substrate',
  });
});

afterAll(() => {});

describe('RequestAuthToken', () => {
  test('it works', async () => {
    const call = createNativeCallType(registry, {
      method: 'request_auth_token',
      params: {
        identity: aliceIdentity,
        authOptions: {
          expiresAt: 1000,
        },
      },
    });

    expect(call).toBeDefined();
    expect(call.isRequestAuthToken).toBeTruthy();
    expect(call.asRequestAuthToken[0]).toEqual(aliceIdentity); // signer
    expect(call.asRequestAuthToken[1].toJSON().expires_at).toEqual(1000); // auth options
  });
});
