import type { HexString } from '@polkadot/util/types';
import { bufferToU8a, hexToU8a, isString, stringToU8a, u8aToHex } from '@polkadot/util';
import { KeyObject } from 'crypto';
import { AesOutput, HeimaPrimitivesIdentity } from '@heima-network/api-argument/identity';
import crypto from 'crypto';
import { KeyringPair } from '@polkadot/keyring/types';
import { ethers } from 'ethers';
import { blake2AsU8a } from '@polkadot/util-crypto';
import { Keypair } from '@solana/web3.js';
import nacl from 'tweetnacl';
import { IntegrationTestContext } from './../common-types';
import { buildIdentityHelper } from './identity-helper';
import { ECPairInterface } from 'ecpair';
import * as bitcoinMessage from 'bitcoinjs-message';
import { isHexString } from 'ethers/lib/utils';
export type KeypairType = 'ed25519' | 'sr25519' | 'ecdsa' | 'ethereum' | 'bitcoin';

export function encryptWithTeeShieldingKey(teeShieldingKey: KeyObject, plaintext: Uint8Array): Buffer {
    return encryptBuffer(teeShieldingKey, plaintext);
}

/**
 * Encrypts a plaintext buffer using the provided public key in segments.
 *
 * Same logic as: https://github.com/apache/incubator-teaclave-sgx-sdk/blob/master/sgx_crypto_helper/src/rsa3072.rs#L161-L179
 *
 * @param {crypto.KeyLike} pubKey - The public key to use for encryption.
 * @param {Uint8Array} plaintext - The plaintext buffer to encrypt.
 * @returns {Buffer} The encrypted data.
 */
function encryptBuffer(pubKey: crypto.KeyLike, plaintext: Uint8Array): Buffer {
    const bs = 384; // 3072 bits = 384 bytes
    const bsPlain = bs - (2 * 256) / 8 - 2; // Maximum plaintext block size
    const count = Math.ceil(plaintext.length / bsPlain); // Use Math.ceil to ensure proper chunk count

    const cipherText = Buffer.alloc(bs * count);

    for (let i = 0; i < count; i++) {
        const plainSlice = plaintext.slice(i * bsPlain, Math.min((i + 1) * bsPlain, plaintext.length));
        const cipherSlice = crypto.publicEncrypt(
            {
                key: pubKey,
                padding: crypto.constants.RSA_PKCS1_OAEP_PADDING,
                oaepHash: 'sha256',
            },
            plainSlice
        );

        cipherSlice.copy(cipherText, i * bs);
    }

    return cipherText;
}

// A lazy version without aad. Append the tag to be consistent with rust implementation
export function encryptWithAes(key: HexString, nonce: Uint8Array, cleartext: Buffer): HexString {
    const secretKey = crypto.createSecretKey(hexToU8a(key));
    const cipher = crypto.createCipheriv('aes-256-gcm', secretKey, nonce, {
        authTagLength: 16,
    });
    let encrypted = cipher.update(cleartext.toString('hex'), 'hex', 'hex');
    encrypted += cipher.final('hex');
    encrypted += cipher.getAuthTag().toString('hex');
    return `0x${encrypted}`;
}

export function decryptWithAes(key: HexString, aesOutput: AesOutput, type: 'hex' | 'utf-8'): HexString {
    const secretKey = crypto.createSecretKey(hexToU8a(key));
    const tagSize = 16;
    const ciphertext = aesOutput.ciphertext ? aesOutput.ciphertext : hexToU8a('0x');

    const nonce = aesOutput.nonce ? aesOutput.nonce : hexToU8a('0x');
    const aad = aesOutput.aad ? aesOutput.aad : hexToU8a('0x');

    // notice!!! extract author_tag from ciphertext
    // maybe this code only works with rust aes encryption
    const authorTag = ciphertext.subarray(ciphertext.length - tagSize);

    const decipher = crypto.createDecipheriv('aes-256-gcm', secretKey, nonce, {
        authTagLength: tagSize,
    });
    decipher.setAAD(aad);
    decipher.setAuthTag(authorTag);

    const part1 = decipher.update(ciphertext.subarray(0, ciphertext.length - tagSize), undefined, type);
    const part2 = decipher.final(type);

    return `0x${part1 + part2}`;
}

export interface Signer {
    getAddressRaw(): Uint8Array;
    sign(message: HexString | string | Uint8Array): Promise<Uint8Array>;
    type(): KeypairType;
    getAddressInSubstrateFormat(): Uint8Array;
    getIdentity(api: IntegrationTestContext): Promise<HeimaPrimitivesIdentity>;
}

export class PolkadotSigner implements Signer {
    keypair: KeyringPair;

    constructor(keypair: KeyringPair) {
        this.keypair = keypair;
    }

    getAddressRaw(): Uint8Array {
        return this.keypair.addressRaw;
    }

    sign(message: HexString | string | Uint8Array): Promise<Uint8Array> {
        return new Promise((resolve) => resolve(this.keypair.sign(message)));
    }

    type(): KeypairType {
        return this.keypair.type;
    }

    getAddressInSubstrateFormat(): Uint8Array {
        return this.getAddressRaw();
    }

    getIdentity(context: IntegrationTestContext): Promise<HeimaPrimitivesIdentity> {
        return buildIdentityHelper(u8aToHex(this.getAddressRaw()), 'Substrate', context);
    }
}

export class EthersSigner implements Signer {
    wallet: ethers.Wallet;

    constructor(wallet: ethers.Wallet) {
        this.wallet = wallet;
    }

    getAddressRaw(): Uint8Array {
        return hexToU8a(this.wallet.address);
    }

    sign(message: HexString | string | Uint8Array): Promise<Uint8Array> {
        return this.wallet.signMessage(message).then((sig) => {
            return hexToU8a(sig);
        });
    }

    type(): KeypairType {
        return 'ethereum';
    }

    getAddressInSubstrateFormat(): Uint8Array {
        const prefix = stringToU8a('evm:');
        const address = this.getAddressRaw();
        const merged = new Uint8Array(prefix.length + address.length);
        merged.set(prefix);
        merged.set(address, 4);
        return blake2AsU8a(merged, 256);
    }

    getIdentity(context: IntegrationTestContext): Promise<HeimaPrimitivesIdentity> {
        return buildIdentityHelper(u8aToHex(this.getAddressRaw()), 'Evm', context);
    }
}

export class BitcoinSigner implements Signer {
    keypair: ECPairInterface;

    constructor(keypair: ECPairInterface) {
        this.keypair = keypair;
    }

    getAddressRaw(): Uint8Array {
        return bufferToU8a(this.keypair.publicKey);
    }

    sign(message: HexString | string | Uint8Array): Promise<Uint8Array> {
        return new Promise((resolve, reject) => {
            if (isString(message)) {
                // produce deterministic signatures
                const sig = bitcoinMessage.sign(message, this.keypair.privateKey!, this.keypair.compressed);
                resolve(sig);
            } else {
                reject('wrong message type');
            }
        });
    }

    type(): KeypairType {
        return 'bitcoin';
    }

    getAddressInSubstrateFormat(): Uint8Array {
        return blake2AsU8a(this.getAddressRaw(), 256);
    }

    getIdentity(context: IntegrationTestContext): Promise<HeimaPrimitivesIdentity> {
        return buildIdentityHelper(u8aToHex(this.getAddressRaw()), 'Bitcoin', context);
    }
}

export class SolanaSigner implements Signer {
    keypair: Keypair;

    constructor(keypair: Keypair) {
        this.keypair = keypair;
    }

    getAddressRaw(): Uint8Array {
        return this.keypair.publicKey.toBytes();
    }

    sign(message: HexString | string | Uint8Array): Promise<Uint8Array> {
        return new Promise((resolve) =>
            resolve(
                nacl.sign.detached(
                    isHexString(message)
                        ? hexToU8a(message as HexString)
                        : isString(message)
                        ? stringToU8a(message)
                        : message,
                    this.keypair.secretKey
                )
            )
        );
    }

    type(): KeypairType {
        return 'ed25519';
    }

    getAddressInSubstrateFormat(): Uint8Array {
        return this.getAddressRaw();
    }

    getIdentity(context: IntegrationTestContext): Promise<HeimaPrimitivesIdentity> {
        return buildIdentityHelper(u8aToHex(this.getAddressRaw()), 'Solana', context);
    }
}
