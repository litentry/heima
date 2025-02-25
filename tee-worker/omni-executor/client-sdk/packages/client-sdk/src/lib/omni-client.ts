import { TypeRegistry, U8aFixed } from '@polkadot/types';
import { identity, MrEnclave, omniExecutor } from '@litentry/parachain-api';
import { JsonRpcRequest } from './util/types';

export interface OmniClientConfig {
  requestTimeout: number;
}

export enum ConnectionState {
  Connected = 'connected',
  Connecting = 'connecting',
  Disconnected = 'disconnected',
  Disconnecting = 'disconnecting',
}

export type ConnectionListener = (state: ConnectionState) => void;

const types = {
  ...identity.types,
  ...omniExecutor.types,
};
const registry = new TypeRegistry();
registry.register(types);

export class OmniClient {
  readonly #config: OmniClientConfig;
  readonly #MAX_ID = Number.MAX_SAFE_INTEGER;
  readonly #endpoint: string | null = null;
  readonly #connectionListeners: Set<ConnectionListener> = new Set();
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
  #mrEnclave: MrEnclave | null = null;

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

  /**
   * Registers a connection state change listener
   * @param listener Callback function to be called on state changes
   * @returns Function to unregister the listener
   */
  onConnectionStateChange(listener: ConnectionListener): () => void {
    this.#connectionListeners.add(listener);
    return () => this.#connectionListeners.delete(listener);
  }

  async getMrEnclave(): Promise<MrEnclave> {
    if (this.#mrEnclave) {
      return this.#mrEnclave;
    }

    // TODO: get mrEnclave from the omni worker
    const mrEnclave = new Uint8Array(32).fill(0);
    this.#mrEnclave = registry.createType<U8aFixed>('MrEnclave', mrEnclave) as unknown as MrEnclave;
    return this.#mrEnclave;
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
      return Promise.reject(new Error('WebSocket connection failed'));
    }

    const id = this.#getNextId();
    const request = { ...payload, id };

    return new Promise<T>((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.#pendingRequests.delete(id);
        reject(new Error(`Request timeout: ${id}`));
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
        console.trace('sending request', request);
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
   * Updates the connection state and notifies all listeners
   * @param state New connection state to set
   */
  #setState(state: ConnectionState) {
    this.#currentState = state;
    this.#connectionListeners.forEach((listener) => listener(state));
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
      return Promise.reject(new Error('WebSocket endpoint is not set'));
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

        this.#ws.addEventListener('close', (e: CloseEvent) => {
          this.#ws = null;
          this.#connectionPromise = null;
          this.#setState(ConnectionState.Disconnected);
          for (const [id, { reject }] of this.#pendingRequests) {
            reject(new Error(`WebSocket connection closed - please retry the request: ${id}, event: ${e}`));
          }
          this.#pendingRequests.clear();
        });

        this.#ws.addEventListener('message', (event) => {
          try {
            const response = JSON.parse(event.data);
            if (typeof response.id !== 'number') {
              console.error('Invalid response id:', response);
              return;
            }

            const pendingRequest = this.#pendingRequests.get(response.id);
            if (!pendingRequest) {
              console.error('No pending request found for id:', response.id);
              return;
            }

            this.#pendingRequests.delete(response.id);
            if (response.error) {
              pendingRequest.reject(response.error);
            } else {
              pendingRequest.resolve(response.result);
            }
          } catch (err) {
            console.error('Failed to process message:', err);
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
  // native_getShieldingKey
  // native_submitAesRequest
  // native_submitPlainRequest
}
