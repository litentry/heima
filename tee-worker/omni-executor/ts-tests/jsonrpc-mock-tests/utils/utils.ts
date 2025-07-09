import { sha256 } from 'js-sha256';
import { Address } from 'viem';
import { ClientId } from './constants';
import { privateKeyToAccount } from 'viem/accounts';
import { GetShieldingKeyResponse } from './response-types';
import { hexToU8a, u8aToHex, u8aToString } from '@polkadot/util';
import { base64Encode, base64Trim } from '@polkadot/util-crypto';
import type { HexString, U8aLike } from '@polkadot/util/types';
import { AesOutput } from '@heima-network/api-augment/identity';

export function u8aToBase64Url(value: U8aLike): string {
    return (
        // Remove padding (`=`) from base64
        base64Trim(base64Encode(value))
            // Replace `+` with `-`
            .replace(/\+/g, '-')
            // Replace `/` with `_`
            .replace(/\//g, '_')
    );
}

export function calculateOmniAccount(
    address: Address | string,
    clientId: string = ClientId.Wildmeta,
    identityType: 'evm' | 'solana' = 'evm'
): `0x${string}` {
    // Create input array for hashing - order is important: clientId, oaType, sender
    const inputs: Uint8Array[] = [];

    // First: clientId as raw bytes (not padded to 32)
    const clientIdBytes = new TextEncoder().encode(clientId);
    inputs.push(clientIdBytes);

    // Second: identity type ("evm" or "solana")
    inputs.push(new TextEncoder().encode(identityType));

    // Third: address
    if (identityType === 'evm') {
        // For EVM addresses, remove 0x prefix and convert from hex
        const addressHex = address.slice(2).toLowerCase();
        const addressBytes = new Uint8Array(20); // EVM addresses are 20 bytes
        for (let i = 0; i < addressHex.length; i += 2) {
            addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
        }
        inputs.push(addressBytes);
    } else {
        // For Solana, handle as appropriate
        // This is a simplified version - real implementation would handle base58 for Solana
        // TODO
        inputs.push(new TextEncoder().encode(address));
    }

    const totalLength = inputs.reduce((sum, arr) => sum + arr.length, 0);
    const combined = new Uint8Array(totalLength);
    let offset = 0;
    for (const input of inputs) {
        combined.set(input, offset);
        offset += input.length;
    }

    // Calculate SHA256 hash
    const hash = sha256.array(combined);

    // Convert to hex string
    const result = `0x${hash.map((b) => b.toString(16).padStart(2, '0')).join('')}`;

    // console.log("OmniAccount calculation:", {
    // 	address,
    // 	clientId,
    // 	identityType,
    // 	clientIdBytes: Array.from(inputs[0])
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	identityTypeBytes: Array.from(inputs[1])
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	addressBytes: Array.from(inputs[2])
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	combined: Array.from(combined)
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	result,
    // });

    return result as `0x${string}`;
}

export interface EvmWallet {
    address: Address;
    privateKey: `0x${string}`;
}

export function randomEvmWallet(): EvmWallet {
    const privateKeyBytes = new Uint8Array(32);
    crypto.getRandomValues(privateKeyBytes);

    const privateKey = `0x${Array.from(privateKeyBytes)
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('')}` as `0x${string}`;

    const account = privateKeyToAccount(privateKey);

    return {
        address: account.address,
        privateKey: privateKey,
    };
}

export async function encrypt({
    cleartext,
    shieldingKey,
}: {
    cleartext: Uint8Array;
    shieldingKey: GetShieldingKeyResponse;
}): Promise<{ ciphertext: Uint8Array }> {
    const nHex = shieldingKey.n.startsWith('0x') ? shieldingKey.n.substring(2) : shieldingKey.n;
    const eHex = shieldingKey.e.startsWith('0x') ? shieldingKey.e.substring(2) : shieldingKey.e;
    const jwkData = {
        alg: 'RSA-OAEP-256',
        kty: 'RSA',
        use: 'enc',
        n: u8aToBase64Url(new Uint8Array([...hexToU8a(nHex).reverse()])),
        e: u8aToBase64Url(new Uint8Array([...hexToU8a(eHex).reverse()])),
    };

    const cryptoKey = await crypto.subtle.importKey(
        'jwk',
        jwkData,
        {
            name: 'RSA-OAEP',
            hash: 'SHA-256',
        },
        false,
        ['encrypt']
    );
    const encrypted = await crypto.subtle.encrypt(
        {
            name: 'RSA-OAEP',
        },
        cryptoKey,
        cleartext
    );

    return { ciphertext: new Uint8Array(encrypted) };
}

export async function importKey(key: HexString): Promise<CryptoKey> {
    const secretKey = await crypto.subtle.importKey('raw', hexToU8a(key), { name: 'AES-GCM' }, false, [
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
        const decryptedArrayBuffer = await crypto.subtle.decrypt(
            {
                name: 'AES-GCM',
                iv: nonceBytes,
                additionalData: aadBytes,
                tagLength: tagSize * 8,
            },
            secretKey,
            new Uint8Array([...actualCiphertext, ...authorTag])
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
