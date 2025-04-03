import { Keyring } from '@polkadot/api';
import { TypeRegistry } from '@polkadot/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import { identity, Identity, omniExecutor, sidechain } from '@heima-network/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { createNativeTaskType } from '@type-creators/native-task';

const types = {
  ...identity.types, // Identity is defined here
  ...omniExecutor.types, // NativeTask is defined here
  ...sidechain.types, // AesOutput is defined here
};

const registry = new TypeRegistry();
registry.register(types);

describe('native task', () => {
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

  it('RequestAuthToken', async () => {
    const { task } = createNativeTaskType(registry, {
      method: 'RequestAuthToken',
      params: {
        member,
      },
    });

    expect(task).toBeDefined();
    expect(task.isRequestAuthToken).toBeTruthy();
    expect(task.asRequestAuthToken).toEqual(member); // signer
  });

  it('CreateAccountStore', async () => {
    const { task } = createNativeTaskType(registry, {
      method: 'CreateAccountStore',
      params: {
        member,
      },
    });

    expect(task).toBeDefined();
    expect(task.isCreateAccountStore).toBeTruthy();
    expect(task.asCreateAccountStore).toEqual(member); // signer
  });
});
