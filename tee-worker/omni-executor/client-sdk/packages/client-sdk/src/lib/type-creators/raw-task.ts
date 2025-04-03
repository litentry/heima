import type { Index } from '@polkadot/types/interfaces';
import type { ApiPromise } from '@polkadot/api';
import { compactAddLength, u8aToHex } from '@polkadot/util';

import type { AesOutput, AesTask, NativeTask, NativeTaskWrapper, RawTask } from '@heima-network/parachain-api';
import { enclave } from '@lib/enclave';
import { encrypt, generateNonce12, generate, exportKey } from '@utils/shielding-key';
import { createAesOutputType } from './aes-output';
import { createOmniAuth, OmniAuthData } from './omni-auth';

/**
 * Creates a raw task type for unencrypted NativeTask.
 * 
 * @param api - Polkadot API instance
 * @param data - Task configuration
 * @param data.task - Native task to be wrapped
 * @param data.nonce - Optional nonce value
 * @param data.authData - Optional authentication data
 * @param data.plain - Flag to return unencrypted raw task
 * @returns Promise resolving to RawTask
 */
export async function createRawTaskType(
  api: ApiPromise,
  data: {
    task: NativeTask;
    nonce?: Index;
    authData?: OmniAuthData;
    plain: true;
  },
): Promise<RawTask>;

/**
 * Creates an encrypted raw task type.
 * 
 * @param api - Polkadot API instance
 * @param data - Task configuration
 * @param data.task - Native task to be wrapped
 * @param data.nonce - Optional nonce value
 * @param data.authData - Optional authentication data
 * @param data.plain - Flag to return encrypted raw task (default)
 * @returns Promise resolving to object containing encrypted RawTask and encryption key
 */
export async function createRawTaskType(
  api: ApiPromise,
  data: {
    task: NativeTask;
    nonce?: Index;
    authData?: OmniAuthData;
    plain?: false;
  },
): Promise<{ rawTask: RawTask, encryptionKey: CryptoKey }>;

/**
 * Creates a RawTask wrapper for NativeTask with optional encryption.
 * 
 * When plain=true:
 * - Returns unencrypted RawTask directly
 * 
 * When plain=false or omitted:
 * 1. Generates ephemeral shielding key
 * 2. Encrypts the task using client shielding key
 * 3. Encrypts the shielding key using Enclave's public key
 * 4. Packages as AES task
 * 
 * @param api - Polkadot API instance
 * @param data - Task configuration
 * @param data.task - Native task to be wrapped
 * @param data.nonce - Optional nonce value
 * @param data.authData - Optional authentication data
 * @param data.plain - Whether to skip encryption, defaults to false
 * @returns Either RawTask or object with encrypted RawTask and encryption key
 * 
 * @example
 * // Get unencrypted raw task
 * const rawTask = await createRawTaskType(api, { task, plain: true });
 * 
 * @example
 * // Get encrypted raw task with key
 * const { rawTask, encryptionKey } = await createRawTaskType(api, { task });
 */
export async function createRawTaskType(
  api: ApiPromise,
  data: {
    task: NativeTask;
    nonce?: Index;
    authData?: OmniAuthData;
    plain?: boolean;
  },
): Promise<RawTask | { rawTask: RawTask, encryptionKey: CryptoKey }> {
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

  return {
    rawTask: api.createType<RawTask>('RawTask', {
      Aes: aesTask,
    }),
    encryptionKey,
  };
}
