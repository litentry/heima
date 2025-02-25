import '@litentry/sidechain-api';

export { OmniClient, OmniClientConfig, ConnectionState, type ConnectionListener } from './lib/omni-client';

// type creators
export * from './lib/type-creators/key-aes-output';
export * from './lib/type-creators/litentry-identity';
export * from './lib/type-creators/validation-data';
