// augment on-chain lookup types
import "./interfaces/types-lookup.js";

// augment types for createType(...)
import "./interfaces/augment-types.js";
import "./interfaces/registry.js";

// augment API interfaces
import "./interfaces/augment-api.js";

export * from "@polkadot/types/lookup";
export * from "./interfaces";

import { default as omniAccount } from "./interfaces/omniAccount/definitions";
import { default as omniExecutor } from "./interfaces/omniExecutor/definitions";
export { omniAccount, omniExecutor };


