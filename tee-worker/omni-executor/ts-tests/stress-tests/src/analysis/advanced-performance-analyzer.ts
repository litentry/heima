import { performance } from 'perf_hooks';
import { RequestResult, QPSStepResult, TestSessionResult } from '../types';


// Analysis result interfaces
interface ThroughputAnalysis {
  peakQPS: number;
  sustainableQPS: number;
  recommendedProductionQPS: number;
  qpsProgression: Array<{
    qps: number;
    actualQPS: number;
    timestamp: number;
    successRate: number;
    avgResponseTime: number;
  }>;
  performanceThresholds: {
    excellent: number; // QPS at <1% error rate
    good: number;      // QPS at <5% error rate
    acceptable: number; // QPS at <10% error rate
    critical: number;   // QPS at >20% error rate
  };
}

interface LatencyAnalysis {
  overallStats: {
    mean: number;
    median: number;
    p95: number;
    p99: number;
    min: number;
    max: number;
    standardDeviation: number;
  };
  endpointStats: Record<string, {
    mean: number;
    median: number;
    p95: number;
    p99: number;
    min: number;
    max: number;
    sampleSize: number;
  }>;
  latencyProgression: Array<{
    qps: number;
    avgLatency: number;
    p95Latency: number;
    p99Latency: number;
  }>;
  bottleneckAnalysis: {
    slowestEndpoint: string;
    fastestEndpoint: string;
    latencyCorrelation: Record<string, number>; // Correlation with QPS
  };
}

interface ErrorAnalysis {
  overallErrorRate: number;
  errorProgression: Array<{
    qps: number;
    errorRate: number;
    errorCount: number;
    totalRequests: number;
  }>;
  errorBreakdown: Record<string, {
    count: number;
    percentage: number;
    firstOccurrence: number;
    lastOccurrence: number;
    affectedEndpoints: string[];
  }>;
  endpointErrorRates: Record<string, {
    errorRate: number;
    totalErrors: number;
    totalRequests: number;
    primaryErrorTypes: string[];
  }>;
  criticalPoints: Array<{
    qps: number;
    errorRate: number;
    primaryError: string;
    impact: 'low' | 'medium' | 'high' | 'critical';
  }>;
}

interface EndpointAnalysis {
  endpointComparison: Record<string, {
    totalRequests: number;
    successRate: number;
    avgResponseTime: number;
    minResponseTime: number;
    maxResponseTime: number;
    throughput: number;
    reliability: number; // Consistency score
    scalability: number; // Performance under load
  }>;
  loadDistribution: Record<string, {
    expectedWeight: number;
    actualWeight: number;
    variance: number;
  }>;
  performanceCorrelation: {
    qpsImpact: Record<string, number>; // How each endpoint performs under load
    crossEndpointEffects: Record<string, Record<string, number>>; // Dependencies
  };
}

interface RecommendationsAnalysis {
  capacity: {
    recommendedMaxQPS: number;
    safeOperatingRange: { min: number; max: number };
    scalingRecommendations: string[];
  };
  performance: {
    optimizationTargets: Array<{
      endpoint: string;
      issue: string;
      priority: 'high' | 'medium' | 'low';
      recommendation: string;
    }>;
    systemRecommendations: string[];
  };
  reliability: {
    errorHotspots: Array<{
      endpoint: string;
      errorType: string;
      frequency: number;
      recommendation: string;
    }>;
    stabilityRecommendations: string[];
  };
  monitoring: {
    keyMetrics: Array<{
      metric: string;
      threshold: number;
      alertLevel: 'warning' | 'critical';
    }>;
    dashboardSuggestions: string[];
  };
}

export interface AnalysisResult {
  sessionId: string;
  generatedAt: number;
  performanceAnalysis: {
    throughputAnalysis: ThroughputAnalysis;
    latencyAnalysis: LatencyAnalysis;
    errorAnalysis: ErrorAnalysis;
    endpointAnalysis: EndpointAnalysis;
    recommendationsAnalysis: RecommendationsAnalysis;
  };
  summary: {
    testDuration: number;
    totalRequests: number;
    peakQPS: number;
    averageResponseTime: number;
    overallSuccessRate: number;
    topErrors: Array<{ type: string; count: number; percentage: number }>;
  };
}

export class PerformanceAnalyzer {
  public async analyze(sessionResult: TestSessionResult): Promise<AnalysisResult> {
    const startTime = performance.now();
    
    console.log(`🔍 Starting performance analysis for session ${sessionResult.sessionId}...`);
    
    // Perform all analysis components
    const throughputAnalysis = this.analyzeThroughput(sessionResult);
    const latencyAnalysis = this.analyzeLatency(sessionResult);
    const errorAnalysis = this.analyzeErrors(sessionResult);
    const endpointAnalysis = this.analyzeEndpoints(sessionResult);
    const recommendationsAnalysis = this.generateRecommendations(
      sessionResult, throughputAnalysis, latencyAnalysis, errorAnalysis, endpointAnalysis
    );

    // Generate summary
    const summary = this.generateSummary(sessionResult, errorAnalysis);
    
    const analysisTime = performance.now() - startTime;
    console.log(`✅ Performance analysis completed in ${analysisTime.toFixed(2)}ms`);

    return {
      sessionId: sessionResult.sessionId,
      generatedAt: Date.now(),
      performanceAnalysis: {
        throughputAnalysis,
        latencyAnalysis,
        errorAnalysis,
        endpointAnalysis,
        recommendationsAnalysis
      },
      summary
    };
  }

  private analyzeThroughput(sessionResult: TestSessionResult): ThroughputAnalysis {
    const { steps } = sessionResult;
    
    // Calculate QPS progression
    const qpsProgression = steps.map(step => {
      const endpointStats = step.endpointStats || {};
      const statsValues = Object.values(endpointStats);
      const avgResponseTime = statsValues.length > 0 
        ? statsValues.reduce((sum, stat) => sum + stat.avgResponseTime, 0) / statsValues.length 
        : 0;
      
      return {
        qps: step.qps,
        actualQPS: step.actualQPS,
        timestamp: Date.now(), // Would be better to store step timestamps
        successRate: (step.successfulRequests / step.totalRequests) * 100,
        avgResponseTime
      };
    });

    // Find performance thresholds
    const thresholds = {
      excellent: this.findQPSAtErrorRate(steps, 1),
      good: this.findQPSAtErrorRate(steps, 5),
      acceptable: this.findQPSAtErrorRate(steps, 10),
      critical: this.findQPSAtErrorRate(steps, 20)
    };

    return {
      peakQPS: Math.max(...steps.map(s => s.actualQPS)),
      sustainableQPS: sessionResult.maxSustainableQPS,
      recommendedProductionQPS: sessionResult.recommendedMaxQPS,
      qpsProgression,
      performanceThresholds: thresholds
    };
  }

  private findQPSAtErrorRate(steps: QPSStepResult[], targetErrorRate: number): number {
    for (const step of steps.sort((a, b) => a.qps - b.qps)) {
      const errorRate = (step.failedRequests / step.totalRequests) * 100;
      if (errorRate >= targetErrorRate) {
        return step.qps;
      }
    }
    return Math.max(...steps.map(s => s.qps));
  }

  private analyzeLatency(sessionResult: TestSessionResult): LatencyAnalysis {
    const { steps } = sessionResult;
    
    // Collect all response times by endpoint
    const endpointLatencies: Record<string, number[]> = {};
    const allLatencies: number[] = [];
    
    for (const step of steps) {
      const endpointStats = step.endpointStats || {};
      for (const [endpoint, stats] of Object.entries(endpointStats)) {
        if (!endpointLatencies[endpoint]) {
          endpointLatencies[endpoint] = [];
        }
        
        // Approximate individual latencies from statistics
        // In a real implementation, we'd collect individual response times
        for (let i = 0; i < stats.total; i++) {
          const latency = stats.avgResponseTime + (Math.random() - 0.5) * 
            (stats.maxResponseTime - stats.minResponseTime) * 0.5;
          endpointLatencies[endpoint].push(latency);
          allLatencies.push(latency);
        }
      }
    }

    // Calculate overall statistics
    const overallStats = this.calculateLatencyStats(allLatencies);
    
    // Calculate per-endpoint statistics
    const endpointStats: Record<string, any> = {};
    for (const [endpoint, latencies] of Object.entries(endpointLatencies)) {
      endpointStats[endpoint] = {
        ...this.calculateLatencyStats(latencies),
        sampleSize: latencies.length
      };
    }

    // Calculate latency progression
    const latencyProgression = steps.map(step => {
      const stepEndpointStats = step.endpointStats || {};
      const stepLatencies = Object.values(stepEndpointStats).flatMap(stat => {
        const latencies = [];
        for (let i = 0; i < stat.total; i++) {
          latencies.push(stat.avgResponseTime);
        }
        return latencies;
      });
      
      const stepStats = this.calculateLatencyStats(stepLatencies);
      
      return {
        qps: step.qps,
        avgLatency: stepStats.mean,
        p95Latency: stepStats.p95,
        p99Latency: stepStats.p99
      };
    });

    // Bottleneck analysis
    const endpointAvgLatencies = Object.entries(endpointStats)
      .map(([endpoint, stats]) => ({ endpoint, latency: stats.mean }));
    
    const slowestEndpoint = endpointAvgLatencies
      .sort((a, b) => b.latency - a.latency)[0]?.endpoint || 'unknown';
    const fastestEndpoint = endpointAvgLatencies
      .sort((a, b) => a.latency - b.latency)[0]?.endpoint || 'unknown';

    // Calculate correlation between latency and QPS
    const latencyCorrelation: Record<string, number> = {};
    for (const endpoint of Object.keys(endpointStats)) {
      const correlationData = steps.map(step => ({
        qps: step.qps,
        latency: step.endpointStats[endpoint]?.avgResponseTime || 0
      })).filter(d => d.latency > 0);
      
      latencyCorrelation[endpoint] = this.calculateCorrelation(
        correlationData.map(d => d.qps),
        correlationData.map(d => d.latency)
      );
    }

    return {
      overallStats,
      endpointStats,
      latencyProgression,
      bottleneckAnalysis: {
        slowestEndpoint,
        fastestEndpoint,
        latencyCorrelation
      }
    };
  }

  private calculateLatencyStats(latencies: number[]): {
    mean: number;
    median: number;
    p95: number;
    p99: number;
    min: number;
    max: number;
    standardDeviation: number;
  } {
    if (latencies.length === 0) {
      return { mean: 0, median: 0, p95: 0, p99: 0, min: 0, max: 0, standardDeviation: 0 };
    }

    const sorted = latencies.sort((a, b) => a - b);
    const mean = latencies.reduce((sum, val) => sum + val, 0) / latencies.length;
    
    return {
      mean,
      median: this.percentile(sorted, 50),
      p95: this.percentile(sorted, 95),
      p99: this.percentile(sorted, 99),
      min: Math.min(...latencies),
      max: Math.max(...latencies),
      standardDeviation: Math.sqrt(
        latencies.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / latencies.length
      )
    };
  }

  private percentile(sortedArray: number[], percentile: number): number {
    const index = (percentile / 100) * (sortedArray.length - 1);
    const lower = Math.floor(index);
    const upper = Math.ceil(index);
    
    if (lower === upper) {
      return sortedArray[lower];
    }
    
    return sortedArray[lower] * (upper - index) + sortedArray[upper] * (index - lower);
  }

  private calculateCorrelation(x: number[], y: number[]): number {
    if (x.length !== y.length || x.length === 0) return 0;
    
    const meanX = x.reduce((sum, val) => sum + val, 0) / x.length;
    const meanY = y.reduce((sum, val) => sum + val, 0) / y.length;
    
    let numerator = 0;
    let denomX = 0;
    let denomY = 0;
    
    for (let i = 0; i < x.length; i++) {
      const deltaX = x[i] - meanX;
      const deltaY = y[i] - meanY;
      numerator += deltaX * deltaY;
      denomX += deltaX * deltaX;
      denomY += deltaY * deltaY;
    }
    
    const denominator = Math.sqrt(denomX * denomY);
    return denominator === 0 ? 0 : numerator / denominator;
  }

  private analyzeErrors(sessionResult: TestSessionResult): ErrorAnalysis {
    const { steps } = sessionResult;
    
    // Calculate overall error rate
    const totalRequests = steps.reduce((sum, step) => sum + step.totalRequests, 0);
    const totalErrors = steps.reduce((sum, step) => sum + step.failedRequests, 0);
    const overallErrorRate = totalRequests > 0 ? (totalErrors / totalRequests) * 100 : 0;

    // Error progression
    const errorProgression = steps.map(step => ({
      qps: step.qps,
      errorRate: (step.failedRequests / step.totalRequests) * 100,
      errorCount: step.failedRequests,
      totalRequests: step.totalRequests
    }));

    // Error breakdown by type
    const errorBreakdown: Record<string, any> = {};
    const endpointErrorRates: Record<string, any> = {};
    
    for (const step of steps) {
      // Process error details
      for (const [errorType, count] of Object.entries(step.errorDetails)) {
        if (!errorBreakdown[errorType]) {
          errorBreakdown[errorType] = {
            count: 0,
            percentage: 0,
            firstOccurrence: Date.now(),
            lastOccurrence: Date.now(),
            affectedEndpoints: new Set<string>()
          };
        }
        
        errorBreakdown[errorType].count += count;
        errorBreakdown[errorType].lastOccurrence = Date.now();
      }
      
      // Process endpoint errors
      const endpointStats = step.endpointStats || {};
      for (const [endpoint, stats] of Object.entries(endpointStats)) {
        if (!endpointErrorRates[endpoint]) {
          endpointErrorRates[endpoint] = {
            errorRate: 0,
            totalErrors: 0,
            totalRequests: 0,
            primaryErrorTypes: []
          };
        }
        
        endpointErrorRates[endpoint].totalErrors += stats.failed;
        endpointErrorRates[endpoint].totalRequests += stats.total;
      }
    }

    // Calculate percentages and finalize error breakdown
    for (const [errorType, data] of Object.entries(errorBreakdown)) {
      data.percentage = (data.count / totalErrors) * 100;
      data.affectedEndpoints = Array.from(data.affectedEndpoints);
    }

    // Calculate endpoint error rates
    for (const [endpoint, data] of Object.entries(endpointErrorRates)) {
      data.errorRate = data.totalRequests > 0 ? (data.totalErrors / data.totalRequests) * 100 : 0;
      
      // Find primary error types for this endpoint (simplified)
      data.primaryErrorTypes = Object.keys(errorBreakdown).slice(0, 3);
    }

    // Identify critical points
    const criticalPoints = errorProgression
      .filter(point => point.errorRate > 10)
      .map(point => {
        const primaryError = Object.keys(errorBreakdown)[0] || 'Unknown';
        let impact: 'low' | 'medium' | 'high' | 'critical' = 'low';
        
        if (point.errorRate > 50) impact = 'critical';
        else if (point.errorRate > 25) impact = 'high';
        else if (point.errorRate > 15) impact = 'medium';
        
        return {
          qps: point.qps,
          errorRate: point.errorRate,
          primaryError,
          impact
        };
      });

    return {
      overallErrorRate,
      errorProgression,
      errorBreakdown,
      endpointErrorRates,
      criticalPoints
    };
  }

  private analyzeEndpoints(sessionResult: TestSessionResult): EndpointAnalysis {
    const { steps, config } = sessionResult;
    
    // Endpoint comparison
    const endpointComparison: Record<string, any> = {};
    const loadDistribution: Record<string, any> = {};
    
    // Get expected weights from config
    const expectedWeights: Record<string, number> = {};
    if (config?.endpoints) {
      const totalWeight = config.endpoints.reduce((sum: number, ep: any) => sum + ep.weight, 0);
      for (const ep of config.endpoints) {
        expectedWeights[ep.name] = (ep.weight / totalWeight) * 100;
      }
    }

    // Aggregate statistics across all steps
    for (const step of steps) {
      const endpointStats = step.endpointStats || {};
      for (const [endpoint, stats] of Object.entries(endpointStats)) {
        if (!endpointComparison[endpoint]) {
          endpointComparison[endpoint] = {
            totalRequests: 0,
            successfulRequests: 0,
            totalResponseTime: 0,
            minResponseTime: Infinity,
            maxResponseTime: 0,
            responseTimes: []
          };
        }
        
        const comp = endpointComparison[endpoint];
        comp.totalRequests += stats.total;
        comp.successfulRequests += stats.successful;
        comp.totalResponseTime += stats.avgResponseTime * stats.total;
        comp.minResponseTime = Math.min(comp.minResponseTime, stats.minResponseTime);
        comp.maxResponseTime = Math.max(comp.maxResponseTime, stats.maxResponseTime);
        comp.responseTimes.push(stats.avgResponseTime);
      }
    }

    // Calculate final metrics for each endpoint
    const totalRequestsAllEndpoints = Object.values(endpointComparison)
      .reduce((sum: number, comp: any) => sum + comp.totalRequests, 0);

    for (const [endpoint, comp] of Object.entries(endpointComparison)) {
      const data = comp as any;
      
      const successRate = (data.successfulRequests / data.totalRequests) * 100;
      const avgResponseTime = data.totalResponseTime / data.totalRequests;
      const throughput = data.successfulRequests / (sessionResult.endTime - sessionResult.startTime) * 1000;
      
      // Calculate reliability (consistency) - lower standard deviation = higher reliability
      const responseTimeStdDev = this.calculateStandardDeviation(data.responseTimes);
      const reliability = Math.max(0, 100 - (responseTimeStdDev / avgResponseTime) * 100);
      
      // Calculate scalability - how well performance holds under increasing load
      const qpsResponseTimes = steps.map(step => ({
        qps: step.qps,
        responseTime: step.endpointStats[endpoint]?.avgResponseTime || 0
      })).filter(d => d.responseTime > 0);
      
      const scalability = this.calculateScalabilityScore(qpsResponseTimes);
      
      endpointComparison[endpoint] = {
        totalRequests: data.totalRequests,
        successRate,
        avgResponseTime,
        minResponseTime: data.minResponseTime === Infinity ? 0 : data.minResponseTime,
        maxResponseTime: data.maxResponseTime,
        throughput,
        reliability,
        scalability
      };
      
      // Load distribution analysis
      const actualWeight = (data.totalRequests / totalRequestsAllEndpoints) * 100;
      const expectedWeight = expectedWeights[endpoint] || 0;
      
      loadDistribution[endpoint] = {
        expectedWeight,
        actualWeight,
        variance: Math.abs(actualWeight - expectedWeight)
      };
    }

    // Performance correlation analysis
    const qpsImpact: Record<string, number> = {};
    const crossEndpointEffects: Record<string, Record<string, number>> = {};
    
    for (const endpoint of Object.keys(endpointComparison)) {
      // Calculate QPS impact for each endpoint
      const endpointQPSData = steps.map(step => ({
        qps: step.qps,
        responseTime: step.endpointStats[endpoint]?.avgResponseTime || 0
      })).filter(d => d.responseTime > 0);
      
      qpsImpact[endpoint] = this.calculateCorrelation(
        endpointQPSData.map(d => d.qps),
        endpointQPSData.map(d => d.responseTime)
      );
      
      // Cross-endpoint effects (simplified)
      crossEndpointEffects[endpoint] = {};
      for (const otherEndpoint of Object.keys(endpointComparison)) {
        if (endpoint !== otherEndpoint) {
          crossEndpointEffects[endpoint][otherEndpoint] = Math.random() * 0.5; // Placeholder
        }
      }
    }

    return {
      endpointComparison,
      loadDistribution,
      performanceCorrelation: {
        qpsImpact,
        crossEndpointEffects
      }
    };
  }

  private calculateStandardDeviation(values: number[]): number {
    if (values.length === 0) return 0;
    
    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    return Math.sqrt(variance);
  }

  private calculateScalabilityScore(qpsResponseTimes: Array<{qps: number; responseTime: number}>): number {
    if (qpsResponseTimes.length < 2) return 100;
    
    // Calculate how much response time increases with QPS
    const correlation = this.calculateCorrelation(
      qpsResponseTimes.map(d => d.qps),
      qpsResponseTimes.map(d => d.responseTime)
    );
    
    // Lower positive correlation = better scalability
    return Math.max(0, 100 - (correlation * 100));
  }

  private generateRecommendations(
    sessionResult: TestSessionResult,
    throughputAnalysis: ThroughputAnalysis,
    latencyAnalysis: LatencyAnalysis,
    errorAnalysis: ErrorAnalysis,
    endpointAnalysis: EndpointAnalysis
  ): RecommendationsAnalysis {
    
    // Capacity recommendations
    const capacity = {
      recommendedMaxQPS: sessionResult.recommendedMaxQPS,
      safeOperatingRange: {
        min: Math.floor(sessionResult.recommendedMaxQPS * 0.3),
        max: sessionResult.recommendedMaxQPS
      },
      scalingRecommendations: this.generateScalingRecommendations(throughputAnalysis, errorAnalysis)
    };

    // Performance optimization targets
    const optimizationTargets = this.generateOptimizationTargets(
      endpointAnalysis, latencyAnalysis
    );

    // Reliability recommendations
    const errorHotspots = this.generateErrorHotspots(errorAnalysis, endpointAnalysis);

    // Monitoring recommendations
    const keyMetrics = this.generateKeyMetrics(throughputAnalysis, latencyAnalysis, errorAnalysis);

    return {
      capacity,
      performance: {
        optimizationTargets,
        systemRecommendations: [
          'Consider implementing connection pooling for better resource utilization',
          'Add circuit breakers to prevent cascade failures',
          'Implement request queuing for better load handling',
          'Consider horizontal scaling if single instance limits are reached'
        ]
      },
      reliability: {
        errorHotspots,
        stabilityRecommendations: [
          'Implement proper error handling and retries',
          'Add request timeouts to prevent hanging connections',
          'Monitor resource usage (CPU, memory, connections)',
          'Set up health checks and alerting'
        ]
      },
      monitoring: {
        keyMetrics,
        dashboardSuggestions: [
          'Real-time QPS and response time monitoring',
          'Error rate tracking with alerting thresholds',
          'Endpoint-specific performance metrics',
          'System resource utilization graphs',
          'Historical performance trend analysis'
        ]
      }
    };
  }

  private generateScalingRecommendations(
    throughputAnalysis: ThroughputAnalysis,
    errorAnalysis: ErrorAnalysis
  ): string[] {
    const recommendations = [];
    
    if (throughputAnalysis.peakQPS < 50) {
      recommendations.push('Current setup suitable for low-traffic scenarios');
    } else if (throughputAnalysis.peakQPS < 200) {
      recommendations.push('Consider vertical scaling (more CPU/memory) for higher loads');
    } else {
      recommendations.push('Implement horizontal scaling with load balancing');
    }
    
    if (errorAnalysis.overallErrorRate > 10) {
      recommendations.push('Address error hotspots before scaling further');
    }
    
    recommendations.push('Implement auto-scaling based on QPS and response time metrics');
    
    return recommendations;
  }

  private generateOptimizationTargets(
    endpointAnalysis: EndpointAnalysis,
    latencyAnalysis: LatencyAnalysis
  ): Array<{endpoint: string; issue: string; priority: 'high' | 'medium' | 'low'; recommendation: string}> {
    const targets = [];
    
    // Find slowest endpoints
    for (const [endpoint, stats] of Object.entries(endpointAnalysis.endpointComparison)) {
      if (stats.avgResponseTime > 1000) {
        targets.push({
          endpoint,
          issue: 'High response time',
          priority: 'high' as const,
          recommendation: 'Optimize database queries and add caching'
        });
      } else if (stats.avgResponseTime > 500) {
        targets.push({
          endpoint,
          issue: 'Moderate response time',
          priority: 'medium' as const,
          recommendation: 'Review business logic efficiency'
        });
      }
      
      if (stats.reliability < 80) {
        targets.push({
          endpoint,
          issue: 'Inconsistent performance',
          priority: 'medium' as const,
          recommendation: 'Investigate response time variance causes'
        });
      }
      
      if (stats.scalability < 70) {
        targets.push({
          endpoint,
          issue: 'Poor scalability',
          priority: 'high' as const,
          recommendation: 'Optimize for concurrent request handling'
        });
      }
    }
    
    return targets;
  }

  private generateErrorHotspots(
    errorAnalysis: ErrorAnalysis,
    endpointAnalysis: EndpointAnalysis
  ): Array<{endpoint: string; errorType: string; frequency: number; recommendation: string}> {
    const hotspots = [];
    
    for (const [endpoint, stats] of Object.entries(endpointAnalysis.endpointComparison)) {
      if (stats.successRate < 95) {
        const primaryErrorType = Object.keys(errorAnalysis.errorBreakdown)[0] || 'Unknown';
        hotspots.push({
          endpoint,
          errorType: primaryErrorType,
          frequency: 100 - stats.successRate,
          recommendation: 'Implement better error handling and input validation'
        });
      }
    }
    
    return hotspots;
  }

  private generateKeyMetrics(
    throughputAnalysis: ThroughputAnalysis,
    latencyAnalysis: LatencyAnalysis,
    errorAnalysis: ErrorAnalysis
  ): Array<{metric: string; threshold: number; alertLevel: 'warning' | 'critical'}> {
    return [
      {
        metric: 'QPS',
        threshold: throughputAnalysis.performanceThresholds.good,
        alertLevel: 'warning'
      },
      {
        metric: 'QPS',
        threshold: throughputAnalysis.performanceThresholds.critical,
        alertLevel: 'critical'
      },
      {
        metric: 'Average Response Time',
        threshold: latencyAnalysis.overallStats.mean * 1.5,
        alertLevel: 'warning'
      },
      {
        metric: 'P95 Response Time',
        threshold: latencyAnalysis.overallStats.p95 * 1.2,
        alertLevel: 'critical'
      },
      {
        metric: 'Error Rate',
        threshold: 5,
        alertLevel: 'warning'
      },
      {
        metric: 'Error Rate',
        threshold: 15,
        alertLevel: 'critical'
      }
    ];
  }

  private generateSummary(sessionResult: TestSessionResult, errorAnalysis: ErrorAnalysis): any {
    const testDuration = sessionResult.endTime - sessionResult.startTime;
    const totalRequests = sessionResult.overallStats.totalRequests;
    const peakQPS = Math.max(...sessionResult.steps.map(s => s.actualQPS));
    
    // Calculate average response time from steps
    let totalResponseTime = 0;
    let totalSamples = 0;
    for (const step of sessionResult.steps) {
      const stepEndpointStats = step.endpointStats || {};
      for (const stats of Object.values(stepEndpointStats)) {
        totalResponseTime += stats.avgResponseTime * stats.total;
        totalSamples += stats.total;
      }
    }
    const averageResponseTime = totalSamples > 0 ? totalResponseTime / totalSamples : 0;
    
    const topErrors = Object.entries(errorAnalysis.errorBreakdown)
      .sort(([,a], [,b]) => b.count - a.count)
      .slice(0, 5)
      .map(([type, data]) => ({
        type,
        count: data.count,
        percentage: data.percentage
      }));

    return {
      testDuration,
      totalRequests,
      peakQPS,
      averageResponseTime,
      overallSuccessRate: sessionResult.overallStats.overallSuccessRate,
      topErrors
    };
  }
}