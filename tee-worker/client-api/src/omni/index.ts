// augment on-chain lookup types
import "./interfaces/types-lookup.js";

// augment types for createType(...)
import "./interfaces/augment-types.js";
import "./interfaces/registry.js";

// augment API interfaces
import "./interfaces/augment-api.js";

export * from "@polkadot/types/lookup";
export * from "./interfaces/index.js";

import { default as omniAccount } from "./interfaces/omniAccount/definitions.js";
import { default as omniExecutor } from "./interfaces/omniExecutor/definitions.js";
import { default as identity } from "./interfaces/identity/definitions.js";
import { default as trusted_operations } from "./interfaces/trusted_operations/definitions.js";
export { omniAccount, omniExecutor, identity, trusted_operations };
