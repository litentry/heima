#!/usr/bin/env node

import 'dotenv/config';
import chalk from 'chalk';
import { performance } from 'perf_hooks';
import { JsonRpcClient } from './utils/json-rpc-client';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { sha256 } from 'js-sha256';
import { DataStorage } from './storage/advanced-data-storage';
import { Logger } from './utils/advanced-logger';
import { PerformanceAnalyzer } from './analysis/advanced-performance-analyzer';
import { Dashboard } from './dashboard/advanced-dashboard';

// Test configuration interface
interface TestConfig {
  targetUrl: string;
  maxQPSPercentage: number; // Percentage of discovered max (e.g., 70 for 70%)
  stepDurationSeconds: number;
  rampUpStrategy: 'linear' | 'exponential' | 'fibonacci';
  testDurationMinutes: number;
  walletPoolSize: number;
  endpoints: TestEndpoint[];
  outputDir: string;
}

// Endpoint configuration
interface TestEndpoint {
  name: string;
  method: string;
  weight: number; // Percentage of total requests (0-100)
  paramGenerator: () => any;
  validator: (response: any) => boolean;
  enabled: boolean;
}

// Request result interface
interface RequestResult {
  endpoint: string;
  success: boolean;
  responseTime: number;
  requestSize: number;
  responseSize: number;
  timestamp: number;
  qps: number;
  error?: string;
  statusCode?: number;
  requestData?: any;
  responseData?: any;
}

// QPS step result
interface QPSStepResult {
  qps: number;
  actualQPS: number;
  duration: number;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  endpointStats: Record<string, {
    total: number;
    successful: number;
    failed: number;
    avgResponseTime: number;
    minResponseTime: number;
    maxResponseTime: number;
    errorRate: number;
    throughput: number;
  }>;
  responseMetrics: {
    avgLatency: number;
    minLatency: number;
    maxLatency: number;
    p50Latency: number;
    p95Latency: number;
    p99Latency: number;
  };
  errorDetails: Record<string, number>;
  shouldStop: boolean;
  stopReason?: string;
}

// Test session result
interface TestSessionResult {
  sessionId: string;
  startTime: number;
  endTime: number;
  config: TestConfig;
  steps: QPSStepResult[];
  maxSustainableQPS: number;
  recommendedMaxQPS: number;
  overallStats: {
    totalRequests: number;
    totalSuccessful: number;
    totalFailed: number;
    overallSuccessRate: number;
    avgResponseTime: number;
    maxResponseTime: number;
    minResponseTime: number;
  };
}

const ClientId = {
  Wildmeta: 'wildmeta',
  Heima: 'heima',
  Console: 'console'
};

class AdvancedStressTest {
  private rpcClient: JsonRpcClient;
  private walletPool: Array<{ privateKey: string; address: string; omniAccount: string }> = [];
  private currentWalletIndex = 0;
  private config: TestConfig;
  private storage: DataStorage;
  private logger: Logger;
  private analyzer: PerformanceAnalyzer;
  private dashboard: Dashboard;
  private sessionId: string;
  private running = false;

  constructor(config: Partial<TestConfig> = {}) {
    this.sessionId = `stress_test_${Date.now()}`;
    
    // Default configuration
    this.config = {
      targetUrl: process.env.OMNI_RPC_URL || 'http://localhost:2100/',
      maxQPSPercentage: 70,
      stepDurationSeconds: 20,
      rampUpStrategy: 'exponential',
      testDurationMinutes: 10,
      walletPoolSize: 500,
      outputDir: './stress-test-results',
      endpoints: [
        {
          name: 'omni_getNextIntentId',
          method: 'omni_getNextIntentId',
          weight: 50,
          paramGenerator: () => ({
            omni_account: this.getRandomWallet().omniAccount
          }),
          validator: (response) => typeof response === 'number' && response > 0,
          enabled: true
        },
        {
          name: 'omni_getWeb3SignInMessage',
          method: 'omni_getWeb3SignInMessage',
          weight: 50,
          paramGenerator: () => {
            const wallet = this.getRandomWallet();
            // Use object parameters (both work, but object is clearer)
            return {
              client_id: ClientId.Wildmeta,
              omni_account: wallet.omniAccount
            };
          },
          validator: (response) => 
            response && 
            typeof response.message_code === 'string' &&
            typeof response.client_id === 'string' &&
            typeof response.omni_account === 'string',
          enabled: true
        }
      ],
      ...config
    };

    this.rpcClient = new JsonRpcClient(this.config.targetUrl);
    this.storage = new DataStorage(this.config.outputDir, this.sessionId);
    this.logger = new Logger(this.config.outputDir, this.sessionId);
    this.analyzer = new PerformanceAnalyzer();
    this.dashboard = new Dashboard(this.config.outputDir);
    
    this.initializeWalletPool();
  }

  private initializeWalletPool(): void {
    this.logger.info(`Initializing wallet pool with ${this.config.walletPoolSize} wallets...`);
    
    for (let i = 0; i < this.config.walletPoolSize; i++) {
      const privateKey = generatePrivateKey();
      const account = privateKeyToAccount(privateKey);
      const omniAccount = this.calculateOmniAccount(account.address, ClientId.Wildmeta);
      
      this.walletPool.push({
        privateKey,
        address: account.address,
        omniAccount
      });
    }
    
    this.logger.info(`Wallet pool initialized with ${this.walletPool.length} wallets`);
  }

  private calculateOmniAccount(evmAddress: string, clientId: string): string {
    const inputs: Uint8Array[] = [];
    
    // Client ID as raw bytes
    const clientIdBytes = new TextEncoder().encode(clientId);
    inputs.push(clientIdBytes);
    
    // Identity type ("evm")
    inputs.push(new TextEncoder().encode('evm'));
    
    // EVM address bytes (remove 0x prefix)
    const addressHex = evmAddress.slice(2).toLowerCase();
    const addressBytes = new Uint8Array(20);
    for (let i = 0; i < addressHex.length; i += 2) {
      addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
    }
    inputs.push(addressBytes);
    
    // Combine all inputs
    const totalLength = inputs.reduce((sum, arr) => sum + arr.length, 0);
    const combined = new Uint8Array(totalLength);
    let offset = 0;
    for (const input of inputs) {
      combined.set(input, offset);
      offset += input.length;
    }
    
    // Calculate SHA256 hash using js-sha256
    const hash = sha256.array(combined);
    return `0x${hash.map((b: number) => b.toString(16).padStart(2, '0')).join('')}`;
  }

  private getRandomWallet() {
    const wallet = this.walletPool[this.currentWalletIndex];
    this.currentWalletIndex = (this.currentWalletIndex + 1) % this.walletPool.length;
    return wallet;
  }

  private selectEndpointByWeight(): TestEndpoint {
    const enabledEndpoints = this.config.endpoints.filter(ep => ep.enabled);
    const totalWeight = enabledEndpoints.reduce((sum, ep) => sum + ep.weight, 0);
    const random = Math.random() * totalWeight;
    
    let currentWeight = 0;
    for (const endpoint of enabledEndpoints) {
      currentWeight += endpoint.weight;
      if (random <= currentWeight) {
        return endpoint;
      }
    }
    
    return enabledEndpoints[0]; // Fallback
  }

  private async executeRequest(endpoint: TestEndpoint, qps: number): Promise<RequestResult> {
    const startTime = performance.now();
    const timestamp = Date.now();
    
    try {
      const params = endpoint.paramGenerator();
      const requestData = { method: endpoint.method, params };
      const requestSize = JSON.stringify(requestData).length;
      
      // Network latency measurement - DNS resolution + connection time
      const networkStartTime = performance.now();
      
      this.logger.debug(`Executing ${endpoint.name}`, { params, qps });
      
      const response = await this.rpcClient.call(endpoint.method, params);
      const endTime = performance.now();
      
      // Total response time including network overhead
      const responseTime = endTime - startTime;
      const networkLatency = endTime - networkStartTime;
      const responseSize = JSON.stringify(response).length;
      
      const isValid = endpoint.validator(response);
      
      const result: RequestResult = {
        endpoint: endpoint.name,
        success: isValid,
        responseTime, // Total latency including processing and network
        requestSize,
        responseSize,
        timestamp,
        qps,
        requestData: params,
        responseData: response,
        statusCode: 200
      };
      
      if (!isValid) {
        result.error = 'Response validation failed';
        this.logger.warn(`Response validation failed for ${endpoint.name}`, { response, responseTime });
      } else {
        this.logger.debug(`${endpoint.name} success`, { 
          responseTime: Math.round(responseTime * 100) / 100, // Round to 2 decimal places
          networkLatency: Math.round(networkLatency * 100) / 100,
          responseSize 
        });
      }
      
      return result;
      
    } catch (error) {
      const endTime = performance.now();
      const responseTime = endTime - startTime;
      const errorMessage = error instanceof Error ? error.message : String(error);
      
      this.logger.error(`${endpoint.name} failed`, { error: errorMessage, qps });
      
      return {
        endpoint: endpoint.name,
        success: false,
        responseTime,
        requestSize: 0,
        responseSize: 0,
        timestamp,
        qps,
        error: errorMessage,
        statusCode: this.extractStatusCode(errorMessage)
      };
    }
  }

  private extractStatusCode(error: string): number {
    const match = error.match(/HTTP (\d+)/);
    return match ? parseInt(match[1]) : 0;
  }

  private categorizeError(error: string): string {
    if (error.includes('timeout')) return 'Timeout';
    if (error.includes('connection')) return 'Connection';
    if (error.includes('HTTP 4')) return 'Client Error (4xx)';
    if (error.includes('HTTP 5')) return 'Server Error (5xx)';
    if (error.includes('validation failed')) return 'Validation Error';
    if (error.includes('JSON-RPC')) return 'RPC Error';
    return 'Other';
  }

  private calculateNextQPS(currentQPS: number, stepResult: QPSStepResult): number {
    const { errorRate } = this.calculateStepErrorRate(stepResult);
    
    switch (this.config.rampUpStrategy) {
      case 'linear':
        if (errorRate < 1) return currentQPS + Math.max(5, Math.floor(currentQPS * 0.3));
        if (errorRate < 5) return currentQPS + 2;
        return currentQPS; // Stop increasing
        
      case 'exponential':
        // More aggressive scaling to reach blockchain-level QPS
        if (currentQPS < 10) {
          return errorRate < 1 ? currentQPS * 3 : currentQPS + 2;
        } else if (currentQPS < 50) {
          return errorRate < 1 ? Math.floor(currentQPS * 2) : currentQPS + Math.floor(currentQPS * 0.2);
        } else if (currentQPS < 200) {
          return errorRate < 1 ? Math.floor(currentQPS * 1.5) : currentQPS + Math.floor(currentQPS * 0.1);
        } else {
          return errorRate < 1 ? Math.floor(currentQPS * 1.2) : currentQPS + 5;
        }
        
      case 'fibonacci':
        // Fibonacci-like sequence for gradual increase
        if (errorRate < 1) {
          const increment = Math.max(2, Math.floor(Math.log2(currentQPS + 1)) * 2);
          return currentQPS + increment;
        }
        if (errorRate < 5) return currentQPS + 2;
        return currentQPS;
        
      default:
        return currentQPS + 2;
    }
  }

  private calculateStepErrorRate(stepResult: QPSStepResult): { errorRate: number; criticalErrors: number } {
    const totalRequests = stepResult.totalRequests;
    const failedRequests = stepResult.failedRequests;
    const errorRate = totalRequests > 0 ? (failedRequests / totalRequests) * 100 : 0;
    
    // Count critical errors (5xx, timeouts, connections) - safely handle errorDetails
    const criticalErrorTypes = ['Server Error (5xx)', 'Timeout', 'Connection'];
    const errorDetails = stepResult.errorDetails || {};
    const criticalErrors = Object.entries(errorDetails)
      .filter(([type]) => criticalErrorTypes.includes(type))
      .reduce((sum, [, count]) => sum + count, 0);
    
    return { errorRate, criticalErrors };
  }

  private async runWorker(
    requestsPerWorker: number, 
    intervalMs: number, 
    endTime: number, 
    targetQPS: number, 
    results: RequestResult[]
  ): Promise<void> {
    let requestCount = 0;
    
    while (Date.now() < endTime && requestCount < requestsPerWorker && this.running) {
      const endpoint = this.selectEndpointByWeight();
      
      try {
        const result = await this.executeRequest(endpoint, targetQPS);
        results.push(result);
        this.storage.storeRequestResult(result);
        requestCount++;
        
        // Wait for next request with some jitter to avoid thundering herd
        const jitter = Math.random() * 0.2 - 0.1; // ±10% jitter
        const actualInterval = intervalMs * (1 + jitter);
        await new Promise(resolve => setTimeout(resolve, Math.max(1, actualInterval)));
      } catch (error) {
        this.logger.error(`Worker error: ${error}`);
        break;
      }
    }
  }

  private async runQPSStep(targetQPS: number): Promise<QPSStepResult> {
    const stepDuration = this.config.stepDurationSeconds;
    const stepStartTime = Date.now();
    const endTime = stepStartTime + (stepDuration * 1000);
    
    const results: RequestResult[] = [];
    const workers: Promise<void>[] = [];
    
    this.logger.info(`Starting QPS step: ${targetQPS} QPS for ${stepDuration}s`);
    
    // Use concurrent workers instead of sequential timing
    const workerCount = Math.min(targetQPS, 50); // Max 50 concurrent workers
    const requestsPerWorker = Math.ceil((targetQPS * stepDuration) / workerCount);
    const intervalMs = Math.max(10, 1000 / (targetQPS / workerCount)); // Min 10ms interval
    
    // Start concurrent workers
    for (let i = 0; i < workerCount; i++) {
      const worker = this.runWorker(requestsPerWorker, intervalMs, endTime, targetQPS, results);
      workers.push(worker);
    }
    
    // Wait for all workers to complete
    await Promise.allSettled(workers);
    
    const actualDuration = (Date.now() - stepStartTime) / 1000;
    const actualQPS = results.length / actualDuration;
    
    // Calculate endpoint-specific statistics
    const endpointStats: Record<string, any> = {};
    const errorDetails: Record<string, number> = {};
    
    for (const endpoint of this.config.endpoints.filter(ep => ep.enabled)) {
      const endpointResults = results.filter(r => r.endpoint === endpoint.name);
      const successful = endpointResults.filter(r => r.success);
      const failed = endpointResults.filter(r => !r.success);
      
      if (endpointResults.length > 0) {
        const responseTimes = endpointResults.map(r => r.responseTime);
        
        endpointStats[endpoint.name] = {
          total: endpointResults.length,
          successful: successful.length,
          failed: failed.length,
          avgResponseTime: responseTimes.reduce((sum, rt) => sum + rt, 0) / responseTimes.length,
          minResponseTime: Math.min(...responseTimes),
          maxResponseTime: Math.max(...responseTimes),
          errorRate: failed.length / endpointResults.length * 100,
          throughput: successful.length / actualDuration
        };
      }
      
      // Categorize errors
      failed.forEach(result => {
        if (result.error) {
          const errorType = this.categorizeError(result.error);
          errorDetails[errorType] = (errorDetails[errorType] || 0) + 1;
        }
      });
    }
    
    // Calculate response latency metrics
    const allResponseTimes = results.map(r => r.responseTime).sort((a, b) => a - b);
    const responseMetrics = {
      avgLatency: allResponseTimes.length > 0 ? allResponseTimes.reduce((sum, rt) => sum + rt, 0) / allResponseTimes.length : 0,
      minLatency: allResponseTimes.length > 0 ? Math.min(...allResponseTimes) : 0,
      maxLatency: allResponseTimes.length > 0 ? Math.max(...allResponseTimes) : 0,
      p50Latency: allResponseTimes.length > 0 ? allResponseTimes[Math.floor(allResponseTimes.length * 0.5)] : 0,
      p95Latency: allResponseTimes.length > 0 ? allResponseTimes[Math.floor(allResponseTimes.length * 0.95)] : 0,
      p99Latency: allResponseTimes.length > 0 ? allResponseTimes[Math.floor(allResponseTimes.length * 0.99)] : 0,
    };
    
    const totalSuccessful = results.filter(r => r.success).length;
    const totalFailed = results.filter(r => !r.success).length;
    
    const { errorRate, criticalErrors } = this.calculateStepErrorRate({
      totalRequests: results.length,
      failedRequests: totalFailed
    } as QPSStepResult);
    
    const shouldStop = errorRate > 20 || criticalErrors > (results.length * 0.1);
    const stopReason = shouldStop ? 
      (errorRate > 20 ? 'High error rate detected' : 'Too many critical errors') : 
      undefined;
    
    const stepResult: QPSStepResult = {
      qps: targetQPS,
      actualQPS: Math.round(actualQPS * 10) / 10,
      duration: Math.round(actualDuration),
      totalRequests: results.length,
      successfulRequests: totalSuccessful,
      failedRequests: totalFailed,
      endpointStats,
      responseMetrics,
      errorDetails,
      shouldStop,
      stopReason
    };
    
    this.logger.info(`QPS step completed`, {
      targetQPS,
      actualQPS: stepResult.actualQPS,
      totalRequests: stepResult.totalRequests,
      successRate: totalSuccessful / results.length * 100,
      errorRate
    });
    
    // Store step result
    this.storage.storeStepResult(stepResult);
    
    return stepResult;
  }

  public async runStressTest(): Promise<TestSessionResult> {
    const startTime = Date.now();
    
    this.logger.info('🚀 Starting Advanced Omni Stress Test', {
      sessionId: this.sessionId,
      config: this.config
    });
    
    // Test connectivity first
    await this.testConnectivity();
    
    const results: QPSStepResult[] = [];
    let currentQPS = 5; // Start higher for blockchain-level testing
    let maxSustainableQPS = 0;
    this.running = true;
    
    // Setup graceful shutdown
    process.on('SIGINT', () => {
      this.logger.warn('Test interrupted by user');
      this.running = false;
    });
    
    // Main test loop - start higher for blockchain-level testing
    while (this.running && Date.now() - startTime < this.config.testDurationMinutes * 60 * 1000) {
      const stepResult = await this.runQPSStep(currentQPS);
      results.push(stepResult);
      
      // Display real-time results
      this.displayStepResult(stepResult);
      
      if (stepResult.shouldStop) {
        this.logger.warn(`Stopping test: ${stepResult.stopReason}`);
        break;
      }
      
      const { errorRate } = this.calculateStepErrorRate(stepResult);
      if (errorRate < 5) {
        maxSustainableQPS = currentQPS;
      }
      
      // Calculate next QPS - more aggressive scaling for higher targets
      const nextQPS = this.calculateNextQPS(currentQPS, stepResult);
      if (nextQPS === currentQPS && errorRate > 5) {
        this.logger.info('Max QPS reached, stopping test');
        break;
      }
      
      currentQPS = nextQPS;
      
      // Stop if we've reached a very high QPS to avoid runaway
      if (currentQPS > 1000) {
        this.logger.info('Reached maximum test limit (1000 QPS), stopping test');
        break;
      }
      
      if (!this.running) break;
    }
    
    const endTime = Date.now();
    const recommendedMaxQPS = Math.floor(maxSustainableQPS * (this.config.maxQPSPercentage / 100));
    
    // Calculate overall statistics
    const allRequests = results.reduce((sum, step) => sum + step.totalRequests, 0);
    const allSuccessful = results.reduce((sum, step) => sum + step.successfulRequests, 0);
    const allFailed = results.reduce((sum, step) => sum + step.failedRequests, 0);
    
    const sessionResult: TestSessionResult = {
      sessionId: this.sessionId,
      startTime,
      endTime,
      config: this.config,
      steps: results,
      maxSustainableQPS,
      recommendedMaxQPS,
      overallStats: {
        totalRequests: allRequests,
        totalSuccessful: allSuccessful,
        totalFailed: allFailed,
        overallSuccessRate: allRequests > 0 ? (allSuccessful / allRequests) * 100 : 0,
        avgResponseTime: 0, // Will be calculated by analyzer
        maxResponseTime: 0, // Will be calculated by analyzer
        minResponseTime: 0  // Will be calculated by analyzer
      }
    };
    
    // Store final session result
    this.storage.storeSessionResult(sessionResult);
    
    // Generate analysis and reports
    const analysis = await this.analyzer.analyze(sessionResult);
    this.storage.storeAnalysis(analysis);
    
    // Start dashboard server
    await this.dashboard.start();
    
    this.logger.info('✅ Stress test completed', {
      duration: endTime - startTime,
      totalSteps: results.length,
      maxSustainableQPS,
      recommendedMaxQPS,
      totalRequests: allRequests,
      successRate: sessionResult.overallStats.overallSuccessRate
    });
    
    this.displayFinalResults(sessionResult);
    
    return sessionResult;
  }

  private async testConnectivity(): Promise<void> {
    this.logger.info('Testing connectivity to all endpoints...');
    
    const enabledEndpoints = this.config.endpoints.filter(ep => ep.enabled);
    
    for (const endpoint of enabledEndpoints) {
      try {
        const params = endpoint.paramGenerator();
        const response = await this.rpcClient.call(endpoint.method, params);
        const isValid = endpoint.validator(response);
        
        if (isValid) {
          this.logger.info(`✅ ${endpoint.name} connectivity test passed`);
        } else {
          this.logger.warn(`⚠️ ${endpoint.name} returned invalid response`, { response });
        }
      } catch (error) {
        this.logger.error(`❌ ${endpoint.name} connectivity test failed`, { error });
        throw new Error(`Connectivity test failed for ${endpoint.name}: ${error}`);
      }
    }
    
    this.logger.info('All connectivity tests passed');
  }

  private displayStepResult(result: QPSStepResult): void {
    const { errorRate } = this.calculateStepErrorRate(result);
    
    const status = errorRate < 1 ? chalk.green('✅ Excellent') :
                   errorRate < 5 ? chalk.yellow('⚠️ Good') :
                   errorRate < 15 ? chalk.red('❌ Poor') :
                   chalk.red('🚨 Critical');
    
    // Safely handle endpointStats
    const endpointStats = result.endpointStats || {};
    const statsValues = Object.values(endpointStats);
    const avgResponseTime = statsValues.length > 0 
      ? Math.round(statsValues.reduce((sum, stat) => sum + stat.avgResponseTime, 0) / statsValues.length)
      : 0;
    
    console.log(
      `QPS ${result.actualQPS.toString().padStart(6)}: ` +
      `${result.successfulRequests.toString().padStart(4)}/${result.totalRequests.toString().padStart(4)} requests, ` +
      `${errorRate.toFixed(1).padStart(5)}% errors, ` +
      `${avgResponseTime}ms avg ${status}`
    );
  }

  private displayFinalResults(sessionResult: TestSessionResult): void {
    console.log('\n' + chalk.blue.bold('📊 Final Stress Test Results'));
    console.log('='.repeat(80));
    
    console.log(chalk.green(`Session ID: ${sessionResult.sessionId}`));
    console.log(chalk.green(`Test Duration: ${((sessionResult.endTime - sessionResult.startTime) / 1000 / 60).toFixed(1)} minutes`));
    console.log(chalk.green(`Max Sustainable QPS: ${sessionResult.maxSustainableQPS}`));
    console.log(chalk.green(`Recommended Production QPS (${this.config.maxQPSPercentage}%): ${sessionResult.recommendedMaxQPS}`));
    console.log(chalk.green(`Total Requests: ${sessionResult.overallStats.totalRequests.toLocaleString()}`));
    console.log(chalk.green(`Overall Success Rate: ${sessionResult.overallStats.overallSuccessRate.toFixed(2)}%`));
    
    console.log('\n' + chalk.blue.bold('📈 Endpoint Performance Summary'));
    console.log('-'.repeat(80));
    
    // Display endpoint-specific summary
    const endpointSummary: Record<string, any> = {};
    for (const step of sessionResult.steps) {
      // Safely handle endpointStats
      const endpointStats = step.endpointStats || {};
      for (const [endpoint, stats] of Object.entries(endpointStats)) {
        if (!endpointSummary[endpoint]) {
          endpointSummary[endpoint] = {
            total: 0,
            successful: 0,
            failed: 0,
            totalResponseTime: 0,
            minResponseTime: Infinity,
            maxResponseTime: 0
          };
        }
        
        const summary = endpointSummary[endpoint];
        summary.total += stats.total;
        summary.successful += stats.successful;
        summary.failed += stats.failed;
        summary.totalResponseTime += stats.avgResponseTime * stats.total;
        summary.minResponseTime = Math.min(summary.minResponseTime, stats.minResponseTime);
        summary.maxResponseTime = Math.max(summary.maxResponseTime, stats.maxResponseTime);
      }
    }
    
    for (const [endpoint, summary] of Object.entries(endpointSummary)) {
      const successRate = (summary.successful / summary.total * 100).toFixed(1);
      const avgResponseTime = (summary.totalResponseTime / summary.total).toFixed(1);
      
      console.log(
        `${endpoint.padEnd(25)}: ` +
        `${summary.successful.toString().padStart(6)}/${summary.total.toString().padStart(6)} ` +
        `(${successRate.padStart(5)}%) ` +
        `${avgResponseTime.padStart(6)}ms avg`
      );
    }
    
    console.log('\n' + chalk.blue.bold('💾 Results Storage'));
    console.log('-'.repeat(80));
    console.log(`Results saved to: ${this.config.outputDir}/${this.sessionId}/`);
    console.log(`Dashboard available at: http://localhost:3000`);
    
    console.log('\n' + chalk.yellow('💡 Recommendations'));
    console.log('-'.repeat(80));
    console.log(`• Production safe QPS: ${chalk.green(sessionResult.recommendedMaxQPS)}`);
    console.log(`• Warning threshold: ${chalk.yellow(Math.floor(sessionResult.maxSustainableQPS * 0.9))} (90% of max)`);
    console.log(`• Critical threshold: ${chalk.red(sessionResult.maxSustainableQPS)} (maximum sustainable)`);
    console.log(`• Monitor error rates and response times continuously`);
    console.log(`• Consider load balancing if targeting higher QPS`);
  }

  // Public API for configuration
  public setEndpointWeight(endpointName: string, weight: number): void {
    const endpoint = this.config.endpoints.find(ep => ep.name === endpointName);
    if (endpoint) {
      endpoint.weight = weight;
      this.logger.info(`Updated ${endpointName} weight to ${weight}%`);
    }
  }

  public enableEndpoint(endpointName: string, enabled: boolean = true): void {
    const endpoint = this.config.endpoints.find(ep => ep.name === endpointName);
    if (endpoint) {
      endpoint.enabled = enabled;
      this.logger.info(`${enabled ? 'Enabled' : 'Disabled'} endpoint: ${endpointName}`);
    }
  }

  public getConfig(): TestConfig {
    return { ...this.config };
  }
}

// CLI execution
async function main() {
  const targetUrl = process.argv[2] || process.env.OMNI_RPC_URL || 'http://localhost:2100/';
  
  const config: Partial<TestConfig> = {
    targetUrl,
    maxQPSPercentage: 70,
    stepDurationSeconds: 20,
    rampUpStrategy: 'exponential',
    testDurationMinutes: 10,
    walletPoolSize: 500,
    outputDir: './stress-test-results'
  };
  
  try {
    const test = new AdvancedStressTest(config);
    await test.runStressTest();
  } catch (error) {
    console.error(chalk.red('❌ Test failed:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Run if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { AdvancedStressTest, TestConfig, TestEndpoint, RequestResult, QPSStepResult, TestSessionResult };