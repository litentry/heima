import { hexToU8a } from '@polkadot/util';
import type { Registry, Codec } from '@polkadot/types-codec/types';
import type { DetectCodec } from '@polkadot/types/types';

/**
 * Safely decodes an `Option<value>` from `WorkerRpcReturnValue`'s opaque responses.
 */
export function safelyDecodeOption<Type extends string>(
  registry: Registry,
  args: { type: Type; value: string; throw?: undefined | true }
): DetectCodec<Codec, Type>;
export function safelyDecodeOption<Type extends string>(
  registry: Registry,
  args: { type: Type; value: string; throw?: false }
): DetectCodec<Codec, Type> | null;
export function safelyDecodeOption<Type extends string>(
  registry: Registry,
  args: { type: Type; value: string; throw?: boolean }
): DetectCodec<Codec, Type> | null {
  const { type, value } = args;
  const shouldThrow = typeof args.throw === 'undefined' ? false : args.throw;

  // Heads-up: We need to use `hexToU8a(hexValue)` here to get the encoding right.
  // `codec.toHex()` or `codec.toU8a()` will work but produce wrong values.
  const maybeValue = registry.createType('Option<Bytes>', hexToU8a(value));

  if (maybeValue.isNone) {
    if (shouldThrow) {
      throw new Error(
        `[safelyDecodeOption]: Found empty value for type '${type}' in '<Option<${type}>>'.`
      );
    }

    return null;
  }

  const raw = maybeValue.unwrap();
  const output = registry.createType(type, raw);

  return output;
}
