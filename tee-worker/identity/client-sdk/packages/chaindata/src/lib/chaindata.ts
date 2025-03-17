export type ChainId =
  | 'litentry-local'
  | 'litentry-dev'
  | 'litentry-staging'
  | 'litentry-prod';

export type ChainSpec = {
  id: ChainId;
  name: string;
  isTestnet: boolean;
  isDefault: boolean;
  rpcs: Array<{
    url: string;
  }>;
  enclaveRpcs: Array<{
    url: string;
  }>;
};

export const litentryLocal: ChainSpec = {
  id: 'litentry-local',
  name: 'Litentry Local Network',
  isTestnet: true,
  isDefault: false,
  rpcs: [{ url: 'ws://localhost:9944' }],
  // On local, we can connect directly to the worker
  enclaveRpcs: [{ url: 'ws://localhost:2000' }],
};

export const litentryDev: ChainSpec = {
  id: 'litentry-dev',
  name: 'Litentry Development Network',
  isTestnet: true,
  isDefault: false,
  rpcs: [{ url: 'wss://tee-dev.litentry.io' }],
  enclaveRpcs: [{ url: 'wss://enclave-dev.litentry.io' }],
};

export const litentryStaging: ChainSpec = {
  id: 'litentry-staging',
  name: 'Litentry Staging Network',
  isTestnet: true,
  isDefault: false,
  rpcs: [{ url: 'wss://tee-staging.litentry.io' }], // Parachain + Enclave (TEE}
  enclaveRpcs: [{ url: 'wss://enclave-staging.litentry.io' }],
};

export const litentryProd: ChainSpec = {
  id: 'litentry-prod',
  name: 'Litentry Pre Production Network',
  isTestnet: false,
  isDefault: true,
  rpcs: [
    { url: 'wss://litentry-rpc.dwellir.com' },
    { url: 'wss://rpc.heima-parachain.heima.network' },
  ],
  enclaveRpcs: [{ url: 'wss://enclave-prod.litentry.io' }],
};

export const all = [litentryProd, litentryStaging, litentryDev, litentryLocal];

export const byId = all.reduce((acc, spec) => {
  acc[spec.id] = spec;
  return acc;
}, {} as Record<ChainId, ChainSpec>);

export type GetChainOptions = {
  throw?: boolean;
  allowDefault?: boolean;
};

export const getChain = (
  id: ChainId | string | null | undefined
): ChainSpec => {
  if (!id) {
    throw new Error(`Chain id is required. Got: ${id}`);
  }

  const spec = byId[id as ChainId];

  if (!spec) {
    throw new Error(
      `Unknown chain id: ${id}. Available chains: ${Object.keys(byId).join(
        ', '
      )}`
    );
  }

  return spec;
};
