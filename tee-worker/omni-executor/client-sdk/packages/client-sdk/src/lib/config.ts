import { type ChainId, getChain } from '@heima-network/chaindata';

const DEFAULT_NETWORK = 'heima-prod';

let CURRENT_NETWORK = DEFAULT_NETWORK;
try {
  CURRENT_NETWORK =
    process.env.HEIMA_NETWORK ||
    process.env.NX_HEIMA_NETWORK ||
    process.env.PARACHAIN_NETWORK ||
    process.env.NX_PARACHAIN_NETWORK ||
    DEFAULT_NETWORK;
} catch (error) {
  // Fallback to default network if any error occurs, e.g. non-node environment or environment pattern is not been replaced
  CURRENT_NETWORK = DEFAULT_NETWORK;
}

export let ENCLAVE_ENDPOINT = '';

// Custom networks have priority
if (CURRENT_NETWORK.startsWith('ws://') || CURRENT_NETWORK.startsWith('wss://')) {
  ENCLAVE_ENDPOINT = CURRENT_NETWORK;
} else {
  ENCLAVE_ENDPOINT = getChain(CURRENT_NETWORK as ChainId).enclaveRpcs[0].url;
}
