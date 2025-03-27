import { Keyring } from '@polkadot/api';
import { TypeRegistry } from '@polkadot/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import { identity, Identity, omniExecutor, sidechain } from '@heima-network/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { createNativeCallType } from '@type-creators/native-call';

const types = {
  ...identity.types, // Identity is defined here
  ...omniExecutor.types, // NativeCall is defined here
  ...sidechain.types, // AesOutput is defined here
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
      },
    });

    expect(operation).toBeDefined();
    expect(operation.isRequestAuthToken).toBeTruthy();
    expect(operation.asRequestAuthToken).toEqual(member); // signer
  });

  it('create_account_store', async () => {
    const { operation } = createNativeCallType(registry, {
      method: 'create_account_store',
      params: {
        member,
      },
    });

    expect(operation).toBeDefined();
    expect(operation.isCreateAccountStore).toBeTruthy();
    expect(operation.asCreateAccountStore).toEqual(member); // signer
  });
});
