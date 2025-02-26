import WebSocket from 'isomorphic-ws';
import { TypeRegistry } from '@polkadot/types';
import { Index } from '@polkadot/types/interfaces';
import { compactStripLength, hexToU8a, u8aToString } from '@polkadot/util';
import { ApiPromise, identity, LitentryIdentity, omniExecutor } from '@litentry/parachain-api';
import { JsonRpcRequest } from './util/types';
import { u8aToBase64Url } from './util/u8aToBase64Url';
import { getLastRegisteredEnclave } from './request/get-last-registered-enclave';
import { OMNI_ENDPOINT } from './config';

export interface OmniClientConfig {
  requestTimeout: number;
}

export enum ConnectionState {
  Connected = 'connected',
  Connecting = 'connecting',
  Disconnected = 'disconnected',
  Disconnecting = 'disconnecting',
}

const types = {
  ...identity.types,
  ...omniExecutor.types,
};
const registry = new TypeRegistry();
registry.register(types);

export class Enclave {
  readonly #config: OmniClientConfig;
  readonly #MAX_ID = Number.MAX_SAFE_INTEGER;
  readonly #endpoint: string | null = null;
  readonly #pendingRequests: Map<
    number,
    {
      resolve: (value: any) => void;
      reject: (reason?: any) => void;
    }
  > = new Map();

  #ws: WebSocket | null = null;
  #connectionPromise: Promise<void> | null = null;
  #currentState: ConnectionState = ConnectionState.Disconnected;
  #messageId: number = 1;
  #shard: `0x${string}` | null = null;
  #shieldingKey: CryptoKey | null = null;

  static #instance: Enclave | null = null;

  static getInstance() {
    if (!Enclave.#instance) {
      Enclave.#instance = new Enclave(OMNI_ENDPOINT);
    }
    return Enclave.#instance;
  }

  /**
   * Creates a new Omni client instance
   * @param endpoint WebSocket endpoint URL
   * @param config Optional configuration overrides
   */
  constructor(endpoint: string, config: Partial<OmniClientConfig> = {}) {
    this.#endpoint = endpoint;
    this.#config = {
      // default request timeout 30 seconds
      requestTimeout: 30000,
      ...config,
    };
  }

  /**
   * Returns the current connection state
   */
  getConnectionState(): ConnectionState {
    return this.#currentState;
  }

  async getNonce(
    /** Litentry Parachain API instance from Polkadot.js */
    api: ApiPromise,
    /** The user's omniAccount.  Use `createLitentryIdentityType` helper to create this struct */
    omniAccount: LitentryIdentity,
  ): Promise<Index> {
    return await api.rpc.system.accountNextIndex(omniAccount.asSubstrate.toHex());
  }

  /**
   * Retrieve the Enclave's Shard from the Parachain.
   *
   * The Enclave registry contains the information of the registered TEE workers. These TEE Workers share the
   * same Enclave's Shard value.
   *
   * @see Test it by yourself https://polkadot.js.org/apps/?rpc=wss://tee-dev.litentry.io#/chainstate
   */
  async getShard(api: ApiPromise): Promise<`0x${string}`> {
    if (this.#shard) {
      return this.#shard;
    }

    const { account, enclave } = await getLastRegisteredEnclave(api);

    const firstTEEWorkerJson = {
      pubkey: account.toHuman(), // SS58 formatted (address)
      timestamp: enclave.lastSeenTimestamp.toNumber(), // e.g., 1674819846045
      mrEnclave: enclave.mrenclave.toHex(), // same as shard
      sgxMode: enclave.sgxBuildMode.toHuman(),
    };

    console.trace(
      `[omni-sdk] Reading TEE Shielding Key from TEE Worker ${
        firstTEEWorkerJson.pubkey
      }. Timestamp ${new Date(firstTEEWorkerJson.timestamp)}`,
    );

    this.#shard = firstTEEWorkerJson.mrEnclave;

    return this.#shard;
  }

  async getShieldingKey(): Promise<CryptoKey> {
    if (this.#shieldingKey) {
      return this.#shieldingKey;
    }

    const hexString = await this.send<string>({
      jsonrpc: '2.0',
      method: 'native_getShieldingKey',
      params: [],
    });

    // Remove the hex prefix and SCALE prefix
    const [, data] = compactStripLength(hexToU8a(hexString))
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

  async encrypt(cleartext: Uint8Array): Promise<{ ciphertext: Uint8Array }> {
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
  async send<T = any>(payload: JsonRpcRequest, options?: { timeout?: number }): Promise<T> {
    await this.#ensureConnection();

    if (!this.#ws) {
      return Promise.reject(new Error('[omni-sdk] WebSocket connection failed'));
    }

    const id = this.#getNextId();
    const request = { ...payload, id };

    return new Promise<T>((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.#pendingRequests.delete(id);
        reject(new Error(`[omni-sdk] Request timeout: ${id}`));
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

      try {
        console.trace('[omni-sdk] sending request', request);
        this.#ws!.send(JSON.stringify(request));
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

    if (!this.#endpoint) {
      return Promise.reject(new Error('[omni-sdk] WebSocket endpoint is not set'));
    }

    this.#setState(ConnectionState.Connecting);

    this.#connectionPromise = new Promise<void>((resolve, reject) => {
      try {
        this.#ws = new WebSocket(this.#endpoint!);

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
            reject(new Error(`[omni-sdk] WebSocket connection closed - please retry the request: ${id}, event: ${e}`));
          }
          this.#pendingRequests.clear();
        });

        this.#ws.addEventListener('message', (event: any) => {
          try {
            const response = JSON.parse(event.data);
            console.trace('[omni-sdk] received response', response);
            if (typeof response.id !== 'number') {
              console.error('[omni-sdk] Invalid response id:', response);
              return;
            }

            const pendingRequest = this.#pendingRequests.get(response.id);
            if (!pendingRequest) {
              console.error('[omni-sdk] No pending request found for id:', response.id);
              return;
            }

            this.#pendingRequests.delete(response.id);
            if (response.error) {
              pendingRequest.reject(response.error);
            } else {
              pendingRequest.resolve(response.result);
            }
          } catch (err) {
            console.error('[omni-sdk] Failed to process message:', err);
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

  // omni_getOAuth2GoogleAuthorizationUrl
  // omni_requestEmailVerificationCode
  // native_submitPlainRequest
}

export const enclave = Enclave.getInstance();
