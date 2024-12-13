import '../config';

// in order to handle self-signed certificates we need to turn off the validation
// TODO add self signed certificate ??
process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

export * from './assertion';
export * from './common';
export * from './context';
export * from './crypto';
export * from './identity-helper';
export * from './integration-setup';
export * from './storage';
export * from './vc-helper';
export * from './type-creators';
export * from './native-request-helpers';
