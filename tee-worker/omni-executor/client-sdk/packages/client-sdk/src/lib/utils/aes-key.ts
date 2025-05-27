import { compactStripLength, hexToU8a } from '@polkadot/util';
import { assert, isHex, u8aToHex, u8aToString, u8aToU8a } from '@polkadot/util';
import type { HexString } from '@polkadot/util/types';
import type { AesOutput } from '@heima-network/parachain-api';



export async function importKey(key: HexString): Promise<CryptoKey> {
  const secretKey = await globalThis.crypto.subtle.importKey('raw', hexToU8a(key), { name: 'AES-GCM' }, false, [
    'encrypt',
    'decrypt',
  ]);
  return secretKey;
}

export async function decryptWithAes(key: HexString, aesOutput: AesOutput, type: 'hex' | 'utf-8'): Promise<HexString> {
  const secretKey = await importKey(key);
  const tagSize = 16;

  // Ensure ciphertext is a Uint8Array
  const ciphertextBytes = aesOutput.ciphertext
    ? aesOutput.ciphertext instanceof Uint8Array
      ? aesOutput.ciphertext
      : hexToU8a(aesOutput.ciphertext)
    : hexToU8a('0x');

  // Ensure nonce and aad are Uint8Arrays
  const nonceBytes = aesOutput.nonce
    ? aesOutput.nonce instanceof Uint8Array
      ? aesOutput.nonce
      : hexToU8a(aesOutput.nonce)
    : hexToU8a('0x');

  const aadBytes = aesOutput.aad
    ? aesOutput.aad instanceof Uint8Array
      ? aesOutput.aad
      : hexToU8a(aesOutput.aad)
    : hexToU8a('0x');

  // Extract auth tag from ciphertext (last 16 bytes)
  const authorTag = ciphertextBytes.subarray(ciphertextBytes.length - tagSize);
  const actualCiphertext = ciphertextBytes.subarray(0, ciphertextBytes.length - tagSize);

  try {
    // Decrypt the data
    const decryptedArrayBuffer = await globalThis.crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: nonceBytes,
        additionalData: aadBytes,
        tagLength: tagSize * 8,
      },
      secretKey,
      new Uint8Array([...actualCiphertext, ...authorTag]),
    );

    // Convert the result based on the requested type
    const decryptedBytes = new Uint8Array(decryptedArrayBuffer);
    if (type === 'hex') {
      return `0x${u8aToHex(decryptedBytes).substring(2)}`;
    } else {
      return `0x${u8aToString(decryptedBytes)}`;
    }
  } catch (error) {
    console.error('Decryption error:', error);
    throw new Error('Failed to decrypt data');
  }
}
