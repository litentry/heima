import { hexToU8a } from '@polkadot/util';
import { randomAsHex } from '@polkadot/util-crypto';

import { createPayloadToSign } from '../util/create-payload-to-sign';
import {
  enclave,
  parseRequestVcResultOrError,
  throwIfEmptyOrError,
} from '../enclave';
import * as shieldingKeyUtils from '../util/shielding-key';
import { createTrustedCallType } from '../type-creators/trusted-call';
import { createRequestType } from '../type-creators/request';

import type { JsonRpcRequest } from '../util/types';
import type { ApiPromise } from '@polkadot/api';
import type { Registry } from '@polkadot/types-codec/types';
import type {
  Assertion,
  LitentryIdentity,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';

/**
 * Request a Batch of Verifiable Credential (VC) from the Litentry Protocol.
 *
 * The send's subscribeFn is optional and is used to process the VC payload as it comes in.
 *
 * The final response array, contains WorkerRpcReturnValue as they come in from the Enclave.
 * Notice that the response array is not ordered. Decoding the `WorkerRpcReturnValue.value`
 * into `RequestVcResultOrError` will contain the index of the request and the payload or error.
 *
 * The information about available assertions and their payload can be found in the
 * `Assertion` (`Assertion`) type.
 */
export async function requestBatchVC(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The signer's account.  Use `createLitentryIdentityType` helper to create this struct */
    signer?: LitentryIdentity;
    /** The user's account.  Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** the assertions to be claimed. See `Assertion` type */
    assertions: Array<Assertion>;
  }
): Promise<{
  payloadToSign: string;
  txHash: string;
  send: (
    args: { signedPayload: string },
    subscribeFn?: (
      error: Error | null,
      data: {
        vcPayload: Uint8Array;
        index: number;
        partialResult: Array<WorkerRpcReturnValue>;
      }
    ) => void
  ) => Promise<{
    response: Array<WorkerRpcReturnValue>;
    vcPayloads: Array<Uint8Array | Error>;
    txHash: string;
  }>;
}> {
  const { who, assertions } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);
  const txHash = randomAsHex();

  // identity to sign the requests
  const signer = data.signer || who;

  // Fake the nonce. the batch-vc system has no nonce system yet.
  // it is required because of the TrustedOperation struct. So, it could
  // be anything
  const nonce = api.createType('Index', 0);

  const { call, key } = await createTrustedCallType(api.registry, {
    method: 'request_batch_vc',
    params: {
      signer,
      who,
      assertions,
      hash: txHash,
    },
  });

  const payloadToSign = createPayloadToSign({
    who,
    call,
    nonce,
    shard: shardU8,
  });

  const send = async (
    args: {
      signedPayload: string;
    },
    subscribeFn?: (
      error: Error | null,
      data: {
        vcPayload: Uint8Array;
        index: number;
        partialResult: Array<WorkerRpcReturnValue>;
      }
    ) => void
  ): Promise<{
    response: Array<WorkerRpcReturnValue>;
    vcPayloads: Array<Uint8Array | Error>;
    txHash: string;
  }> => {
    // prepare and encrypt request
    const requestPayload = await createRequestType(api, {
      authentication: {
        type: 'Web3',
        signer,
        signature: args.signedPayload,
      },
      call,
      nonce,
      shard: shardU8,
    });

    // send the request to the Enclave
    const request: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'author_requestVc',
      params: [requestPayload.toHex()],
    };

    // process chunks as VC or error and call the root callback
    const processor = async (
      workerResponse: WorkerRpcReturnValue,
      partialResult: Array<WorkerRpcReturnValue>
    ): Promise<void> => {
      if (typeof subscribeFn !== 'function') {
        return;
      }

      const { error, vcPayload, index } = await processRequestVcResultOrError(
        api.registry,
        workerResponse,
        key
      );

      subscribeFn(error, { vcPayload, index, partialResult });
    };

    const enclaveResult = await enclave.send(api, request, processor);

    // Order the enclave result by index so it matches the order of the request.
    const vcPayloads: Array<Uint8Array | Error> = Array.from({
      length: enclaveResult.length,
    });
    await Promise.all(
      enclaveResult.map(async (workerResponse, idx) => {
        try {
          throwIfEmptyOrError(workerResponse);
        } catch (err: unknown) {
          vcPayloads[idx] = err as Error;
        }

        const { error, vcPayload, index } = await processRequestVcResultOrError(
          api.registry,
          workerResponse,
          key
        );

        vcPayloads[index] = error ?? vcPayload;
      })
    );

    return {
      txHash,
      response: enclaveResult,
      vcPayloads,
    };
  };

  return {
    txHash,
    payloadToSign,
    send,
  };
}

/**
 * Process the `RequestVcResultOrError` type and returns its decrypted content with the given key,
 * or an error if the result contains errors.
 *
 * Unlike other methods, the errors need to be handled as `RequestVcResultOrError`
 * and `WorkerRpcReturnValue.status` isn't checked.
 */
async function processRequestVcResultOrError(
  registry: Registry,
  workerResponse: WorkerRpcReturnValue,
  key: CryptoKey
): Promise<{ error: Error | null; vcPayload: Uint8Array; index: number }> {
  const { errorMessage, hasErrors, index, resultOrError } =
    parseRequestVcResultOrError(registry, workerResponse);

  if (hasErrors) {
    return {
      error: new Error(errorMessage),
      vcPayload: new Uint8Array(),
      index,
    };
  }

  const vcResult = registry.createType(
    'RequestVCResult',
    resultOrError.result.asOk
  );

  const { cleartext: decryptedVcPayload } = await shieldingKeyUtils.decrypt(
    {
      ciphertext: vcResult.vc_payload.ciphertext,
      nonce: vcResult.vc_payload.nonce,
    },
    key
  );

  return {
    error: null,
    vcPayload: decryptedVcPayload,
    index: resultOrError.idx.toNumber(),
  };
}
