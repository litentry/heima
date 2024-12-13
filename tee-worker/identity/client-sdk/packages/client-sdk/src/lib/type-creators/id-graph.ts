import type { ITuple, Registry } from '@polkadot/types-codec/types';
import type { Vec } from '@polkadot/types-codec';
import type { U8aLike } from '@polkadot/util/types';

import type {
  IdentityContext,
  LitentryIdentity,
} from '@litentry/parachain-api';

/**
 * The Identity Graph type
 */
export type IdGraph = Vec<ITuple<[LitentryIdentity, IdentityContext]>>;

/**
 * The Type Struct that represents an Identity Graph
 */
export const ID_GRAPH_STRUCT = 'Vec<(LitentryIdentity, IdentityContext)>';

/**
 * Creates an IdGraph type
 */
export function createIdGraphType(
  registry: Registry,
  data: U8aLike | unknown[]
): IdGraph {
  return registry.createType(ID_GRAPH_STRUCT, data);
}
