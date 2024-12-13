const aesKeyGenParams: AesKeyGenParams = {
  name: 'AES-GCM',
  length: 256,
};

const aesKeyUsages: KeyUsage[] = ['encrypt', 'decrypt'];

/**
 * Generates a random shielding key.
 */
export async function generate(): Promise<CryptoKey> {
  const key = await globalThis.crypto.subtle.generateKey(
    aesKeyGenParams,
    true,
    aesKeyUsages
  );

  return key;
}

export async function decrypt(
  args: { ciphertext: Uint8Array; nonce: Uint8Array },
  shieldingKey: CryptoKey
): Promise<{ cleartext: Uint8Array }> {
  try {
    const decrypted = await globalThis.crypto.subtle.decrypt(
      {
        name: aesKeyGenParams.name,
        iv: args.nonce,
      },
      shieldingKey,
      args.ciphertext
    );

    return { cleartext: new Uint8Array(decrypted) };
  } catch (e) {
    // It throws Throws OperationError if the ciphertext is invalid
    // We would like to return a more human error.
    throw new Error('Failed to decrypt data');
  }
}

/**
 * Generates a random nonce of 12 bytes.
 *
 * 12 bytes is the recommended size for AES-GCM and what is used by the Enclave in most cases.
 */
export function generateNonce12(): Uint8Array {
  return globalThis.crypto.getRandomValues(new Uint8Array(12));
}

/**
 * Encrypts the given cleartext with the given nonce and shielding key.
 */
export async function encrypt(
  args: { cleartext: Uint8Array; nonce: Uint8Array },
  shieldingKey: CryptoKey
): Promise<{ ciphertext: Uint8Array }> {
  const encrypted = await globalThis.crypto.subtle.encrypt(
    {
      name: aesKeyGenParams.name,
      iv: args.nonce,
    },
    shieldingKey,
    args.cleartext
  );

  return { ciphertext: new Uint8Array(encrypted) };
}

export async function exportKey(cryptoKey: CryptoKey): Promise<Uint8Array> {
  const key = await globalThis.crypto.subtle.exportKey('raw', cryptoKey);
  const hexKey = new Uint8Array(key);

  return hexKey;
}
