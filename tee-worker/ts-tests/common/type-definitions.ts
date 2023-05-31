import { ApiPromise } from '@polkadot/api';
import { KeyObject } from 'crypto';
import { HexString } from '@polkadot/util/types';
import WebSocketAsPromised from 'websocket-as-promised';
import type { KeyringPair } from '@polkadot/keyring/types';
import { ApiTypes, SubmittableExtrinsic } from '@polkadot/api/types';
import { Metadata, Vec, TypeRegistry } from '@polkadot/types';
import { Wallet } from 'ethers';
import type { Assertion as GenericAssertion, DirectRequestStatus } from '../parachain-interfaces/identity/types';
import { default as teeTypes } from '../parachain-interfaces/identity/definitions';
import { AnyTuple, IMethod } from '@polkadot/types/types';
import { Call } from '@polkadot/types/interfaces';
import type {
    LitentryPrimitivesIdentitySubstrateNetwork,
    LitentryPrimitivesIdentityEvmNetwork,
    LitentryPrimitivesIdentityWeb2Network,
    PalletIdentityManagementTeeIdentityContext,
    LitentryPrimitivesIdentity,
} from '@polkadot/types/lookup';
export { teeTypes };

export type Web2Network = LitentryPrimitivesIdentityWeb2Network['type'];
export type SubstrateNetwork = LitentryPrimitivesIdentitySubstrateNetwork['type'];
export type EvmNetwork = LitentryPrimitivesIdentityEvmNetwork['type'];
export type ParachainAssertion = GenericAssertion['type'];
export type WorkerRpcReturnString = {
    vec: string;
};

export type BatchCall = Vec<Call> | (string | Uint8Array | IMethod<AnyTuple, any> | Call)[];
export type WorkerRpcReturnValue = {
    value: `0x${string}`;
    do_watch: boolean;
    status: DirectRequestStatus;
};
export type EnclaveResult = {
    mrEnclave: `0x${string}`;
    shieldingKey: `0x${string}`;
    vcPubkey: `0x${string}`;
    sgxMetadata: {};
};
export type PubicKeyJson = {
    n: Uint8Array;
    e: Uint8Array;
};

interface EthersWalletItem {
    [key: string]: Wallet;
}
interface SubstrateWalletItem {
    [key: string]: KeyringPair;
}
export type IntegrationTestContext = {
    tee: WebSocketAsPromised;
    api: ApiPromise;
    teeShieldingKey: KeyObject;
    mrEnclave: HexString;
    ethersWallet: EthersWalletItem;
    substrateWallet: SubstrateWalletItem;
    metaData: Metadata;
    sidechainRegistry: TypeRegistry;
    web3Signers: Web3Wallets[];
};

export class AESOutput {
    ciphertext?: Uint8Array;
    aad?: Uint8Array;
    nonce?: Uint8Array;
}

export type Web3Wallets = {
    substrateWallet: KeyringPair;
    ethereumWallet: Wallet;
};

export type Web3Network = {
    Substrate?: SubstrateNetwork;
    Evm?: EvmNetwork;
};

export type IdentityGenericEvent = {
    who: HexString;
    identity: LitentryPrimitivesIdentity;
    idGraph: [LitentryPrimitivesIdentity, PalletIdentityManagementTeeIdentityContext][];
    challengeCode?: HexString;
};

//vc types
export type VCRequested = {
    account: HexString;
    mrEnclave: HexString;
    assertion: Assertion;
};

export enum IndexingNetwork {
    Litentry = 'Litentry',
    Litmus = 'Litmus',
    Polkadot = 'Polkadot',
    Kusama = 'Kusama',
    Khala = 'Khala',
    Ethereum = 'Ethereum',
}

export type Assertion = {
    A1?: string;
    A2?: [string];
    A3?: [string, string, string];
    A4?: string;
    A5?: [string, string];
    A6?: string;
    A7?: string;
    A8?: [IndexingNetwork];
    A9?: string;
    A10?: string;
    A11?: string;
};

export type TransactionSubmit = {
    tx: SubmittableExtrinsic<ApiTypes>;
    nonce: number;
};

//call types
export type RequestBody = {
    id: number;
    jsonrpc: string;
    method: string;
};

export const JsonSchema = {
    type: 'object',
    properties: {
        id: {
            type: 'string',
        },
        type: {
            type: 'array',
        },
        issuer: {
            type: 'object',
            properties: {
                id: {
                    type: 'string',
                },
                name: {
                    type: 'string',
                },
                shard: {
                    type: 'string',
                },
            },
        },
        issuanceTimestamp: {
            type: 'integer',
        },
        credentialSubject: {
            type: 'object',
            properties: {
                id: {
                    type: 'string',
                },
                description: {
                    type: 'string',
                },
                type: {
                    type: 'string',
                },
                tag: {
                    type: 'array',
                },
                assertions: {
                    type: 'array',
                    items: {
                        type: 'object',
                    },
                },
                values: {
                    type: 'array',
                    items: {
                        type: 'boolean',
                    },
                },
                endpoint: {
                    type: 'string',
                },
            },
            required: ['id', 'description', 'type', 'assertions', 'values', 'endpoint'],
        },
        proof: {
            type: 'object',
            properties: {
                createdTimestamp: {
                    type: 'integer',
                },
                type: {
                    enum: ['Ed25519Signature2020'],
                },
                proofPurpose: {
                    enum: ['assertionMethod'],
                },
                proofValue: {
                    type: 'string',
                },
                verificationMethod: {
                    type: 'string',
                },
            },
        },
    },
    required: ['id', 'type', 'credentialSubject', 'issuer', 'issuanceTimestamp', 'proof'],
};
