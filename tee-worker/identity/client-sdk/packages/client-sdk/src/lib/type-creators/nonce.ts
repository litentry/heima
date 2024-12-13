import type { Registry } from '@polkadot/types-codec/types';
import type { WorkerRpcReturnValue } from '@litentry/parachain-api';
import { Index } from '@polkadot/types/interfaces';
import { hexToU8a } from '@polkadot/util';

export function createNonceType(
  registry: Registry,
  data: {
    workerRpcReturnValue: WorkerRpcReturnValue;
  }
): Index {
  return registry.createType(
    'Index',
    // Heads-up: We need to use `hexToU8a(hexValue)` here to get the encoding right.
    // transmitted data comes scale encoded (little-endian for basic integers ).
    // hexToU8a(code.toHex()) removes the scale encoding. Similar to byte swapping.
    // Notice that code.toU8a() will work but produce wrong values.
    hexToU8a(data.workerRpcReturnValue.value.toHex())
  );
}
