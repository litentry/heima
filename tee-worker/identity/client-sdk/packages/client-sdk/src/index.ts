import '@litentry/sidechain-api';
import '@litentry/parachain-api';

export { enclave, Enclave } from './lib/enclave';

/** @namespace requests */
export * as request from './lib/requests';

// type creators
export * from './lib/type-creators/key-aes-output';
export * from './lib/type-creators/litentry-identity';
export * from './lib/type-creators/request';
export * from './lib/type-creators/trusted-call';
export * from './lib/type-creators/validation-data';
export * from './lib/type-creators/tc-authentication';

export type { IdGraph } from './lib/type-creators/id-graph';
export { ID_GRAPH_STRUCT } from './lib/type-creators/id-graph';

// vc
export {
  validateVc,
  type VerifiableCredentialLike,
} from './lib/vc-validator/validator';
export type {
  ValidationResultDetail,
  ValidationResult,
} from './lib/vc-validator/validator.types';

// exposed utils
export { calculateIdGraphHash } from './lib/util/calculate-id-graph-hash';
export { getIdGraphHash } from './lib/requests';
