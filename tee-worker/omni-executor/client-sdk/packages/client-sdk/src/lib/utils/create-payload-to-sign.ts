import { stringToHex, u8aConcat } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import type { Index } from '@polkadot/types/interfaces';

import type { Identity, NativeTask } from '@heima-network/parachain-api';

/**
 * Constructs a message that users need to sign to authorize Enclave's requests.
 * The message is created by concatenating the operation, nonce and mrEnclave,
 * then hashing it with blake2 and adding a prefix.
 *
 * @param args.who - The identity of the signer
 * @param args.operation - The operation to be authorized
 * @param args.nonce - Transaction nonce to prevent replay attacks
 * @param args.mrEnclave - The mrEnclave value
 * @returns A formatted message string ready for signing
 */
export function createPayloadToSign(args: {
  who: Identity;
  task: NativeTask;
  nonce: Index;
  mrEnclave: Uint8Array;
}): string {
  const { who, task, nonce, mrEnclave } = args;
  const payload = u8aConcat(task.toU8a(), nonce.toU8a(), mrEnclave);
  const message = blake2AsHex(payload, 256);

  const prefix = getSignatureMessagePrefix(task);
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

function getSignatureMessagePrefix(_: NativeTask): string {
  // TODO: update this when adding request_batch_vc variant
  return 'Token: ';
}
