import type { Enum } from '@polkadot/types';

export function codecToString(codec: Enum): string {
  const output = codec.type;

  const value = codec.value?.toHuman();

  if (value == null) {
    return output;
  }

  if (value.toString() === '[object Object]') {
    return `${output}: ${JSON.stringify(value)}`;
  }

  return `${output}: ${value}`;
}
