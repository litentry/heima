export interface StressTestConfig {
  // Test Configuration
  testName: string;
  duration: number; // seconds
  concurrency: number;
  rampUpTime: number; // seconds
  rampDownTime: number; // seconds
  
  // Target Configuration
  targetUrl: string;
  
  // Request Configuration
  requestTimeout: number; // milliseconds
  retries: number;
  
  // Reporting Configuration
  outputFormat: ('json' | 'excel' | 'console')[];
  outputPath: string;
  enableDashboard: boolean;
  dashboardPort: number;
}

export interface RequestResult {
  timestamp: number;
  requestId: string;
  method: string;
  url: string;
  statusCode: number;
  responseTime: number; // milliseconds
  success: boolean;
  error?: string;
  requestSize: number; // bytes
  responseSize: number; // bytes
}

export interface SystemMetrics {
  timestamp: number;
  cpu: {
    usage: number; // percentage
    cores: number[];
    loadAverage: number[];
    contextSwitches: number;
    interrupts: number;
  };
  memory: {
    total: number; // bytes
    used: number;
    available: number;
    usage: number; // percentage
    cached: number;
    buffers: number;
  };
  disk: {
    readRate: number; // bytes/s
    writeRate: number; // bytes/s
    ioWait: number; // percentage
    queueLength: number;
    usage: number; // percentage
  };
  network: {
    bytesIn: number; // bytes/s
    bytesOut: number; // bytes/s
    packetsIn: number;
    packetsOut: number;
    errors: number;
    dropped: number;
    latency: number; // milliseconds
  };
}

export interface TestSession {
  sessionId: string;
  config: StressTestConfig;
  startTime: number;
  endTime?: number;
  status: 'preparing' | 'running' | 'completed' | 'failed' | 'cancelled';
  results: RequestResult[];
  systemMetrics: SystemMetrics[];
  summary?: TestSummary;
}

export interface TestSummary {
  // Response Time Metrics
  responseTime: {
    average: number;
    min: number;
    max: number;
    p50: number;
    p90: number;
    p95: number;
    p99: number;
  };
  
  // Throughput Metrics
  throughput: {
    rps: number; // requests per second
    tps: number; // transactions per second
    dataTransferRate: number; // bytes per second
    peakConcurrency: number;
  };
  
  // Error Rate Metrics
  errorRate: {
    total: number; // percentage
    http4xx: number;
    http5xx: number;
    timeout: number;
    connection: number;
    errorsByType?: Record<string, number>;
  };
  
  // System Resource Summary
  systemSummary?: {
    cpu: {
      averageUsage: number;
      maxUsage: number;
      averageLoad: number;
    };
    memory: {
      averageUsage: number;
      maxUsage: number;
      peakMemory: number;
    };
    disk: {
      averageReadRate: number;
      averageWriteRate: number;
      maxIoWait: number;
    };
    network: {
      averageBandwidth: number;
      totalBytesTransferred: number;
      packetsLost: number;
    };
  };
  
  // Additional Stats
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  testDuration: number;
}

// Distributed Testing Types
export interface DistributedTestConfig {
  testName: string;
  targetUrl: string;
  duration: number;
  concurrency: number;
  rampUpTime?: number;
  testType: 'omni_userLogin' | 'omni_getShieldingKey' | 'omni_getNextIntentId' | 'omni_addWallet' | 'mixed' | 'weighted_mixed';
}

// QPS Ramp Testing Types
export interface QPSRampConfig {
  testName: string;
  targetUrl: string;
  startQPS: number;
  maxQPS: number;
  stepSize: number;
  stepDuration: number;
  testType: 'omni_userLogin' | 'omni_getShieldingKey' | 'omni_addWallet' | 'mixed' | 'weighted_mixed';
  outputPath?: string;
}

export interface StressTestResult {
  requestId: string;
  timestamp: number;
  success: boolean;
  responseTime: number;
  responseSize: number;
  data?: any;
  error?: string;
}

// System Performance Thresholds
export interface PerformanceThresholds {
  responseTime: {
    target: number; // ms
    acceptable: number; // ms
    critical: number; // ms
  };
  throughput: {
    target: number; // RPS
    minimum: number; // RPS
  };
  errorRate: {
    warning: number; // percentage
    critical: number; // percentage
  };
  systemLoad: {
    cpu: number; // percentage
    memory: number; // percentage
    disk: number; // percentage
  };
}