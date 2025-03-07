import { Keyring } from '@polkadot/api';
import { TypeRegistry } from '@polkadot/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import { identity, Identity, omniAccount, omniExecutor } from '@heima/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { createNativeCallType } from '@type-creators/native-call';

const types = {
  ...identity.types, // Identity is defined here
  ...omniAccount.types, // AuthOptions is defined here
  ...omniExecutor.types, // NativeCall is defined here
};

const registry = new TypeRegistry();
registry.register(types);

describe('native call', () => {
  let member: Identity;

  beforeAll(async () => {
    await cryptoWaitReady();
    const keyring = new Keyring({ type: 'sr25519' });
    const memberSigner = keyring.addFromUri('//Alice');
    member = createIdentityType(registry, {
      addressOrHandle: memberSigner.address,
      type: 'Substrate',
    });
  });

  it('request_auth_token', async () => {
    const { operation } = createNativeCallType(registry, {
      method: 'request_auth_token',
      params: {
        member,
        authOptions: {
          expiresAt: 1000,
        },
      },
    });

    expect(operation).toBeDefined();
    expect(operation.isRequestAuthToken).toBeTruthy();
    expect(operation.asRequestAuthToken[0]).toEqual(member); // signer
    expect(operation.asRequestAuthToken[1].toJSON().expires_at).toEqual(1000); // auth options
  });
});
