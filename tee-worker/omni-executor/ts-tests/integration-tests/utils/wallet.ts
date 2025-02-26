import { Signer, EthersSigner, SubstrateSigner, BitcoinSigner, SolanaSigner } from './signer';
import { ethers, Wallet as EvmWallet } from 'ethers';
import { Keyring } from '@polkadot/api';
import { Keypair as SolanaKeypair } from '@solana/web3.js';
import type { KeyringPair } from '@polkadot/keyring/types';
import { ECPairFactory, ECPairInterface } from 'ecpair';
import * as ecc from 'tiny-secp256k1';
import { createHash } from 'crypto';

export interface WalletType {
    [walletName: string]: Signer;
}

export interface Wallets {
    evm: WalletType;
    substrate: WalletType;
    bitcoin: WalletType;
    solana: WalletType;
}

export const createWeb3Wallets = (): Wallets => {
    const wallets: Wallets = {
        evm: {},
        substrate: {},
        bitcoin: {},
        solana: {},
    };
    const walletNames = ['Alice', 'Bob', 'Charlie', 'Dave', 'Eve'];
    for (const name of walletNames) {
        for (const walletType in wallets) {
            (wallets as any)[walletType][name] = createWeb3Wallet(walletType, name);
        }
    }

    return wallets;
};

export const createWeb3Wallet = (walletType: string, walletName: string): Signer => {
    switch (walletType) {
        case 'evm':
            return new EthersSigner(randomEvmWallet());
        case 'substrate':
            return new SubstrateSigner(genesisSubstrateWallet(walletName));
        case 'bitcoin':
            return new BitcoinSigner(randomBitcoinWallet());
        case 'solana':
            return new SolanaSigner(genesisSolanaWallet(walletName));
        default:
            throw new Error(`Unsupported wallet type: ${walletType}`);
    }
};

function randomEvmWallet(): EvmWallet {
    return ethers.Wallet.createRandom();
}

function randomBitcoinWallet(): ECPairInterface {
    const ecPair = ECPairFactory(ecc);
    const keyPair = ecPair.makeRandom();
    return keyPair;
}

function genesisSubstrateWallet(name: string): KeyringPair {
    const keyring = new Keyring({ type: 'sr25519' });
    const keyPair = keyring.addFromUri(`//${name}`, { name });
    return keyPair;
}

function genesisSolanaWallet(name: string): SolanaKeypair {
    let seed = createHash('sha256').update(name).digest();
    seed = seed.subarray(0, 32);
    const keyPair = SolanaKeypair.fromSeed(seed);
    return keyPair;
}
