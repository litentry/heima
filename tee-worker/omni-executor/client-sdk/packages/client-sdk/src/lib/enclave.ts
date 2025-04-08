import WebSocket from 'isomorphic-ws';
import { ApiPromise } from '@polkadot/api';
import { Codec } from '@polkadot/types-codec/types';
import { compactStripLength, hexToU8a, u8aToString } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';

import { JsonRpcRequest } from '@utils/types';
import { u8aToBase64Url } from '@utils/u8aToBase64Url';

import { ENCLAVE_ENDPOINT } from './config';

export interface EnclaveConfig {
  debug: boolean;
  requestTimeout: number;
}

export enum ConnectionState {
  Connected = 'connected',
  Connecting = 'connecting',
  Disconnected = 'disconnected',
  Disconnecting = 'disconnecting',
}

/**
 * This is a singleton class to mainly hold the Enclave's Shielding Key and MrEnclave.
 *
 * With this class you can:
 * - Retrieve the Enclave's Shielding Key. (1)
 * - Retrieve the Enclave's MrEnclave value which is used as the mrEnclave value. (1)
 * - Encrypt data using the Enclave's Shielding Key.
 * - Send request to the Enclave.
 *
 * (1) Querying from the Parachain, instead of directly from the Enclave Worker itself helps
 * ensuring clients are connected to a trusted worker.
 *
 * 1. Using the global enclave instance:
 * @example
 * ```ts
 * import { enclave } from '@heima-network/client-sdk';
 *
 * const mrEnclave = await enclave.getMrEnclave(api);
 * const key = await enclave.getShieldingKey();
 *
 * console.log({ mrEnclave, key });
 *
 * // Encrypt data using the Enclave's Shielding Key
 * const encrypted = await enclave.encrypt({ cleartext: new Uint8Array([1, 2, 3]) });
 *
 * // Send request to the Enclave.
 * const response = await enclave.send({
 *  jsonrpc: '2.0',
 *  method: 'native_submitAesRequest',
 *  params: ['0x123']
 * });
 * ```
 * 
 * 2. Create your own enclave instance:
 * @example
 * ```ts
 * import { Enclave } from '@heima-network/client-sdk';
 *
 * const enclave = new Enclave('ws://tee-dev.litentry.io');
 * 
 * // you can also set debug mode for logging debug logs
 * enclave.setEnableDebug(true);
 * ```
 */
export class Enclave {
  readonly #config: EnclaveConfig;
  readonly #MAX_ID = Number.MAX_SAFE_INTEGER;
  readonly #endpoint: string | null = null;
  readonly #pendingRequests: Map<
    number,
    {
      resolve: (value: string) => void;
      reject: (reason?: unknown) => void;
    }
  > = new Map();

  #ws: WebSocket | null = null;
  #connectionPromise: Promise<void> | null = null;
  #currentState: ConnectionState = ConnectionState.Disconnected;
  #messageId = 0;
  #mrEnclave: HexString | null = null;
  #shieldingKey: CryptoKey | null = null;

  static #instance: Enclave | null = null;

  static getInstance() {
    if (!Enclave.#instance) {
      Enclave.#instance = new Enclave(ENCLAVE_ENDPOINT);
    }
    return Enclave.#instance;
  }

  /**
   * Creates a new Omni client instance
   * @param endpoint WebSocket endpoint URL
   * @param config Optional configuration overrides
   */
  constructor(endpoint: string, config: Partial<EnclaveConfig> = {}) {
    this.#endpoint = endpoint;
    this.#config = {
      // enable debug logs
      debug: false,
      // default request timeout 30 seconds
      requestTimeout: 60000,
      ...config,
    };
  }

  /**
   * Set debug mode.
   * @param enable Enable debug logs
   */
  setEnableDebug(enable: boolean) {
    this.#config.debug = enable;
  }

  /**
   * Returns the current connection state
   */
  getConnectionState(): ConnectionState {
    return this.#currentState;
  }

  /**
   * Retrieve the Enclave's mrEnclave from the Parachain.
   *
   * The Enclave registry contains the information of the registered TEE workers. These TEE Workers share the
   * same Enclave's mrEnclave value.
   *
   * The value will be held in memory for the duration of the session.
   *
   * @see Test it by yourself https://polkadot.js.org/apps/?rpc=wss://tee-dev.litentry.io#/chainstate
   */
  async getMrEnclave(api: ApiPromise): Promise<`0x${string}`> {
    if (this.#mrEnclave) {
      return this.#mrEnclave;
    }

    const entries = (await api.query.teebag.enclaveRegistry.entries()) as unknown as [HexString, Codec][];

    if (entries.length === 0) {
      throw new Error(`[omni-sdk] No Enclave registry found`);
    }

    const workerType = 'OmniExecutor';
    const omniExecutorEnclaves = entries
      .map((entry) => entry[1].toJSON() as { lastSeenTimestamp: number; workerType: string; mrenclave: HexString })
      .filter((entry) => entry.workerType === workerType);

    if (omniExecutorEnclaves.length === 0) {
      throw new Error(`[omni-sdk] No Enclave registry with type [${workerType}] found`);
    }

    // Find the most recent enclave by lastSeenTimestamp
    const mostRecentEnclave = omniExecutorEnclaves.reduce((prev, current) =>
      current.lastSeenTimestamp > prev.lastSeenTimestamp ? current : prev,
    );

    this.#mrEnclave = mostRecentEnclave.mrenclave;

    return this.#mrEnclave;
  }

  /**
   * Get the Enclave's Shielding Key.
   *
   * @returns Promise that resolves with the crypto key, the value will be held in memory for the duration of the session.
   */
  async getShieldingKey(): Promise<CryptoKey> {
    if (this.#shieldingKey) {
      return this.#shieldingKey;
    }

    const hexString = await this.send({
      jsonrpc: '2.0',
      method: 'omni_getShieldingKey',
      params: [],
    });

    // Remove the hex prefix and SCALE prefix
    const [, data] = compactStripLength(hexToU8a(hexString));
    const pubKey = u8aToString(data);
    const pubKeyJSON = JSON.parse(pubKey);

    const jwkData = {
      alg: 'RSA-OAEP-256',
      kty: 'RSA',
      use: 'enc',
      n: u8aToBase64Url(new Uint8Array([...pubKeyJSON.n].reverse())),
      e: u8aToBase64Url(new Uint8Array([...pubKeyJSON.e].reverse())),
    };

    this.#shieldingKey = await globalThis.crypto.subtle.importKey(
      'jwk',
      jwkData,
      {
        name: 'RSA-OAEP',
        hash: 'SHA-256',
      },
      false,
      ['encrypt'],
    );

    return this.#shieldingKey;
  }

  async encrypt({ cleartext }: { cleartext: Uint8Array }): Promise<{ ciphertext: Uint8Array }> {
    const key = await this.getShieldingKey();

    const encrypted = await globalThis.crypto.subtle.encrypt(
      {
        name: 'RSA-OAEP',
      },
      key,
      cleartext,
    );

    return { ciphertext: new Uint8Array(encrypted) };
  }

  /**
   * Sends a JSON-RPC request over the WebSocket connection
   * @param payload The JSON-RPC request to send
   * @param options Optional settings including custom timeout
   * @returns Promise that resolves with the response
   */
  async send(payload: JsonRpcRequest, options?: { timeout?: number }): Promise<string> {
    await this.#ensureConnection();

    const id = this.#getNextId();
    const request = { ...payload, id };

    return new Promise<string>((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.#pendingRequests.delete(id);
        reject(new Error(`[error:omni-sdk] Request timeout: ${id}`));
      }, options?.timeout ?? this.#config.requestTimeout);

      this.#pendingRequests.set(id, {
        resolve: (value) => {
          clearTimeout(timeoutId);
          resolve(value);
        },
        reject: (reason) => {
          clearTimeout(timeoutId);
          reject(reason);
        },
      });

      if (!this.#ws) {
        reject(new Error('[error:omni-sdk] WebSocket connection failed'));
        return;
      }

      try {
        this.#log('[debug:omni-sdk] sending request', request);
        this.#ws.send(JSON.stringify(request));
      } catch (err) {
        clearTimeout(timeoutId);
        this.#pendingRequests.delete(id);
        reject(err);
      }
    });
  }

  /**
   * Closes the WebSocket connection and performs cleanup
   * Use this method to properly terminate the client connection
   */
  disconnect() {
    if (
      this.#ws &&
      (this.#currentState === ConnectionState.Connected || this.#currentState === ConnectionState.Connecting)
    ) {
      this.#setState(ConnectionState.Disconnecting);
      this.#ws.close();
    }
  }

  /**
   * Updates the connection state
   * @param state New connection state to set
   */
  #setState(state: ConnectionState) {
    this.#currentState = state;
  }

  /**
   * Ensures a valid WebSocket connection exists
   * Creates a new connection if necessary and handles connection events
   * @returns Promise that resolves when connection is established
   */
  async #ensureConnection(): Promise<void> {
    if (this.#connectionPromise) {
      return this.#connectionPromise;
    }

    if (this.#ws?.readyState === WebSocket.OPEN) {
      return Promise.resolve();
    }

    this.#setState(ConnectionState.Connecting);

    this.#connectionPromise = new Promise<void>((resolve, reject) => {
      try {
        if (!this.#endpoint) {
          reject(new Error('[error:omni-sdk] WebSocket endpoint is not set'));
          return;
        }

        this.#ws = new WebSocket(this.#endpoint);

        this.#ws.addEventListener('open', () => {
          this.#setState(ConnectionState.Connected);
          resolve();
        });

        this.#ws.addEventListener('error', (err) => {
          this.#ws = null;
          this.#setState(ConnectionState.Disconnected);
          reject(err);
        });

        this.#ws.addEventListener('close', (e) => {
          this.#ws = null;
          this.#connectionPromise = null;
          this.#setState(ConnectionState.Disconnected);
          for (const [id, { reject }] of this.#pendingRequests) {
            reject(
              new Error(`[error:omni-sdk] WebSocket connection closed - please retry the request: ${id}, event: ${e}`),
            );
          }
          this.#pendingRequests.clear();
        });

        this.#ws.addEventListener('message', (event: WebSocket.MessageEvent) => {
          try {
            const response = JSON.parse(event.data as string);
            this.#log('[debug:omni-sdk] received response', response);
            if (typeof response.id !== 'number') {
              this.#log('[error:omni-sdk] Invalid response id:', response);
              return;
            }

            const pendingRequest = this.#pendingRequests.get(response.id);
            if (!pendingRequest) {
              this.#log('[error:omni-sdk] No pending request found for id:', response.id);
              return;
            }

            this.#pendingRequests.delete(response.id);
            if (response.error) {
              pendingRequest.reject(response.error);
            } else {
              pendingRequest.resolve(response.result);
            }
          } catch (err) {
            this.#log('[error:omni-sdk] Failed to process message:', err);
          }
        });
      } catch (err) {
        this.#ws = null;
        this.#setState(ConnectionState.Disconnected);
        reject(err);
      }
    });

    return this.#connectionPromise;
  }

  /**
   * Generates the next message ID for requests
   * @returns A unique message ID within safe integer bounds
   */
  #getNextId(): number {
    const nextId = (this.#messageId % this.#MAX_ID) + 1;
    this.#messageId = nextId;
    return nextId;
  }

  #log(...args: unknown[]) {
    if (this.#config.debug) {
      console.log(...args);
    }
  }
}

export const enclave = Enclave.getInstance();
try {
  enclave.setEnableDebug(process.env.NODE_ENV !== 'production')
} catch (e) {
  console.warn('cannot set enclave debug mode via process.env.NODE_ENV', e);
}