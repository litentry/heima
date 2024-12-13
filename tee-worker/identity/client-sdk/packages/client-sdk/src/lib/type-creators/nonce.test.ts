import { TypeRegistry } from '@polkadot/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import { createNonceType } from './nonce';
import { WorkerRpcReturnValue } from '@litentry/parachain-api';

const types = {
  // No custom types are needed for Index
};

let registry: TypeRegistry;

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);
});

const mockWorkerRpcReturnValue = (value: string) =>
  ({
    value: {
      toHex: () => value,
    },
  } as WorkerRpcReturnValue);

describe('Nonce decoder', () => {
  const table: Array<{ data: string; value: number }> = [
    { data: '0x01000000', value: 1 },
    { data: '0x08000000', value: 8 },
    { data: '0x06000000', value: 6 },
    { data: '0x02000000', value: 2 },
  ];

  it.each(table)(
    'correctly decodes $data into $value',
    async ({ data, value }) => {
      expect(
        createNonceType(registry, {
          workerRpcReturnValue: mockWorkerRpcReturnValue(data),
        }).toNumber()
      ).toBe(value);
    }
  );
});
