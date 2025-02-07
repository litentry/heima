// augment on-chain lookup types
import "../build/interfaces/types-lookup.js";

// augment types for createType(...)
import "../build/interfaces/augment-types.js";
import "../build/interfaces/registry.js";

// augment API interfaces
import "../build/interfaces/augment-api.js";

export * from "@polkadot/types/lookup";
export * from "../build/interfaces";
export * from "@polkadot/api";
export * from "@polkadot/api/types";
import { default as identity } from "../build/interfaces/identity/definitions";
import { default as vc } from "../build/interfaces/vc/definitions";
import { default as omniAccount } from "../build/interfaces/omniAccount/definitions";
import { default as trusted_operations } from "../build/interfaces/trusted_operations/definitions";
import { default as sidechain } from "../build/interfaces/sidechain/definitions";
export { identity, vc, trusted_operations, sidechain, omniAccount };

// Export handy types
import type { LitentryIdentity, Web3Network } from "../build/interfaces/identity/types";

export type SubstrateNetwork = Extract<
    Web3Network["type"],
    "Polkadot" | "Kusama" | "Litentry" | "Litmus" | "LitentryRococo" | "Khala" | "SubstrateTestnet"
>;

export type EvmNetwork = Extract<Web3Network["type"], "Ethereum" | "Bsc" | "Polygon" | "Arbitrum" | "Combo">;

export type SolanaNetwork = Extract<Web3Network["type"], "Solana">;

export type BitcoinNetwork = Exclude<Web3Network["type"], SubstrateNetwork | EvmNetwork | SolanaNetwork>;

export type Web2Network = Exclude<LitentryIdentity["type"], "Substrate" | "Evm" | "Bitcoin" | "Solana">;

/**
 * Identities that can be used as prime identity to own an idGraph.
 */
export type PrimeIdentity = Extract<LitentryIdentity["type"], "Substrate" | "Evm" | "Bitcoin" | "Solana">;
