import type { Index } from '@polkadot/types/interfaces';
import type { ApiPromise } from '@polkadot/api';
import { compactAddLength, u8aToHex } from '@polkadot/util';

import { enclave } from '../enclave';

import type {
  TrustedCall,
  LitentryIdentity,
  AesRequest,
  AesOutput,
} from '@litentry/parachain-api';
import {
  encrypt,
  generateNonce12,
  generate,
  exportKey,
} from '../util/shielding-key';
import { createKeyAesOutputType } from './key-aes-output';
import { createLitentryMultiSignature } from './litentry-multi-signature';

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
    signer: LitentryIdentity;
    signature: string;
    call: TrustedCall;
    nonce: Index;
    shard: Uint8Array;
  }
): Promise<AesRequest> {
  const { signer, shard, signature: signedPayload, nonce, call } = data;

  // generate ephemeral shielding key to encrypt the operation
  const encryptionKey = await generate();
  const encryptionKeyU8 = await exportKey(encryptionKey);

  const signature = createLitentryMultiSignature(api.registry, {
    who: signer,
    signature: signedPayload,
  });

  let operationU8a = new Uint8Array();

  if (call.isRequestIntent) {
    const callAuthenticated = api.createType('TrustedCallAuthenticated', {
      call,
      nonce,
      authentication: api.createType('TCAuthentication', {
        Web3: signature,
      }),
    });

    const operation = api.createType('TrustedOperation', {
      direct_call: callAuthenticated,
    });

    operationU8a = operation.toU8a();
  } else {
    const signedCall = api.createType('TrustedCallSigned', {
      call,
      index: nonce,
      signature: signature,
    });

    const operation = api.createType('TrustedOperation', {
      direct_call: signedCall,
    });

    operationU8a = operation.toU8a();
  }

  // Encrypt the operation call using the client shielding key
  const encryptionNonce = generateNonce12();
  const { ciphertext: encryptedOperation } = await encrypt(
    {
      cleartext: operationU8a,
      nonce: encryptionNonce,
    },
    encryptionKey
  );

  // Describe the encrypted operation as KeyAesOutput
  const encryptedPayload: AesOutput = createKeyAesOutputType(api.registry, {
    ciphertext: u8aToHex(encryptedOperation),
    aad: '0x',
    nonce: encryptionNonce,
  });

  // Encrypt the client shielding key using the enclave public key
  const { ciphertext: encryptedKey } = await enclave.encrypt(api, {
    cleartext: encryptionKeyU8,
  });

  return api.createType('AesRequest', {
    shard,
    key: compactAddLength(encryptedKey),
    payload: encryptedPayload,
  });
}
