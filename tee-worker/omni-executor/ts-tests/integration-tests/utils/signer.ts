import type { HexString } from '@polkadot/util/types';

import { bufferToU8a, hexToU8a, isString, stringToU8a, u8aToHex } from '@polkadot/util';
import { KeyringPair } from '@polkadot/keyring/types';
import { blake2AsU8a } from '@polkadot/util-crypto';
import { ApiPromise } from '@polkadot/api';
import { CorePrimitivesIdentity } from 'parachain-api';
import { ethers } from 'ethers';
import { isHexString } from 'ethers/lib/utils';
import { ECPairInterface } from 'ecpair';
import { Keypair as SolanaKeyPair } from '@solana/web3.js';
import * as bitcoinMessage from 'bitcoinjs-message';
import nacl from 'tweetnacl';

import { createIdentityType } from './type_creators';

export type KeypairType = 'ed25519' | 'sr25519' | 'ecdsa' | 'ethereum' | 'bitcoin';

export interface Signer {
    getAddressRaw(): Uint8Array;
    sign(message: HexString | string | Uint8Array): Promise<Uint8Array>;
    type(): KeypairType;
    getAddressInSubstrateFormat(): Uint8Array;
    getIdentity(api: ApiPromise): Promise<CorePrimitivesIdentity>;
}

export class SubstrateSigner implements Signer {
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

    getIdentity(api: ApiPromise): Promise<CorePrimitivesIdentity> {
        return createIdentityType(api, u8aToHex(this.getAddressRaw()), 'Substrate');
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

    async sign(message: HexString | string | Uint8Array): Promise<Uint8Array> {
        let signature = await this.wallet.signMessage(message);
        return hexToU8a(signature);
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

    getIdentity(api: ApiPromise): Promise<CorePrimitivesIdentity> {
        return createIdentityType(api, u8aToHex(this.getAddressRaw()), 'Evm');
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

    getIdentity(api: ApiPromise): Promise<CorePrimitivesIdentity> {
        return createIdentityType(api, u8aToHex(this.getAddressRaw()), 'Bitcoin');
    }
}

export class SolanaSigner implements Signer {
    keypair: SolanaKeyPair;

    constructor(keypair: SolanaKeyPair) {
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

    getIdentity(api: ApiPromise): Promise<CorePrimitivesIdentity> {
        return createIdentityType(api, u8aToHex(this.getAddressRaw()), 'Solana');
    }
}
