import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { signMessage } from 'viem/accounts';
import { JsonRpcClient } from './json-rpc-client';

// Import types and utilities from existing codebase
interface Web3SignInMessage {
  message_code: string;
  client_id: string;
  omni_account: string;
}

interface UserLoginResponse {
  access_token: string;
  id_token: string;
  backend_response: {
    code: number;
    message: string;
    data: any;
  };
}

interface LoginResult {
  success: boolean;
  responseTime: number;
  requestSize: number;
  responseSize: number;
  error?: string;
  data?: UserLoginResponse;
}

const ClientId = {
  Wildmeta: 'wildmeta' as const,
  Console: 'console' as const
};

export class Web3LoginClient {
  private rpcClient: JsonRpcClient;
  private walletPool: Array<{ privateKey: string; address: string }> = [];
  private currentWalletIndex = 0;

  constructor(rpcUrl = process.env.OMNI_RPC_URL || 'http://localhost:2100/') {
    this.rpcClient = new JsonRpcClient(rpcUrl);
    this.initializeWalletPool();
  }

  private initializeWalletPool(size = 100): void {
    for (let i = 0; i < size; i++) {
      const privateKey = generatePrivateKey();
      const account = privateKeyToAccount(privateKey);
      this.walletPool.push({
        privateKey,
        address: account.address
      });
    }
  }

  private getNextWallet() {
    const wallet = this.walletPool[this.currentWalletIndex];
    this.currentWalletIndex = (this.currentWalletIndex + 1) % this.walletPool.length;
    return wallet;
  }

  private calculateOmniAccount(evmAddress: string): string {
    // Simplified omni account calculation
    // This should match the logic from the existing test
    return evmAddress.toLowerCase();
  }

  async performLogin(): Promise<LoginResult> {
    const startTime = performance.now();
    let wallet: { privateKey: string; address: string } | null = null;
    let requestSize = 0;
    let responseSize = 0;
    let stepError = '';
    
    try {
      // Always get a wallet, even if subsequent steps fail
      wallet = this.getNextWallet();
      const omniAccount = this.calculateOmniAccount(wallet.address);
      
      // Step 1: Get Web3 sign-in message (non-breaking)
      let messageResponse: Web3SignInMessage;
      try {
        messageResponse = await this.rpcClient.call(
          'omni_getWeb3SignInMessage',
          [{
            client_id: ClientId.Wildmeta,
            omni_account: omniAccount
          }]
        );
        requestSize += JSON.stringify([ClientId.Wildmeta, omniAccount]).length;
        responseSize += JSON.stringify(messageResponse).length;
      } catch (error) {
        stepError = `Step 1 - Get message failed: ${error instanceof Error ? error.message : String(error)}`;
        throw new Error(stepError);
      }
      
      // Step 2: Sign the message (non-breaking)
      let signature: string;
      try {
        const messageString = JSON.stringify(messageResponse);
        signature = await signMessage({ 
          message: messageString, 
          privateKey: wallet.privateKey as `0x${string}`
        });
        requestSize += messageString.length;
      } catch (error) {
        stepError = `Step 2 - Sign message failed: ${error instanceof Error ? error.message : String(error)}`;
        throw new Error(stepError);
      }
      
      // Step 3: Perform login (non-breaking)
      let loginResponse: UserLoginResponse;
      try {
        const loginRequest = {
          user_id: {
            type: 'evm' as const,
            value: wallet.address,
          },
          user_auth: {
            type: 'evm' as const,
            value: signature,
          },
          client_id: ClientId.Wildmeta,
          client_auth: {
            type: 'wildmeta',
            value: {
              google_code: '',
              invite_code: '',
            },
          },
        };
        
        loginResponse = await this.rpcClient.call('omni_userLogin', [loginRequest]);
        requestSize += JSON.stringify(loginRequest).length;
        responseSize += JSON.stringify(loginResponse).length;
      } catch (error) {
        stepError = `Step 3 - Login failed: ${error instanceof Error ? error.message : String(error)}`;
        throw new Error(stepError);
      }
      
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      // Validate login response
      const success = loginResponse?.backend_response?.code === 10000;
      const errorMsg = success ? undefined : 
        `Login validation failed: ${loginResponse?.backend_response?.message || 'Invalid response structure'}`;
      
      return {
        success,
        responseTime,
        requestSize,
        responseSize,
        data: loginResponse,
        error: errorMsg
      };
      
    } catch (error) {
      // Always return a result, never throw
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      return {
        success: false,
        responseTime,
        requestSize,
        responseSize,
        error: stepError || (error instanceof Error ? error.message : String(error))
      };
    }
  }

  // Alternative request types for variety in stress testing
  async performShieldingKeyRequest(): Promise<LoginResult> {
    const startTime = performance.now();
    let requestSize = 0;
    let responseSize = 0;
    
    try {
      const response = await this.rpcClient.call('omni_getShieldingKey', []);
      
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      requestSize = JSON.stringify({ method: 'omni_getShieldingKey', params: [] }).length;
      responseSize = JSON.stringify(response).length;
      
      // Validate shielding key response
      const isValid = response && 
                     typeof response.n === 'string' && 
                     typeof response.e === 'string' && 
                     response.n.length > 0 && 
                     response.e.length > 0;
      
      return {
        success: isValid,
        responseTime,
        requestSize,
        responseSize,
        data: response,
        error: isValid ? undefined : 'Invalid shielding key response format'
      };
      
    } catch (error) {
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      return {
        success: false,
        responseTime,
        requestSize,
        responseSize: 0,
        error: `Shielding key request failed: ${error instanceof Error ? error.message : String(error)}`
      };
    }
  }

  async performAddWalletRequest(idToken?: string): Promise<LoginResult> {
    const startTime = performance.now();
    let requestSize = 0;
    let responseSize = 0;
    
    try {
      // Use provided token or perform login first to get token
      let authToken = idToken;
      if (!authToken) {
        const loginResult = await this.performLogin();
        if (!loginResult.success || !loginResult.data?.id_token) {
          return {
            success: false,
            responseTime: performance.now() - startTime,
            requestSize: 0,
            responseSize: 0,
            error: 'Failed to get authentication token for addWallet'
          };
        }
        authToken = loginResult.data.id_token;
      }
      
      const response = await this.rpcClient.call('omni_addWallet', [], authToken);
      
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      requestSize = JSON.stringify({ method: 'omni_addWallet', params: [] }).length;
      responseSize = JSON.stringify(response).length;
      
      // Validate addWallet response
      const isValid = response?.backend_response?.code === 10000;
      
      return {
        success: isValid,
        responseTime,
        requestSize,
        responseSize,
        data: response,
        error: isValid ? undefined : `AddWallet failed: ${response?.backend_response?.message || 'Invalid response'}`
      };
      
    } catch (error) {
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      return {
        success: false,
        responseTime,
        requestSize,
        responseSize: 0,
        error: `AddWallet request failed: ${error instanceof Error ? error.message : String(error)}`
      };
    }
  }

  async performGetNextIntentId(): Promise<LoginResult> {
    const startTime = performance.now();
    
    try {
      const wallet = this.getNextWallet();
      const omniAccount = this.calculateOmniAccount(wallet.address);
      
      const response = await this.rpcClient.call('omni_getNextIntentId', {
        omni_account: omniAccount
      });
      
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      const requestSize = JSON.stringify({ 
        method: 'omni_getNextIntentId', 
        params: { omni_account: omniAccount } 
      }).length;
      const responseSize = JSON.stringify(response).length;
      
      return {
        success: typeof response === 'number' && response > 0,
        responseTime,
        requestSize,
        responseSize,
        data: response
      };
      
    } catch (error) {
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      
      return {
        success: false,
        responseTime,
        requestSize: 0,
        responseSize: 0,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }

  // Mixed request pattern for realistic load testing
  async performRandomRequest(authenticatedToken?: string): Promise<LoginResult> {
    const requestTypes = [
      () => this.performLogin(),
      () => this.performShieldingKeyRequest(),
      () => this.performGetNextIntentId(),
      () => this.performAddWalletRequest(authenticatedToken)
    ];
    
    const randomIndex = Math.floor(Math.random() * requestTypes.length);
    return await requestTypes[randomIndex]();
  }

  // Weighted mixed requests (more realistic distribution)
  async performWeightedRandomRequest(authenticatedToken?: string): Promise<LoginResult> {
    const random = Math.random();
    
    // Weight distribution based on typical usage patterns
    if (random < 0.4) {
      // 40% - User login (most common)
      return this.performLogin();
    } else if (random < 0.7) {
      // 30% - Get shielding key (common for security operations)
      return this.performShieldingKeyRequest();
    } else if (random < 0.9) {
      // 20% - Get next intent ID (transaction preparation)
      return this.performGetNextIntentId();
    } else {
      // 10% - Add wallet (less frequent operation)
      return this.performAddWalletRequest(authenticatedToken);
    }
  }
}