import type {
  RequestVcResultOrError,
  StfError,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';
import { JsonRpcRequest, JsonRpcResponse } from './util/types';
import { compactStripLength, hexToU8a, u8aToString } from '@polkadot/util';

import { ENCLAVE_ENDPOINT } from './config';
import { getLastRegisteredEnclave } from './requests';
import { u8aToBase64Url } from './util/u8aToBase64Url';

import type { ApiPromise } from '@polkadot/api';
import type { Enum } from '@polkadot/types-codec';
import type { Registry } from '@polkadot/types-codec/types';

// Direct calls are those that return a response immediately after the request is received.
const DIRECT_REQUEST = ['author_requestVc'];
const isDirectRequest = (call: string): boolean =>
  DIRECT_REQUEST.includes(call);

const log =
  process.env.NODE_ENV !== 'production' ? console.log.bind(console) : () => 0;

/**
 * This is a singleton class to mainly hold the Enclave's Shielding Key and Shard.
 *
 * With this class you can:
 * - Retrieve the Enclave's Shielding Key. (1)
 * - Retrieve the Enclave's MREnclave value which is used as the Shard value. (1)
 * - Encrypt data using the Enclave's Shielding Key.
 * - Send request to the Enclave.
 *
 * (1) Querying from the Parachain, instead of directly from the Enclave Worker itself helps
 * ensuring clients are connected to a trusted worker.
 *
 * @example
 * ```ts
 * import { enclave } from '@litentry/client-sdk';
 *
 * const shard = await enclave.getShard(api);
 * const key = await enclave.getKey(api);
 *
 * console.log({ shard, key });
 *
 * // Encrypt data using the Enclave's Shielding Key
 * const encrypted = await enclave.encrypt(api, { cleartext: new Uint8Array([1, 2, 3]) });
 *
 * // Send request to the Enclave.
 * const response = await enclave.send({
 *  jsonrpc: '2.0',
 *  method: 'author_submitAndWatch',
 *  params: ['0x123']
 * });
 * ```
 */
export class Enclave {
  static #instance: Enclave;
  #key: CryptoKey | null = null;
  #shard: `0x${string}` | null = null;

  constructor() {
    if (Enclave.#instance) {
      return Enclave.#instance;
    }

    Enclave.#instance = this;
  }

  private async retrieveKeyAndShard(
    api: ApiPromise
  ): Promise<{ key: CryptoKey; shard: `0x${string}` }> {
    if (this.#key != null && this.#shard != undefined) {
      return { key: this.#key, shard: this.#shard };
    }

    const data = await retrieveKeyAndShardFromChain(api);
    this.#key = data.key;
    this.#shard = data.shard;

    return data;
  }

  async encrypt(
    api: ApiPromise,
    args: {
      cleartext: Uint8Array;
    }
  ): Promise<{ ciphertext: Uint8Array }> {
    const key = await this.getKey(api);

    const encrypted = await globalThis.crypto.subtle.encrypt(
      {
        name: 'RSA-OAEP',
      },
      key,
      args.cleartext
    );

    return { ciphertext: new Uint8Array(encrypted) };
  }

  /**
   * Get the Enclave's Shielding Key.
   *
   * The value will be held in memory for the duration of the session.
   */
  async getKey(api: ApiPromise): Promise<CryptoKey> {
    const { key } = await this.retrieveKeyAndShard(api);

    return key;
  }

  /**
   * Get the Enclave's Shard. Also referred as MREnclave.
   *
   * The value will be held in memory for the duration of the session.
   */
  async getShard(api: ApiPromise): Promise<`0x${string}`> {
    const { shard } = await this.retrieveKeyAndShard(api);

    return shard;
  }

  /**
   * Send requests to the Enclave.
   *
   * The subscribeFn is a callback that will be called for every message received from the Enclave.
   *
   * The Enclave WebSocket will be closed after the response is completed. A long-lived connection
   * is not offered but should be feasible.
   *
   * For single messages, it will throw an error if the response contains an error.
   *
   */
  async send(
    api: ApiPromise,
    _payload: JsonRpcRequest,
    subscribeFn?: (
      message: WorkerRpcReturnValue,
      partialResult: Array<WorkerRpcReturnValue>
    ) => Promise<void>
  ): Promise<Array<WorkerRpcReturnValue>> {
    const response = await new Promise<Array<WorkerRpcReturnValue>>(
      (resolve, reject) => {
        const ws = new WebSocket(ENCLAVE_ENDPOINT);
        const response: Array<WorkerRpcReturnValue> = [];

        // Since we are using a one-shot connection we can use id=1 for all requests.
        const payload = {
          id: 1,
          ..._payload,
        };

        ws.addEventListener('open', function open() {
          ws.send(JSON.stringify(payload));
        });

        ws.addEventListener('error', (err) => {
          console.error(err);
          ws.close();
          reject(err);
        });

        ws.addEventListener('message', function message(event) {
          const rawData = event.data;
          // heads-up: it is important to cast the data to string before parsing it
          // otherwise it will be parsed as a buffer and the JSONRPC response will
          // be invalid
          const receivedData = JSON.parse(rawData.toString());

          if (
            !isJsonRpcResponse(receivedData) ||
            receivedData.id.toString() !== payload?.id?.toString()
          ) {
            // skip
            log('[debug:enclave:ws] Invalid message. skipped', receivedData);
            return;
          }

          const workerRpcResponse = api.createType(
            'WorkerRpcReturnValue',
            receivedData.result
          );

          log(
            '[debug:enclave:ws] received workerRpcResponse',
            workerRpcResponse.toHuman()
          );

          if (
            workerRpcResponse.value.isEmpty &&
            workerRpcResponse.do_watch.isTrue
          ) {
            log('[enclave:ws] Empty value. skipped');
            return;
          }

          // quick clone for partial result
          const partialResult = response.slice(0);

          // For direct responses, we stream as we get messages. Otherwise,
          // We wait until do_watch is false to resolve the promise.
          if (isDirectRequest(payload.method)) {
            response.push(workerRpcResponse);
            subscribeFn?.(workerRpcResponse, partialResult);
          }

          // Wait until the response is completed to close the connection
          if (workerRpcResponse.do_watch.isTrue) {
            return;
          }

          // Now that the response is completed, we stream the responses for non-direct requests
          if (!isDirectRequest(payload.method)) {
            response.push(workerRpcResponse);
            subscribeFn?.(workerRpcResponse, partialResult);
          }

          // close connection
          ws.close();

          resolve(response);
        });
      }
    );

    // Throw errors for single messages
    if (response.length === 1) {
      const workerResponse = response[0];

      throwIfEmptyOrError(workerResponse);

      // for non-stf requests like vc request, we need to look into `RequestVcResultOrError`
      // for stf requests, we to look into `StfError`
      if (_payload.method === 'author_requestVc') {
        const { hasErrors, errorMessage } = parseRequestVcResultOrError(
          api.registry,
          workerResponse
        );

        if (hasErrors) {
          throw new Error(errorMessage);
        }
      } else {
        const { hasErrors, errorMessage } = extractWorkerRpcReturnValueErrors(
          api.registry,
          workerResponse
        );

        if (hasErrors) {
          throw new Error(errorMessage);
        }
      }
    }

    return response;
  }
}

function isJsonRpcResponse(data: unknown): data is JsonRpcResponse {
  return (
    typeof data === 'object' &&
    data !== null &&
    'jsonrpc' in data &&
    'result' in data &&
    'id' in data &&
    typeof data.result === 'string' &&
    typeof data.id === 'number'
  );
}

/**
 * Enclave instance
 *
 * This an instance of the Enclave class.
 *
 * @see Enclave
 *
 * @example
 * ```ts
 * import { enclave } from '@litentry/identity-hub';
 *
 * const shard = await enclave.getShard(api);
 * const key = await enclave.getKey(api);
 * const encrypted = await enclave.encrypt(api, { cleartext: new Uint8Array([1, 2, 3]) });
 * const response = await enclave.send({
 *  jsonrpc: '2.0',
 *  method: 'author_submitAndWatch',
 *  params: ['0x123']
 * });
 * ```
 */
export const enclave = new Enclave();

/**
 * Retrieve the Enclave's Shielding Key and Shard from the Parachain.
 *
 * The Parachain exposes the TEE Shielding Key via the Enclave Registry on its Storage module.
 * The Enclave registry contains the information of the registered TEE workers. These TEE Workers share the
 * same Enclave's TEE Shielding Key and Shard value.
 *
 * @see Test it by yourself https://polkadot.js.org/apps/?rpc=wss://tee-staging.litentry.io:443#/chainstate
 */
async function retrieveKeyAndShardFromChain(
  api: ApiPromise
): Promise<{ key: CryptoKey; shard: `0x${string}` }> {
  const { account, enclave } = await getLastRegisteredEnclave(api);

  if (enclave.shieldingPubkey.isEmpty || enclave.vcPubkey.isEmpty) {
    throw new Error(
      `[Litentry vault] Unknown TEE records for ${account.toHuman()}`
    );
  }

  const firstTEEWorkerJson = {
    pubkey: account.toHuman(), // SS58 formatted (address)
    shieldingKey: u8aToString(enclave.shieldingPubkey.unwrap()), // JSON string
    timestamp: enclave.lastSeenTimestamp.toNumber(), // e.g., 1674819846045
    mrEnclave: enclave.mrenclave.toHex(), // same as shard
    sgxMode: enclave.sgxBuildMode.toHuman(),
    url: enclave.url.toHuman(),
  };

  console.log(
    `[Litentry vault] Reading TEE Shielding Key from TEE Worker ${
      firstTEEWorkerJson.pubkey
    }. Timestamp ${new Date(firstTEEWorkerJson.timestamp)}`
  );

  const pubKeyJSON = JSON.parse(firstTEEWorkerJson.shieldingKey);

  const jwkData = {
    alg: 'RSA-OAEP-256',
    kty: 'RSA',
    use: 'enc',
    n: u8aToBase64Url(new Uint8Array([...pubKeyJSON.n].reverse())),
    e: u8aToBase64Url(new Uint8Array([...pubKeyJSON.e].reverse())),
  };

  const key = await globalThis.crypto.subtle.importKey(
    'jwk',
    jwkData,
    {
      name: 'RSA-OAEP',
      hash: 'SHA-256',
    },
    false,
    ['encrypt']
  );

  return { key, shard: firstTEEWorkerJson.mrEnclave };
}

export function throwIfEmptyOrError(
  workerResponse: WorkerRpcReturnValue
): void {
  // we ignore falsy values like 0x00
  // Notice that value.isEmpty produces true for 0x00, which is not what we want.
  if (workerResponse.value.toString() === '') {
    throw new Error('Empty response');
  }

  if (workerResponse.status.isError) {
    const msg = u8aToString(
      compactStripLength(hexToU8a(workerResponse.value.toHex()))[1]
    );

    throw new Error(msg);
  }
}

/**
 * Look for errors into `WorkerRpcReturnValue` type and decodes its error message.
 */
export function extractWorkerRpcReturnValueErrors(
  registry: Registry,
  workerResponse: WorkerRpcReturnValue
): {
  hasErrors: boolean;
  errorMessage: string;
} {
  if (workerResponse.status.isOk) {
    return { hasErrors: false, errorMessage: '' };
  }

  // look for error in TrustedOperationStatus
  if (workerResponse.status.isTrustedOperationStatus) {
    const [status, _hashIgnored] =
      workerResponse.status.asTrustedOperationStatus;

    // for stf request like link_identity, InSidechainBlock represents OK
    // for non-stf request like request_vc, Submitted plus do_watch=false (checked before calling this function)
    // represents the real response and should be handled by RequestVcResultOrError
    if (status.isSubmitted || status.isInSidechainBlock) {
      return { hasErrors: false, errorMessage: '' };
    }

    try {
      const stfError = registry.createType('StfError', workerResponse.value);
      const msg = stfErrorToString(stfError);

      return { hasErrors: true, errorMessage: msg };
    } catch (_) {
      // if we can't decode we assume it is not an StfError
    }

    if (status.isInvalid) {
      return {
        hasErrors: true,
        errorMessage: 'Invalid TrustedOperation..',
      };
    }

    // all other status are considered OK
    return { hasErrors: false, errorMessage: '' };
  }

  return {
    hasErrors: true,
    errorMessage: `Unhandled WorkerRpcReturnValue status: ${workerResponse.status.type}`,
  };
}

/**
 * Generic function to extract the Codec's type and value.
 */
function codecToString(codec: Enum): string {
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

/**
 * This helper will try to decode the `WorkerRpcReturnValue.value` response as StfError string
 * and resolve the error message for the Enum types with known ErrorDetail values.
 */
function stfErrorToString(stfError: StfError): string {
  if (stfError.isInvalidNonce) {
    const [nonce1, nonce2] = stfError.asInvalidNonce;

    return `${stfError.type}: [${nonce1.toHuman()}, ${nonce2.toHuman()}]`;
  }

  return codecToString(stfError);
}

export function parseRequestVcResultOrError(
  registry: Registry,
  workerResponse: WorkerRpcReturnValue
): {
  hasErrors: boolean;
  errorMessage: string;
  index: number;
  resultOrError: RequestVcResultOrError;
} {
  const resultOrError = registry.createType(
    'RequestVcResultOrError',
    workerResponse.value
  );

  if (resultOrError.result.isOk) {
    return {
      hasErrors: false,
      errorMessage: '',
      index: resultOrError.idx.toNumber(),
      resultOrError,
    };
  }

  const error = resultOrError.result.asErr;
  let errorMessage = '';

  if (error.isAssertionBuildFailed) {
    const assertionBuildFailed = error.asAssertionBuildFailed;

    if (assertionBuildFailed.isRequestVCFailed) {
      const [assertion, errorDetail] = assertionBuildFailed.asRequestVCFailed;

      errorMessage = `${assertionBuildFailed.type}. ${codecToString(
        errorDetail
      )} (${assertion.type}, ${JSON.stringify(assertion.value.toHuman())})`;
    }
  } else {
    errorMessage = codecToString(error);
  }

  return {
    hasErrors: true,
    errorMessage,
    index: resultOrError.idx.toNumber(),
    resultOrError,
  };
}
