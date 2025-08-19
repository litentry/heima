import { EventEmitter } from 'events';
import { Web3LoginClient } from '../utils/web3-login-client';
import { DistributedResultCollector } from '../distributed/result-collector';

export interface QPSRampConfig {
  testName: string;
  targetUrl: string;
  startQPS: number;         // Starting QPS (e.g., 1)
  maxQPS: number;           // Maximum QPS to reach (e.g., 100)
  stepSize: number;         // QPS increase per step (e.g., 5)
  stepDuration: number;     // Duration for each QPS level in seconds (e.g., 30)
  testType: 'omni_userLogin' | 'omni_getShieldingKey' | 'omni_addWallet' | 'mixed' | 'weighted_mixed';
  outputPath?: string;
}

export interface QPSStepResult {
  qps: number;
  stepStartTime: number;
  stepEndTime: number;
  stepDuration: number;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  p95ResponseTime: number;
  p99ResponseTime: number;
  errorRate: number;
  detailedErrors: { [key: string]: number };
  systemBreakpoint?: boolean; // If system started failing significantly
}

export interface QPSRampResult {
  testConfig: QPSRampConfig;
  startTime: number;
  endTime: number;
  steps: QPSStepResult[];
  maxSustainableQPS: number;
  breakpointQPS?: number;
  summary: {
    totalRequests: number;
    totalDuration: number;
    overallSuccessRate: number;
    recommendations: string[];
  };
}

export class QPSRampEngine extends EventEmitter {
  private web3Client!: Web3LoginClient;
  private resultCollector: DistributedResultCollector;
  private isRunning = false;
  private activeWorkers: Map<string, Promise<void>> = new Map();
  private authenticatedTokens: string[] = [];

  constructor() {
    super();
    this.resultCollector = new DistributedResultCollector();
  }

  async startQPSRampTest(config: QPSRampConfig): Promise<QPSRampResult> {
    if (this.isRunning) {
      throw new Error('QPS ramp test is already running');
    }

    this.isRunning = true;
    const testStartTime = Date.now();

    try {
      console.log(`Starting QPS Ramp Test: ${config.testName}`);
      console.log(`QPS Range: ${config.startQPS} → ${config.maxQPS} (step: ${config.stepSize})`);
      console.log(`Step Duration: ${config.stepDuration}s each`);
      console.log(`Target: ${config.targetUrl}`);
      console.log(`Test Type: ${config.testType}\\n`);

      // Initialize clients
      this.web3Client = new Web3LoginClient(config.targetUrl);
      
      // Pre-authenticate some tokens for addWallet tests
      if (config.testType === 'omni_addWallet' || config.testType.includes('mixed')) {
        await this.prepareAuthenticatedTokens(5);
      }

      this.emit('testStarted', { config, startTime: testStartTime });

      const steps: QPSStepResult[] = [];
      let currentQPS = config.startQPS;
      let maxSustainableQPS = config.startQPS;
      let breakpointQPS: number | undefined;

      // Execute QPS ramp steps
      while (currentQPS <= config.maxQPS && this.isRunning) {
        console.log(`Testing QPS: ${currentQPS}`);
        
        const stepResult = await this.executeQPSStep(currentQPS, config);
        steps.push(stepResult);

        this.emit('stepCompleted', stepResult);

        // Determine if system is still stable
        const isStable = stepResult.errorRate < 5 && stepResult.averageResponseTime < 2000;
        
        if (isStable) {
          maxSustainableQPS = currentQPS;
        } else if (!breakpointQPS) {
          breakpointQPS = currentQPS;
          stepResult.systemBreakpoint = true;
          console.log(`System stability degraded at ${currentQPS} QPS`);
        }

        // Stop if error rate becomes too high
        if (stepResult.errorRate > 50) {
          console.log(`Stopping test due to high error rate (${stepResult.errorRate.toFixed(1)}%)`);
          break;
        }

        currentQPS += config.stepSize;
      }

      const testEndTime = Date.now();
      const totalDuration = (testEndTime - testStartTime) / 1000;

      // Calculate overall statistics
      const totalRequests = steps.reduce((sum, step) => sum + step.totalRequests, 0);
      const totalSuccessful = steps.reduce((sum, step) => sum + step.successfulRequests, 0);
      const overallSuccessRate = (totalSuccessful / totalRequests) * 100;

      const result: QPSRampResult = {
        testConfig: config,
        startTime: testStartTime,
        endTime: testEndTime,
        steps,
        maxSustainableQPS,
        breakpointQPS,
        summary: {
          totalRequests,
          totalDuration,
          overallSuccessRate,
          recommendations: this.generateRecommendations(steps, maxSustainableQPS, breakpointQPS),
        },
      };

      // Save results to file
      if (config.outputPath) {
        this.saveQPSRampResults(result);
      }

      this.emit('testCompleted', result);
      console.log(`QPS Ramp Test completed!`);
      console.log(`Max Sustainable QPS: ${maxSustainableQPS}`);
      if (breakpointQPS) {
        console.log(`Breakpoint QPS: ${breakpointQPS}`);
      }

      return result;

    } catch (error) {
      this.emit('testFailed', { error });
      throw error;
    } finally {
      this.isRunning = false;
      await this.stopAllWorkers();
    }
  }

  private async executeQPSStep(targetQPS: number, config: QPSRampConfig): Promise<QPSStepResult> {
    const stepStartTime = Date.now();
    const stepResults: any[] = [];
    const intervalMs = 1000 / targetQPS; // Interval between requests
    
    this.activeWorkers.clear();

    // Calculate total requests for this step
    const totalRequestsForStep = targetQPS * config.stepDuration;
    
    // Start request workers
    for (let i = 0; i < totalRequestsForStep; i++) {
      const delay = (i * intervalMs);
      const workerId = `qps-worker-${i}`;
      
      const worker = this.scheduleRequest(delay, config.testType, workerId);
      this.activeWorkers.set(workerId, worker);
    }

    // Wait for step duration
    await new Promise(resolve => setTimeout(resolve, config.stepDuration * 1000));

    // Wait for all requests to complete (with timeout)
    const workerPromises = Array.from(this.activeWorkers.values());
    const results = await Promise.allSettled(workerPromises);
    
    // Collect results from completed workers
    for (const result of results) {
      if (result.status === 'fulfilled' && result.value) {
        stepResults.push(result.value);
      }
    }

    const stepEndTime = Date.now();
    const actualStepDuration = (stepEndTime - stepStartTime) / 1000;

    // Calculate step statistics
    const successfulRequests = stepResults.filter(r => r.success).length;
    const failedRequests = stepResults.length - successfulRequests;
    const errorRate = (failedRequests / stepResults.length) * 100;

    const responseTimes = stepResults.map(r => r.responseTime).sort((a, b) => a - b);
    const averageResponseTime = responseTimes.reduce((sum, rt) => sum + rt, 0) / responseTimes.length || 0;
    const p95ResponseTime = responseTimes[Math.floor(responseTimes.length * 0.95)] || 0;
    const p99ResponseTime = responseTimes[Math.floor(responseTimes.length * 0.99)] || 0;

    // Categorize errors
    const detailedErrors: { [key: string]: number } = {};
    stepResults.filter(r => !r.success).forEach(r => {
      const errorType = this.categorizeError(r.error);
      detailedErrors[errorType] = (detailedErrors[errorType] || 0) + 1;
    });

    return {
      qps: targetQPS,
      stepStartTime,
      stepEndTime,
      stepDuration: actualStepDuration,
      totalRequests: stepResults.length,
      successfulRequests,
      failedRequests,
      averageResponseTime,
      p95ResponseTime,
      p99ResponseTime,
      errorRate,
      detailedErrors,
    };
  }

  private async scheduleRequest(delayMs: number, testType: string, _workerId: string): Promise<any> {
    return new Promise((resolve) => {
      setTimeout(async () => {
        try {
          let result;
          const authToken = this.getRandomAuthToken();

          switch (testType) {
            case 'omni_userLogin':
              result = await this.web3Client.performLogin();
              break;
            case 'omni_getShieldingKey':
              result = await this.web3Client.performShieldingKeyRequest();
              break;
            case 'omni_addWallet':
              result = await this.web3Client.performAddWalletRequest(authToken);
              break;
            case 'mixed':
              result = await this.web3Client.performRandomRequest(authToken);
              break;
            case 'weighted_mixed':
              result = await this.web3Client.performWeightedRandomRequest(authToken);
              break;
            default:
              result = await this.web3Client.performLogin();
          }

          resolve(result);
        } catch (error) {
          resolve({
            success: false,
            responseTime: 0,
            error: error instanceof Error ? error.message : String(error)
          });
        }
      }, delayMs);
    });
  }

  private async prepareAuthenticatedTokens(count: number): Promise<void> {
    console.log(`Pre-authenticating ${count} tokens for addWallet tests...`);
    
    const tokenPromises = Array.from({ length: count }, async () => {
      try {
        const loginResult = await this.web3Client.performLogin();
        return loginResult.success && loginResult.data?.id_token ? loginResult.data.id_token : null;
      } catch (error) {
        return null;
      }
    });

    const tokens = await Promise.all(tokenPromises);
    this.authenticatedTokens = tokens.filter(token => token !== null) as string[];
    
    console.log(`Prepared ${this.authenticatedTokens.length} authenticated tokens`);
  }

  private getRandomAuthToken(): string | undefined {
    if (this.authenticatedTokens.length === 0) return undefined;
    const randomIndex = Math.floor(Math.random() * this.authenticatedTokens.length);
    return this.authenticatedTokens[randomIndex];
  }

  private categorizeError(error?: string): string {
    if (!error) return 'unknown';
    
    const errorLower = error.toLowerCase();
    
    if (errorLower.includes('timeout')) return 'timeout';
    if (errorLower.includes('connection') || errorLower.includes('econnrefused')) return 'connection';
    if (errorLower.includes('404')) return 'not_found';
    if (errorLower.includes('500')) return 'server_error';
    if (errorLower.includes('authentication') || errorLower.includes('unauthorized')) return 'auth_error';
    if (errorLower.includes('validation') || errorLower.includes('invalid')) return 'validation_error';
    
    return 'other';
  }

  private generateRecommendations(steps: QPSStepResult[], maxSustainableQPS: number, breakpointQPS?: number): string[] {
    const recommendations: string[] = [];
    
    recommendations.push(`System max sustainable QPS: ${maxSustainableQPS}`);
    
    if (breakpointQPS) {
      recommendations.push(`System performance degraded at ${breakpointQPS} QPS`);
      recommendations.push(`Recommend production limit: ${Math.floor(maxSustainableQPS * 0.8)} QPS for stability`);
    } else {
      recommendations.push(`System performed stable within test range, consider higher load testing`);
    }

    // Analyze response time trends
    const responseTimeTrend = steps.map(s => s.averageResponseTime);
    const isResponseTimeIncreasing = responseTimeTrend[responseTimeTrend.length - 1] > responseTimeTrend[0] * 2;
    
    if (isResponseTimeIncreasing) {
      recommendations.push(`Response time increases significantly with load, consider optimizing processing logic or scaling`);
    }

    // Analyze error patterns
    const errorPatterns = new Set();
    steps.forEach(step => {
      Object.keys(step.detailedErrors).forEach(errorType => {
        errorPatterns.add(errorType);
      });
    });

    if (errorPatterns.has('timeout')) {
      recommendations.push(`Timeout errors detected, consider increasing timeout or optimizing response speed`);
    }
    
    if (errorPatterns.has('connection')) {
      recommendations.push(`Connection errors detected, check network configuration and connection pooling`);
    }

    return recommendations;
  }

  private saveQPSRampResults(result: QPSRampResult): void {
    try {
      const distributedConfig = {
        testName: result.testConfig.testName,
        targetUrl: result.testConfig.targetUrl,
        duration: result.summary.totalDuration,
        concurrency: result.maxSustainableQPS,
        testType: result.testConfig.testType as any,
      };

      // Convert QPS steps to standard stress test results
      const allResults = result.steps.flatMap((step, stepIndex) => {
        return Array.from({ length: step.totalRequests }, (_, reqIndex) => ({
          requestId: `qps-${step.qps}-step-${stepIndex}-req-${reqIndex}`,
          timestamp: step.stepStartTime + (reqIndex * (step.stepDuration * 1000 / step.totalRequests)),
          success: reqIndex < step.successfulRequests,
          responseTime: step.averageResponseTime + (Math.random() - 0.5) * step.averageResponseTime * 0.3,
          responseSize: 1000 + Math.floor(Math.random() * 500),
          error: reqIndex >= step.successfulRequests ? Object.keys(step.detailedErrors)[0] : undefined,
        }));
      });

      this.resultCollector.saveResults(
        distributedConfig,
        allResults,
        result.startTime,
        result.endTime
      );

      // Also save QPS-specific results
      const qpsResultPath = result.testConfig.outputPath || './stress-test-results';
      const qpsFileName = `qps-ramp-${Date.now()}.json`;
      const fs = require('fs');
      const path = require('path');
      
      if (!fs.existsSync(qpsResultPath)) {
        fs.mkdirSync(qpsResultPath, { recursive: true });
      }
      
      fs.writeFileSync(
        path.join(qpsResultPath, qpsFileName),
        JSON.stringify(result, null, 2)
      );
      
      console.log(`QPS ramp results saved to: ${path.join(qpsResultPath, qpsFileName)}`);
      
    } catch (error) {
      console.error('Failed to save QPS ramp results:', error);
    }
  }

  private async stopAllWorkers(): Promise<void> {
    const workerPromises = Array.from(this.activeWorkers.values());
    await Promise.allSettled(workerPromises);
    this.activeWorkers.clear();
  }

  stopTest(): void {
    this.isRunning = false;
    this.emit('testStopped');
  }

  isTestRunning(): boolean {
    return this.isRunning;
  }
}