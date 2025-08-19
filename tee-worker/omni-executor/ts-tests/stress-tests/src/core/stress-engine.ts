import { EventEmitter } from 'node:events';
import { StressTestConfig, RequestResult, TestSession } from '../types/index';
import { PerformanceCollector } from '../metrics/performance-collector';
import { SystemMonitor } from '../metrics/system-monitor';
import { DataStorage } from '../storage/data-storage';
import { Web3LoginClient } from '../utils/web3-login-client';

export class StressTestEngine extends EventEmitter {
  private session: TestSession | null = null;
  private performanceCollector: PerformanceCollector;
  private systemMonitor: SystemMonitor;
  private dataStorage: DataStorage;
  private web3Client: Web3LoginClient;
  private running = false;
  private workers: Promise<void>[] = [];
  private startTime = 0;

  constructor() {
    super();
    this.performanceCollector = new PerformanceCollector();
    this.systemMonitor = new SystemMonitor();
    this.dataStorage = new DataStorage();
    this.web3Client = new Web3LoginClient();
  }

  async startTest(config: StressTestConfig): Promise<string> {
    if (this.running) {
      throw new Error('Test already running');
    }

    const sessionId = this.generateSessionId();
    this.session = {
      sessionId,
      config,
      startTime: Date.now(),
      status: 'preparing',
      results: [],
      systemMetrics: []
    };

    try {
      this.emit('testStarting', { sessionId, config });
      
      // Initialize monitoring
      await this.systemMonitor.start();
      this.performanceCollector.start();
      
      // Setup periodic metrics collection
      this.setupMetricsCollection();
      
      // Start the test
      this.session.status = 'running';
      this.running = true;
      this.startTime = Date.now();
      
      this.emit('testStarted', { sessionId });
      
      // Execute the stress test
      await this.executeStressTest(config);
      
      // Complete the test
      await this.completeTest();
      
      return sessionId;
    } catch (error) {
      this.session.status = 'failed';
      this.emit('testFailed', { sessionId, error });
      throw error;
    }
  }

  async stopTest(): Promise<void> {
    if (!this.running || !this.session) {
      return;
    }

    this.running = false;
    this.session.status = 'cancelled';
    
    // Wait for all workers to complete
    await Promise.allSettled(this.workers);
    
    await this.completeTest();
  }

  getSession(): TestSession | null {
    return this.session;
  }

  private async executeStressTest(config: StressTestConfig): Promise<void> {
    const { duration, concurrency, rampUpTime } = config;
    
    // Calculate ramp-up strategy
    const rampUpIntervalMs = (rampUpTime * 1000) / concurrency;
    const testEndTime = Date.now() + (duration * 1000);
    
    // Start workers with ramp-up
    for (let i = 0; i < concurrency; i++) {
      setTimeout(() => {
        if (this.running) {
          const worker = this.createWorker(i, testEndTime);
          this.workers.push(worker);
        }
      }, i * rampUpIntervalMs);
    }
    
    // Wait for test duration
    await new Promise<void>((resolve) => {
      const checkInterval = setInterval(() => {
        if (!this.running || Date.now() >= testEndTime) {
          this.running = false;
          clearInterval(checkInterval);
          resolve();
        }
      }, 100);
    });
    
    // Wait for all workers to complete
    await Promise.allSettled(this.workers);
  }

  private async createWorker(workerId: number, endTime: number): Promise<void> {
    while (this.running && Date.now() < endTime) {
      // Execute request with guaranteed non-throwing behavior
      const result = await this.executeRequestSafely(workerId);
      
      if (this.session) {
        this.session.results.push(result);
        this.performanceCollector.recordRequest(result);
      }
      
      if (result.success) {
        this.emit('requestCompleted', result);
      } else {
        this.emit('requestFailed', result);
      }
      
      // Small delay to prevent overwhelming, but don't stop on errors
      await new Promise(resolve => setTimeout(resolve, 10));
    }
  }

  private async executeRequestSafely(workerId: number): Promise<RequestResult> {
    const requestStart = performance.now();
    const requestId = `worker-${workerId}-${Date.now()}`;
    
    try {
      // Web3Client.performLogin() is designed to never throw, always returns a result
      const result = await this.web3Client.performLogin();
      const responseTime = performance.now() - requestStart;
      
      return {
        timestamp: Date.now(),
        requestId,
        method: 'POST',
        url: this.session?.config.targetUrl || 'omni-executor-rpc',
        statusCode: result.success ? 200 : 500,
        responseTime: result.responseTime || responseTime,
        success: result.success,
        error: result.error,
        requestSize: result.requestSize || 0,
        responseSize: result.responseSize || 0
      };
    } catch (error) {
      // Fallback: even if performLogin somehow throws, catch it
      const responseTime = performance.now() - requestStart;
      
      return {
        timestamp: Date.now(),
        requestId,
        method: 'POST',
        url: this.session?.config.targetUrl || 'omni-executor-rpc',
        statusCode: 0,
        responseTime,
        success: false,
        error: `Unexpected error in performLogin: ${error instanceof Error ? error.message : String(error)}`,
        requestSize: 0,
        responseSize: 0
      };
    }
  }

  // Legacy method kept for compatibility, but now redirects to safe version
  private async executeRequest(workerId: number): Promise<RequestResult> {
    return this.executeRequestSafely(workerId);
  }

  private setupMetricsCollection(): void {
    if (!this.session) return;
    
    const interval = setInterval(async () => {
      if (!this.running || !this.session) {
        clearInterval(interval);
        return;
      }
      
      try {
        const metrics = await this.systemMonitor.collectMetrics();
        this.session.systemMetrics.push(metrics);
        this.emit('metricsCollected', metrics);
      } catch (error) {
        this.emit('metricsError', error);
      }
    }, 1000); // Collect metrics every second
  }

  private async completeTest(): Promise<void> {
    if (!this.session) return;
    
    this.session.endTime = Date.now();
    this.session.status = this.session.status === 'cancelled' ? 'cancelled' : 'completed';
    
    // Stop monitoring
    await this.systemMonitor.stop();
    this.performanceCollector.stop();
    
    // Generate summary
    this.session.summary = this.performanceCollector.generateSummary(
      this.session.results,
      this.session.systemMetrics
    );
    
    // Save data
    await this.dataStorage.saveTestSession(this.session);
    
    this.emit('testCompleted', { 
      sessionId: this.session.sessionId, 
      summary: this.session.summary 
    });
  }

  private generateSessionId(): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2);
    return `stress-test-${timestamp}-${random}`;
  }
}