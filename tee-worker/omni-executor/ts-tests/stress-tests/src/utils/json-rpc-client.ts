export class JsonRpcClient {
  private url: string;
  private requestId = 0;

  constructor(url = process.env.OMNI_RPC_URL || 'http://localhost:2100/') {
    this.url = url;
  }

  async call(method: string, params: any = {}, token?: string): Promise<any> {
    const id = ++this.requestId;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    
    const body = JSON.stringify({
      jsonrpc: '2.0',
      id,
      method,
      params
    });
    
    const response = await fetch(this.url, {
      method: 'POST',
      headers,
      body
    });
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    const jsonResponse = await response.json() as any;
    
    if (jsonResponse.error) {
      throw new Error(`JSON-RPC Error: ${jsonResponse.error.message}`);
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