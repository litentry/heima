import '@litentry/parachain-api';

export { OmniClient, OmniClientConfig, ConnectionState, type ConnectionListener } from './lib/enclave';

// type creators
export * from './lib/type-creators/key-aes-output';
export * from './lib/type-creators/litentry-identity';
export * from './lib/type-creators/validation-data';
