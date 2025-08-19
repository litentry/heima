import { RequestResult, SystemMetrics, TestSummary } from '../types/index';

export class PerformanceCollector {
  private isRunning = false;
  private requestHistory: RequestResult[] = [];
  private startTime = 0;

  start(): void {
    this.isRunning = true;
    this.startTime = Date.now();
    this.requestHistory = [];
  }

  stop(): void {
    this.isRunning = false;
  }

  recordRequest(result: RequestResult): void {
    if (!this.isRunning) return;
    
    this.requestHistory.push(result);
  }

  generateSummary(results: RequestResult[], systemMetrics: SystemMetrics[]): TestSummary {
    const successfulRequests = results.filter(r => r.success);
    const failedRequests = results.filter(r => !r.success);
    
    // Calculate response time metrics
    const responseTimes = successfulRequests.map(r => r.responseTime);
    const responseTimeMetrics = this.calculatePercentiles(responseTimes);
    
    // Calculate throughput metrics
    const testDuration = this.getTestDuration(results);
    const throughputMetrics = this.calculateThroughput(results, testDuration);
    
    // Calculate error rate metrics
    const errorMetrics = this.calculateErrorRates(results);
    
    // Calculate system resource summary
    const systemSummary = this.calculateSystemSummary(systemMetrics);
    
    return {
      responseTime: {
        average: this.calculateAverage(responseTimes),
        min: Math.min(...responseTimes) || 0,
        max: Math.max(...responseTimes) || 0,
        p50: responseTimeMetrics.p50,
        p90: responseTimeMetrics.p90,
        p95: responseTimeMetrics.p95,
        p99: responseTimeMetrics.p99
      },
      throughput: {
        rps: throughputMetrics.rps,
        tps: throughputMetrics.tps,
        dataTransferRate: throughputMetrics.dataTransferRate,
        peakConcurrency: throughputMetrics.peakConcurrency
      },
      errorRate: {
        total: errorMetrics.total,
        http4xx: errorMetrics.http4xx,
        http5xx: errorMetrics.http5xx,
        timeout: errorMetrics.timeout,
        connection: errorMetrics.connection,
        errorsByType: errorMetrics.errorsByType
      },
      systemSummary,
      totalRequests: results.length,
      successfulRequests: successfulRequests.length,
      failedRequests: failedRequests.length,
      testDuration
    };
  }

  private calculatePercentiles(values: number[]): {
    p50: number;
    p90: number;
    p95: number;
    p99: number;
  } {
    if (values.length === 0) {
      return { p50: 0, p90: 0, p95: 0, p99: 0 };
    }
    
    const sorted = [...values].sort((a, b) => a - b);
    const length = sorted.length;
    
    return {
      p50: this.getPercentile(sorted, 0.5),
      p90: this.getPercentile(sorted, 0.9),
      p95: this.getPercentile(sorted, 0.95),
      p99: this.getPercentile(sorted, 0.99)
    };
  }

  private getPercentile(sortedValues: number[], percentile: number): number {
    const index = Math.ceil(sortedValues.length * percentile) - 1;
    return sortedValues[Math.max(0, index)] || 0;
  }

  private calculateAverage(values: number[]): number {
    if (values.length === 0) return 0;
    return values.reduce((sum, value) => sum + value, 0) / values.length;
  }

  private getTestDuration(results: RequestResult[]): number {
    if (results.length === 0) return 0;
    
    const timestamps = results.map(r => r.timestamp);
    const minTime = Math.min(...timestamps);
    const maxTime = Math.max(...timestamps);
    
    return (maxTime - minTime) / 1000; // Convert to seconds
  }

  private calculateThroughput(results: RequestResult[], testDuration: number): {
    rps: number;
    tps: number;
    dataTransferRate: number;
    peakConcurrency: number;
  } {
    if (testDuration === 0) {
      return { rps: 0, tps: 0, dataTransferRate: 0, peakConcurrency: 0 };
    }
    
    const successfulRequests = results.filter(r => r.success);
    const totalDataTransferred = results.reduce((sum, r) => sum + r.requestSize + r.responseSize, 0);
    
    // Calculate peak concurrency by analyzing concurrent requests in 1-second windows
    const peakConcurrency = this.calculatePeakConcurrency(results);
    
    return {
      rps: results.length / testDuration,
      tps: successfulRequests.length / testDuration,
      dataTransferRate: totalDataTransferred / testDuration, // bytes per second
      peakConcurrency
    };
  }

  private calculatePeakConcurrency(results: RequestResult[]): number {
    if (results.length === 0) return 0;
    
    // Group requests by second and count concurrent requests
    const concurrencyMap = new Map<number, number>();
    
    results.forEach(result => {
      const second = Math.floor(result.timestamp / 1000);
      const requestStart = second;
      const requestEnd = Math.floor((result.timestamp + result.responseTime) / 1000);
      
      // Count concurrent requests for each second
      for (let s = requestStart; s <= requestEnd; s++) {
        concurrencyMap.set(s, (concurrencyMap.get(s) || 0) + 1);
      }
    });
    
    return Math.max(...concurrencyMap.values(), 0);
  }

  private calculateErrorRates(results: RequestResult[]): {
    total: number;
    http4xx: number;
    http5xx: number;
    timeout: number;
    connection: number;
    errorsByType: Record<string, number>;
  } {
    const totalRequests = results.length;
    if (totalRequests === 0) {
      return { total: 0, http4xx: 0, http5xx: 0, timeout: 0, connection: 0, errorsByType: {} };
    }
    
    const failedRequests = results.filter(r => !r.success);
    const http4xxErrors = results.filter(r => r.statusCode >= 400 && r.statusCode < 500);
    const http5xxErrors = results.filter(r => r.statusCode >= 500 && r.statusCode < 600);
    const timeoutErrors = results.filter(r => r.error?.includes('timeout') || r.error?.includes('TIMEOUT'));
    const connectionErrors = results.filter(r => 
      r.error?.includes('connection') || 
      r.error?.includes('ECONNREFUSED') || 
      r.error?.includes('ENOTFOUND')
    );
    
    // Count errors by type
    const errorsByType: Record<string, number> = {};
    failedRequests.forEach(result => {
      if (result.error) {
        const errorType = this.categorizeError(result.error);
        errorsByType[errorType] = (errorsByType[errorType] || 0) + 1;
      }
    });
    
    return {
      total: (failedRequests.length / totalRequests) * 100,
      http4xx: (http4xxErrors.length / totalRequests) * 100,
      http5xx: (http5xxErrors.length / totalRequests) * 100,
      timeout: (timeoutErrors.length / totalRequests) * 100,
      connection: (connectionErrors.length / totalRequests) * 100,
      errorsByType
    };
  }

  private categorizeError(error: string): string {
    const errorLower = error.toLowerCase();
    
    if (errorLower.includes('timeout')) return 'Timeout';
    if (errorLower.includes('connection') || errorLower.includes('econnrefused')) return 'Connection';
    if (errorLower.includes('dns') || errorLower.includes('enotfound')) return 'DNS';
    if (errorLower.includes('ssl') || errorLower.includes('tls')) return 'SSL/TLS';
    if (errorLower.includes('auth')) return 'Authentication';
    if (errorLower.includes('parse') || errorLower.includes('json')) return 'Parse';
    
    return 'Other';
  }

  private calculateSystemSummary(systemMetrics: SystemMetrics[]): TestSummary['systemSummary'] {
    if (systemMetrics.length === 0) {
      return {
        cpu: { averageUsage: 0, maxUsage: 0, averageLoad: 0 },
        memory: { averageUsage: 0, maxUsage: 0, peakMemory: 0 },
        disk: { averageReadRate: 0, averageWriteRate: 0, maxIoWait: 0 },
        network: { averageBandwidth: 0, totalBytesTransferred: 0, packetsLost: 0 }
      };
    }
    
    const cpuUsages = systemMetrics.map(m => m.cpu.usage);
    const memoryUsages = systemMetrics.map(m => m.memory.usage);
    const memoryUsed = systemMetrics.map(m => m.memory.used);
    const loadAverages = systemMetrics.map(m => m.cpu.loadAverage[0] || 0);
    const diskReads = systemMetrics.map(m => m.disk.readRate);
    const diskWrites = systemMetrics.map(m => m.disk.writeRate);
    const ioWaits = systemMetrics.map(m => m.disk.ioWait);
    const networkIn = systemMetrics.map(m => m.network.bytesIn);
    const networkOut = systemMetrics.map(m => m.network.bytesOut);
    const networkErrors = systemMetrics.reduce((sum, m) => sum + m.network.errors, 0);
    const networkDropped = systemMetrics.reduce((sum, m) => sum + m.network.dropped, 0);
    
    const totalBytesTransferred = systemMetrics.reduce(
      (sum, m) => sum + m.network.bytesIn + m.network.bytesOut, 
      0
    );
    
    return {
      cpu: {
        averageUsage: this.calculateAverage(cpuUsages),
        maxUsage: Math.max(...cpuUsages),
        averageLoad: this.calculateAverage(loadAverages)
      },
      memory: {
        averageUsage: this.calculateAverage(memoryUsages),
        maxUsage: Math.max(...memoryUsages),
        peakMemory: Math.max(...memoryUsed)
      },
      disk: {
        averageReadRate: this.calculateAverage(diskReads),
        averageWriteRate: this.calculateAverage(diskWrites),
        maxIoWait: Math.max(...ioWaits)
      },
      network: {
        averageBandwidth: this.calculateAverage([...networkIn, ...networkOut]),
        totalBytesTransferred,
        packetsLost: networkErrors + networkDropped
      }
    };
  }

  // Real-time metrics for dashboard
  getRealtimeStats(): {
    currentRps: number;
    currentErrorRate: number;
    currentResponseTime: number;
    totalRequests: number;
  } {
    const now = Date.now();
    const lastMinute = this.requestHistory.filter(r => now - r.timestamp <= 60000);
    
    if (lastMinute.length === 0) {
      return { currentRps: 0, currentErrorRate: 0, currentResponseTime: 0, totalRequests: 0 };
    }
    
    const successfulInLastMinute = lastMinute.filter(r => r.success);
    const responseTimes = successfulInLastMinute.map(r => r.responseTime);
    
    return {
      currentRps: lastMinute.length / 60, // requests per second in last minute
      currentErrorRate: ((lastMinute.length - successfulInLastMinute.length) / lastMinute.length) * 100,
      currentResponseTime: this.calculateAverage(responseTimes),
      totalRequests: this.requestHistory.length
    };
  }
}