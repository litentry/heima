export type Maybe<T> = null | undefined | T;

export type JsonRpcRequest = {
  jsonrpc: string;
  method: string;
  params: Array<string>;
  /**
   * Use sequential numbers starting from 1 for consecutive requests.
   * For one-time request that closes connections right away, using `1` is ok.
   */
  id?: number;
};

export type JsonRpcResponse = {
  jsonrpc: string;
  id: number | string;
  result: string;
};

/**
 * Substrate SS58 address, Substrate public key or EVM address
 */
export type SubstrateOrEvmOrBtcAddress = string | `0x${string}`;
