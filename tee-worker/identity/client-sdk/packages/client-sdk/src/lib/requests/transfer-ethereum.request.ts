import { assert, hexToU8a } from '@polkadot/util';
import { randomAsHex } from '@polkadot/util-crypto';

import type { ApiPromise } from '@polkadot/api';
import type {
  LitentryIdentity,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';

import { enclave } from '../enclave';
import { createPayloadToSign } from '../util/create-payload-to-sign';
import { createTrustedCallType } from '../type-creators/trusted-call';
import { createRequestType } from '../type-creators/request';

import type { JsonRpcRequest } from '../util/types';

/**
 * OmniAccount: Transfer funds to Ethereum Network.
 */
export async function transferEthereum(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The user's omniAccount.  Use `createLitentryIdentityType` helper to create this struct */
    omniAccount: LitentryIdentity;
    /** The user's account.  Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** Ethereum address destination */
    to: string;
    /** Amount to send */
    amount: bigint;
  }
): Promise<{
  payloadToSign: string;
  txHash: string;
  send: (args: { signedPayload: string }) => Promise<{
    response: Array<WorkerRpcReturnValue>;
    txHash: string;
  }>;
}> {
  const { who, omniAccount } = data;

  assert(omniAccount.isSubstrate, 'OmniAccount must be a Substrate identity');

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);
  const txHash = randomAsHex();

  const { call } = await createTrustedCallType(api.registry, {
    method: 'request_intent',
    params: {
      who,
      intent: api.createType('Intent', {
        TransferEthereum: api.createType('IntentTransferEthereum', {
          to: data.to,
          value: data.amount,
        }),
      }),
    },
  });

  const nonce = await api.rpc.system.accountNextIndex(omniAccount.asSubstrate);

  const payloadToSign = createPayloadToSign({
    who,
    call,
    nonce,
    shard: shardU8,
  });

  const send = async (args: {
    signedPayload: string;
  }): Promise<{
    response: Array<WorkerRpcReturnValue>;
    txHash: string;
  }> => {
    // prepare and encrypt request

    const request = await createRequestType(api, {
      signer: who,
      signature: args.signedPayload,
      call,
      nonce,
      shard: shardU8,
    });

    // send the request to the Enclave
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'author_submitNativeRequest',
      params: [request.toHex()],
    };

    const enclaveResult = await enclave.send(api, rpcRequest);

    return {
      txHash,
      response: enclaveResult,
    };
  };

  return {
    txHash,
    payloadToSign,
    send,
  };
}
