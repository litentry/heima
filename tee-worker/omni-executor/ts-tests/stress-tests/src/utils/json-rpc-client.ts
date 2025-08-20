import 'dotenv/config';

export class JsonRpcClient {
  private url: string;
  private requestId = 0;

  constructor(url = process.env.OMNI_RPC_URL || 'http://localhost:2100/') {
    this.url = url;
  }

  async call(method: string, params: any = null, token?: string): Promise<any> {
    const id = ++this.requestId;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    
    const requestBody = {
      jsonrpc: '2.0',
      id: id.toString(),  // ID should be string like in your curl example
      method,
      params: params || {}  // Always include params object, empty if no params
    };
    
    const body = JSON.stringify(requestBody);
    
    // Print request details
    console.log(`🔵 [${new Date().toISOString()}] JSON-RPC Request:`, {
      url: this.url,
      method: method,
      params: params,
      requestBody: requestBody
    });
    
    const response = await fetch(this.url, {
      method: 'POST',
      headers,
      body
    });
    
    if (!response.ok) {
      console.log(`🔴 [${new Date().toISOString()}] HTTP Error:`, {
        status: response.status,
        statusText: response.statusText,
        method: method
      });
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    const jsonResponse = await response.json() as any;
    
    // Print response details
    console.log(`🟢 [${new Date().toISOString()}] JSON-RPC Response:`, {
      method: method,
      success: !jsonResponse.error,
      response: jsonResponse
    });
    
    if (jsonResponse.error) {
      console.log(`🔴 [${new Date().toISOString()}] JSON-RPC Error:`, {
        method: method,
        error: jsonResponse.error
      });
      throw new Error(`JSON-RPC Error: ${jsonResponse.error.message || jsonResponse.error}`);
    }
    
    return jsonResponse.result;
  }

  setUrl(url: string): void {
    this.url = url;
  }

  getUrl(): string {
    return this.url;
  }
}