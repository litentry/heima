import type { Index } from '@polkadot/types/interfaces';
import type { ApiPromise } from '@polkadot/api';
import { compactAddLength, u8aToHex } from '@polkadot/util';

import type { AesOutput, NativeCall, NativeQuery, AesRequest, PlainRequest } from '@heima/parachain-api';
import { encrypt, generateNonce12, generate, exportKey } from '@utils/shielding-key';
import { createKeyAesOutputType } from './key-aes-output';
import { createAuthentication, AuthenticationData } from './authentication';
import { enclave } from '../enclave';

/**
 * Creates a Request struct type for the `NativeCall` operation.
 *
 * A shielding key is generated and used to encrypt the `TrustedCall` operation and communicated
 * to the enclave to protect the data for transportation.
 *
 * The shielding key is encrypted using the Enclave's shielding key and attached in the Request.
 */
export async function createCallRequestType(
  api: ApiPromise,
  data: {
    authentication: AuthenticationData;
    operation: NativeCall;
    nonce: Index;
    mrEnclave: Uint8Array;
    plain?: boolean;
  },
): Promise<AesRequest | PlainRequest> {
  const { authentication, nonce, operation, mrEnclave, plain = false } = data;

  const authenticationValue = createAuthentication(api.registry, authentication);
  const authenticatedOperation = api.createType('NativeCallAuthenticatedOperation', {
    operation,
    nonce,
    authentication: authenticationValue,
  });

  return createRequestType(api, { operation: authenticatedOperation.toU8a(), mrEnclave, plain });
}

/**
 * Creates a Request struct type for the `NativeQuery` operation.
 *
 * A shielding key is generated and used to encrypt the `TrustedCall` operation and communicated
 * to the enclave to protect the data for transportation.
 *
 * The shielding key is encrypted using the Enclave's shielding key and attached in the Request.
 */
export async function createQueryRequestType(
  api: ApiPromise,
  data: {
    authentication: AuthenticationData;
    operation: NativeQuery;
    nonce: Index;
    mrEnclave: Uint8Array;
    plain?: boolean;
  },
): Promise<AesRequest | PlainRequest> {
  const { authentication, nonce, operation, mrEnclave, plain = false } = data;

  const authenticationValue = createAuthentication(api.registry, authentication);
  const authenticatedOperation = api.createType('NativeQueryAuthenticatedOperation', {
    operation,
    nonce,
    authentication: authenticationValue,
  });

  return createRequestType(api, { operation: authenticatedOperation.toU8a(), mrEnclave, plain });
}

/**
 * Creates a Request struct type for operation.
 *
 * A shielding key is generated and used to encrypt the `NativeCall` or `NativeQuery` operation and communicated
 * to the enclave to protect the data for transportation.
 *
 * The shielding key is encrypted using the Enclave's shielding key and attached in the Request.
 */
async function createRequestType(
  api: ApiPromise,
  data: {
    operation: Uint8Array;
    mrEnclave: Uint8Array;
    plain?: boolean;
  },
): Promise<AesRequest | PlainRequest> {
  const { operation, mrEnclave, plain = false } = data;

  const mrenclave = api.createType('MrEnclave', mrEnclave);
  if (plain) {
    // TODO plain request is invalid, the payload in the created PlainRequest is 0x.
    return api.createType<PlainRequest>('PlainRequest', {
      mrenclave,
      payload: operation,
    });
  }

  // generate ephemeral shielding key to encrypt the operation
  const encryptionKey = await generate();
  const encryptionKeyU8 = await exportKey(encryptionKey);

  // Encrypt the operation call using the client shielding key
  const encryptionNonce = generateNonce12();
  const { ciphertext: encryptedOperation } = await encrypt(
    {
      cleartext: operation,
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
  const { ciphertext: encryptedKey } = await enclave.encrypt({ cleartext: encryptionKeyU8 });

  return api.createType<AesRequest>('AesRequest', {
    mrenclave,
    key: compactAddLength(encryptedKey),
    payload: encryptedPayload,
  });
}
