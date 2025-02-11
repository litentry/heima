import { hexToU8a, compactStripLength, u8aToString } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';

export function decodeRpcBytesAsString(value: HexString): string {
    return u8aToString(compactStripLength(hexToU8a(value))[1]);
}
