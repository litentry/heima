// augment on-chain lookup types
import "../build/interfaces/types-lookup.js";

// augment types for createType(...)
import "../build/interfaces/augment-types.js";
import "../build/interfaces/registry.js";

// augment API interfaces
import "../build/interfaces/augment-api.js";

export * from "@polkadot/types/lookup";
export * from "../build/interfaces/index.js";
export * from "@polkadot/api";
export * from "@polkadot/api/types";
import { default as identity } from "../build/interfaces/identity/definitions.js";
import { default as omniAccount } from "../build/interfaces/omniAccount/definitions.js";
import { default as omniExecutor } from "../build/interfaces/omniExecutor/definitions.js";
export { identity, omniAccount, omniExecutor };
