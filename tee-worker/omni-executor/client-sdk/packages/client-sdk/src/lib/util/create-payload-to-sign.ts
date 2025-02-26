import { stringToHex, u8aConcat } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import type { U8aLike } from '@polkadot/util/types';
import type { Index } from '@polkadot/types/interfaces';

import type { LitentryIdentity, NativeCall } from '@litentry/parachain-api';

/**
 * Construct the message users have to sign to authorize Enclave's requests
 */
export function createPayloadToSign(args: {
  who: LitentryIdentity;
  call: NativeCall;
  nonce: Index;
  shard: U8aLike;
}): string {
  const { who, call, nonce, shard } = args;
  const payload = u8aConcat(call.toU8a(), nonce.toU8a(), shard);
  const message = blake2AsHex(payload, 256);

  const prefix = getSignatureMessagePrefix(call);
  const msg = prefix + message;

  // evm needs hex encoding for proper display
  if (who.isEvm) {
    return stringToHex(msg);
  }

  // Bitcoin, If the message is hex encoded, remove the prefix
  if (who.isBitcoin && msg.startsWith('0x')) {
    return msg.slice(2);
  }

  return msg;
}

function getSignatureMessagePrefix(_: NativeCall): string {
  // TODO: update this when adding request_batch_vc variant
  return 'Token: ';
}
