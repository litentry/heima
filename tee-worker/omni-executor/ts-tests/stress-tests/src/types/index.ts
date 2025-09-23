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
  dashboardPort: number; // Default: 3000
}

export interface RequestResult {
  timestamp: number;
  requestId?: string;
  method?: string;
  url?: string;
  endpoint?: string;
  statusCode?: number;
  responseTime: number; // milliseconds
  success: boolean;
  error?: string;
  requestSize: number; // bytes
  responseSize: number; // bytes
  qps?: number;
  requestData?: any;
  responseData?: any;
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

// QPS step result for stress testing
export interface QPSStepResult {
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

// Test session result for comprehensive testing
export interface TestSessionResult {
  sessionId: string;
  startTime: number;
  endTime: number;
  config: any;
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

// Detailed session result for API responses
export interface DetailedSessionResult {
  sessionId: string;
  startTime: number;
  endTime: number;
  config: any;
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

// Session summary for dashboard listing
export interface SessionSummary {
  sessionId: string;
  startTime: number;
  endTime: number;
  duration: number;
  status: string;
  config: any;
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