import { stringToHex, u8aConcat } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';

import type { LitentryIdentity, TrustedCall } from '@litentry/parachain-api';
import type { U8aLike } from '@polkadot/util/types';
import type { Index } from '@polkadot/types/interfaces';

/**
 * Construct the message users have to sign to authorize Enclave's requests
 */
export function createPayloadToSign(args: {
  who: LitentryIdentity;
  call: TrustedCall;
  nonce: Index;
  shard: U8aLike;
}): string {
  const { who, call, nonce, shard } = args;
  const payload = u8aConcat(call.toU8a(), nonce.toU8a(), shard, shard);
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

// https://github.com/litentry/litentry-parachain/blob/ca83789997b915c5b9dd6117a22a7bec4de0bf1c/tee-worker/app-libs/stf/src/trusted_call.rs#L262
function getSignatureMessagePrefix(call: TrustedCall): string {
  if (call.isLinkIdentity) {
    return "By linking your identity to our platform, you're taking a step towards a more integrated experience. Please be assured, this process is safe and involves no transactions of your assets. Token: ";
  }

  if (call.isRequestBatchVc) {
    const [, , assertions] = call.asRequestBatchVc;
    const length = assertions.length;

    return `We are going to help you generate ${length} secure credential${
      length > 1 ? 's' : ''
    }. Please be assured, this process is safe and involves no transactions of your assets. Token: `;
  }

  return 'Token: ';
}
