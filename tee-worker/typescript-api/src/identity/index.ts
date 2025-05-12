// augment on-chain lookup types
import "./interfaces/types-lookup.js";

// augment types for createType(...)
import "./interfaces/augment-types.js";
import "./interfaces/registry.js";

// augment API interfaces
import "./interfaces/augment-api.js";

export * from "@polkadot/types/lookup";
export * from "./interfaces";
import { default as identity } from "./interfaces/identity/definitions";
import { default as vc } from "./interfaces/vc/definitions";
import { default as trusted_operations } from "./interfaces/trusted_operations/definitions";
import { default as sidechain } from "./interfaces/sidechain/definitions";
export { identity, vc, trusted_operations, sidechain };

// Export handy types
import type { LitentryIdentity, Web3Network } from "./interfaces/identity/types";

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
