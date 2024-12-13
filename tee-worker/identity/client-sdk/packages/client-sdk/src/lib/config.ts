import { type ChainId, getChain } from '@litentry/chaindata';

const CURRENT_NETWORK =
  process.env.LITENTRY_NETWORK ||
  process.env.NX_LITENTRY_NETWORK ||
  process.env.PARACHAIN_NETWORK ||
  process.env.NX_PARACHAIN_NETWORK ||
  'litentry-prod';

export let ENCLAVE_ENDPOINT = '';

// Custom networks have priority
if (
  CURRENT_NETWORK.startsWith('ws://') ||
  CURRENT_NETWORK.startsWith('wss://')
) {
  ENCLAVE_ENDPOINT = CURRENT_NETWORK;
} else {
  ENCLAVE_ENDPOINT = getChain(CURRENT_NETWORK as ChainId).enclaveRpcs[0].url;
}
