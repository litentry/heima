export type ChainId =
  | 'heima-local'
  | 'heima-dev'
  | 'heima-prod';

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

export const heimaLocal: ChainSpec = {
  id: 'heima-local',
  name: 'Heima Local Network',
  isTestnet: true,
  isDefault: false,
  rpcs: [{ url: 'ws://localhost:9944' }],
  // On local, we can connect directly to the worker
  enclaveRpcs: [{ url: 'ws://localhost:2100' }],
};

export const heimaDev: ChainSpec = {
  id: 'heima-dev',
  name: 'Heima Development Network',
  isTestnet: true,
  isDefault: false,
  // TODO update url below
  rpcs: [{ url: 'wss://tee-dev.litentry.io' }],
  enclaveRpcs: [{ url: 'wss://enclave-dev.litentry.io' }],
};

export const heimaProd: ChainSpec = {
  id: 'heima-prod',
  name: 'Heima Production Network',
  isTestnet: false,
  isDefault: true,
  rpcs: [
    { url: 'wss://litentry-rpc.dwellir.com' },
    { url: 'wss://rpc.litentry-parachain.litentry.io' },
  ],
  // TODO update url below
  enclaveRpcs: [{ url: 'wss://enclave-prod.litentry.io' }],
};

export const all = [heimaProd, heimaDev, heimaLocal];

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
