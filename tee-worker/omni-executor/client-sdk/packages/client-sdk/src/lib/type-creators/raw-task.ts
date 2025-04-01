import type { Index } from '@polkadot/types/interfaces';
import type { ApiPromise } from '@polkadot/api';
import { compactAddLength, u8aToHex } from '@polkadot/util';

import type { AesOutput, AesTask, NativeTask, NativeTaskWrapper, RawTask } from '@heima-network/parachain-api';
import { enclave } from '@lib/enclave';
import { encrypt, generateNonce12, generate, exportKey } from '@utils/shielding-key';
import { createAesOutputType } from './aes-output';
import { createOmniAuth, OmniAuthData } from './omni-auth';

/**
 * Creates a RawTask struct type for the `NativeTask`.
 *
 * A shielding key is generated and used to encrypt the `NativeTask` and communicated
 * to the enclave to protect the data for transportation.
 *
 * The shielding key is encrypted using the Enclave's shielding key and attached in the Task.
 */
export async function createRawTaskType(
  api: ApiPromise,
  data: {
    task: NativeTask;
    nonce?: Index;
    authData?: OmniAuthData;
    plain?: boolean;
  },
): Promise<RawTask> {
  const { authData, task, nonce, plain = false } = data;

  const auth = authData ? createOmniAuth(api.registry, authData) : undefined;
  const wrappedTask = api.createType<NativeTaskWrapper>('NativeTaskWrapper', {
    task,
    nonce: api.createType('Option<Nonce>', nonce),
    auth: api.createType('Option<OmniAuth>', auth),
  });

  if (plain) {
    return api.createType<RawTask>('RawTask', {
      Plain: wrappedTask,
    });
  }

  // generate ephemeral shielding key for the task encryption
  const encryptionKey = await generate();
  const encryptionKeyU8 = await exportKey(encryptionKey);

  // Encrypt the task using the client shielding key
  const encryptionNonce = generateNonce12();
  const { ciphertext: encryptedOperation } = await encrypt(
    {
      cleartext: wrappedTask.toU8a(),
      nonce: encryptionNonce,
    },
    encryptionKey,
  );

  // Describe the encrypted operation as KeyAesOutput
  const encryptedPayload: AesOutput = createAesOutputType(api.registry, {
    ciphertext: u8aToHex(encryptedOperation),
    aad: '0x',
    nonce: encryptionNonce,
  });

  // Encrypt the client shielding key using the enclave public key
  const { ciphertext: encryptedKey } = await enclave.encrypt({ cleartext: encryptionKeyU8 });

  const aesTask = api.createType<AesTask>('AesTask', {
    key: compactAddLength(encryptedKey),
    payload: encryptedPayload,
  });

  return api.createType<RawTask>('RawTask', {
    Aes: aesTask,
  });
}
