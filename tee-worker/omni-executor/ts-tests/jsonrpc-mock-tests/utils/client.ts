export interface JsonRpcRequest {
  jsonrpc: '2.0';
  method: string;
  params?: any[];
  id: string | number;
}

export interface JsonRpcResponse<T = any> {
  jsonrpc: '2.0';
  result?: T;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
  id: string | number;
}

export class JsonRpcError extends Error {
  public readonly code: number;
  public readonly data?: any;

  constructor(error: { code: number; message: string; data?: any }) {
    super(error.message);
    this.name = 'JsonRpcError';
    this.code = error.code;
    this.data = error.data;
  }
}

export class JsonRpcClient {
  private static instance: JsonRpcClient;
  private readonly endpoint = 'http://localhost:2100/';
  private requestId = 1;

  private constructor() {}

  static getInstance(): JsonRpcClient {
    if (!JsonRpcClient.instance) {
      JsonRpcClient.instance = new JsonRpcClient();
    }
    return JsonRpcClient.instance;
  }

  async call<T = any>(method: string, params?: any, token?: string): Promise<T> {
    const request: JsonRpcRequest = {
      jsonrpc: '2.0',
      method,
      params: params || [],
      id: this.requestId++
    };

    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(this.endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify(request)
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const jsonResponse: JsonRpcResponse<T> = await response.json();
    console.log('jsonrpc method', method);
    console.log('jsonrpc params', params);
    console.log('jsonrpc response', jsonResponse);
    if (jsonResponse.error) {
      throw new JsonRpcError(jsonResponse.error);
    }

    return jsonResponse.result!;
  }
}
