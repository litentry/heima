import { xxhashAsU8a, blake2AsU8a } from '@polkadot/util-crypto';
import { u8aConcat, u8aToU8a } from '@polkadot/util';
import { Keyring } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { HexString } from '@polkadot/util/types';
import './config';
import { IntegrationTestContext, JsonRpcRequest } from './common-types';
import { createHash, randomBytes, type KeyObject } from 'crypto';
import { ECPairFactory, ECPairInterface } from 'ecpair';
import * as ecc from 'tiny-secp256k1';
import { ethers, Wallet } from 'ethers';
import { Keypair } from '@solana/web3.js';
import { EthersSigner, PolkadotSigner, BitcoinSigner, SolanaSigner, Signer } from './utils/crypto';
import { Wallets } from './common-types';
import type { ErrorDetail, StfError } from 'parachain-api';
import { createSignedTrustedCallCleanIDGraphs, getSidechainNonce, sendRequestFromTrustedCall } from './di-utils';

export function blake2128Concat(data: HexString | Uint8Array): Uint8Array {
    return u8aConcat(blake2AsU8a(data, 128), u8aToU8a(data));
}

export function twox64Concat(data: HexString | Uint8Array): Uint8Array {
    return u8aConcat(xxhashAsU8a(data, 64), u8aToU8a(data));
}

export function identity(data: HexString | Uint8Array): Uint8Array {
    return u8aToU8a(data);
}

export function createJsonRpcRequest(method: string, params: any, id: number): JsonRpcRequest {
    return {
        jsonrpc: '2.0',
        method,
        params,
        id,
    };
}

export function nextRequestId(context: IntegrationTestContext): number {
    const nextId = context.requestId + 1;
    context.requestId = nextId;
    return nextId;
}

export function randomEvmWallet(): Wallet {
    return ethers.Wallet.createRandom();
}
export function randomSubstrateWallet(): KeyringPair {
    const keyring = new Keyring({ type: 'sr25519' });
    return keyring.addFromSeed(randomBytes(32));
}

export function randomBitcoinWallet(): ECPairInterface {
    const ecPair = ECPairFactory(ecc);
    const keyPair = ecPair.makeRandom();
    return keyPair;
}

export function genesisSubstrateWallet(name: string): KeyringPair {
    const keyring = new Keyring({ type: 'sr25519' });
    const keyPair = keyring.addFromUri(`//${name}`, { name });
    return keyPair;
}

export function genesisSolanaWallet(name: string): Keypair {
    let seed = createHash('sha256').update(name).digest();
    seed = seed.subarray(0, 32);
    const keyPair = Keypair.fromSeed(seed);
    return keyPair;
}

export const createWeb3Wallet = (walletType: string, walletName: string): Signer => {
    switch (walletType) {
        case 'evm':
            return new EthersSigner(randomEvmWallet());
        case 'substrate':
            return new PolkadotSigner(genesisSubstrateWallet(walletName));
        case 'bitcoin':
            return new BitcoinSigner(randomBitcoinWallet());
        case 'solana':
            return new SolanaSigner(genesisSolanaWallet(walletName));
        default:
            throw new Error(`Unsupported wallet type: ${walletType}`);
    }
};

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

export function stfErrorToString(stfError: StfError): string {
    if (stfError.isRequestVCFailed) {
        const [_assertionIgnored, errorDetail] = stfError.asRequestVCFailed;

        return `${stfError.type}: ${errorDetail.type}: ${errorDetail.value?.toHuman()}`;
    }

    if (
        stfError.isActivateIdentityFailed ||
        stfError.isDeactivateIdentityFailed ||
        stfError.isSetIdentityNetworksFailed ||
        stfError.isLinkIdentityFailed ||
        stfError.isMissingPrivileges ||
        stfError.isRemoveIdentityFailed ||
        stfError.isDispatch
    ) {
        const errorDetail = stfError.value as ErrorDetail;

        return `${stfError.type}: ${errorDetail.type}: ${errorDetail.value?.toHuman()}`;
    }

    if (stfError.isInvalidNonce) {
        const [nonce1, nonce2] = stfError.asInvalidNonce;

        return `${stfError.type}: [${nonce1?.toHuman()}, ${nonce2?.toHuman()}]`;
    }

    return stfError.type;
}
