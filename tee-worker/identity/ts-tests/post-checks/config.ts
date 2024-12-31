import { byId, ChainId } from '@litentry/chaindata';

// Change this to the environment you want to test
const network = process.env.LITENTRY_NETWORK;
const chain = byId[network as ChainId];

export const nodeEndpoint: string = chain.rpcs[0].url;
export const enclaveEndpoint: string = chain.enclaveRpcs[0].url;
