import { ApiPromise } from 'parachain-api';
import { KeyObject } from 'crypto';
import WebSocketAsPromised from 'websocket-as-promised';
import { Metadata, TypeRegistry } from '@polkadot/types';
import { Wallet } from 'ethers';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { HexString } from '@polkadot/util/types';
import { ECPairInterface } from 'ecpair';
import { Keypair } from '@solana/web3.js';
import { Signer, PolkadotSigner } from './utils/crypto';
// If there are types already defined in the client-api, please avoid redefining these types.
// Instead, make every effort to use the types that have been generated within the client-api.

export type SubstrateSigner = PolkadotSigner;

export interface WalletType {
    [walletName: string]: Signer;
}
export interface Wallets {
    evm: WalletType;
    substrate: WalletType;
    bitcoin: WalletType;
    solana: WalletType;
}
export type IntegrationTestContext = {
    tee: WebSocketAsPromised;
    api: ApiPromise;
    teeShieldingKey: KeyObject;
    mrEnclave: HexString;
    web3Wallets: Wallets;
    sidechainMetaData: Metadata;
    sidechainRegistry: TypeRegistry;
    chainIdentifier: number;
    requestId: number;
};

export type Web3Wallets = {
    substrateWallet: KeyringPair;
    evmWallet: Wallet;
    bitcoinWallet: ECPairInterface;
    solanaWallet: Keypair;
};

export type JsonRpcRequest = {
    jsonrpc: string;
    method: string;
    params: any;
    id: number;
};
