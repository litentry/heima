import type { Index } from '@polkadot/types/interfaces';
import type { ApiPromise } from '@polkadot/api';
import { compactAddLength, u8aToHex } from '@polkadot/util';

import type { AesOutput, MrEnclave, NativeCall, OmniAesRequest } from '@litentry/parachain-api';
import { encrypt, generateNonce12, generate, exportKey } from '../util/shielding-key';
import { createKeyAesOutputType } from './key-aes-output';
import { createAuthentication, AuthenticationData } from './authentication';
import { enclave } from '../enclave';

/**
 * Creates a Request struct type for the `TrustedCall` operation.
 *
 * A shielding key is generated and used to encrypt the `TrustedCall` operation and communicated
 * to the enclave to protect the data for transportation.
 *
 * The shielding key is encrypted using the Enclave's shielding key and attached in the Request.
 */
export async function createRequestType(
  api: ApiPromise,
  data: {
    authentication: AuthenticationData;
    call: NativeCall;
    nonce: Index;
    shard: Uint8Array;
  },
): Promise<OmniAesRequest> {
  const { authentication, nonce, call, shard } = data;

  // generate ephemeral shielding key to encrypt the operation
  const encryptionKey = await generate();
  const encryptionKeyU8 = await exportKey(encryptionKey);

  const authenticationValue = createAuthentication(api.registry, authentication);
  const mrenclave = api.createType('MrEnclave', shard);
  const callAuthenticated = api.createType('NativeCallAuthenticated', {
    call,
    nonce,
    authentication: authenticationValue,
  });

  const operationU8a = callAuthenticated.toU8a();

  // Encrypt the operation call using the client shielding key
  const encryptionNonce = generateNonce12();
  const { ciphertext: encryptedOperation } = await encrypt(
    {
      cleartext: operationU8a,
      nonce: encryptionNonce,
    },
    encryptionKey,
  );

  // Describe the encrypted operation as KeyAesOutput
  const encryptedPayload: AesOutput = createKeyAesOutputType(api.registry, {
    ciphertext: u8aToHex(encryptedOperation),
    aad: '0x',
    nonce: encryptionNonce,
  });

  // Encrypt the client shielding key using the enclave public key
  const { ciphertext: encryptedKey } = await enclave.encrypt(encryptionKeyU8);

  return api.createType('OmniAesRequest', {
    mrenclave,
    key: compactAddLength(encryptedKey),
    payload: encryptedPayload,
  }) as unknown as OmniAesRequest;
}
