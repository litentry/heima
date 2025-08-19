#!/usr/bin/env node

import { program } from 'commander';
import chalk from 'chalk';
import { table } from 'table';
import { DistributedResultCollector } from '../distributed/result-collector';
import { PerformanceThresholds } from '../types/index';

interface CapacityAnalysis {
  maxSustainableRPS: number;
  maxConcurrentUsers: number;
  responseTimeAt50RPS: number;
  responseTimeAt100RPS: number;
  responseTimeAt200RPS: number;
  errorThresholdRPS: number;
  systemBottlenecks: string[];
  recommendations: string[];
  riskAssessment: {
    level: 'low' | 'medium' | 'high';
    factors: string[];
  };
}

const PRODUCTION_THRESHOLDS: PerformanceThresholds = {
  responseTime: {
    target: 300,      // 300ms target for production
    acceptable: 500,  // 500ms acceptable
    critical: 1000,   // 1s critical
  },
  throughput: {
    target: 200,      // 200 RPS target
    minimum: 50,      // 50 RPS minimum
  },
  errorRate: {
    warning: 0.5,     // 0.5% warning
    critical: 2,      // 2% critical
  },
  systemLoad: {
    cpu: 70,          // 70% CPU warning
    memory: 80,       // 80% memory warning
    disk: 85,         // 85% disk warning
  },
};

class PerformanceAnalyzer {
  
  /**
   * Analyze system capacity from test results
   */
  static analyzeCapacity(resultDir: string): CapacityAnalysis {
    const machineResults = DistributedResultCollector.loadMachineResults(resultDir);
    
    if (machineResults.length === 0) {
      throw new Error('No test results found for analysis');
    }
    
    const aggregatedData = DistributedResultCollector.aggregateResults(machineResults);
    if (!aggregatedData) {
      throw new Error('Failed to aggregate test data');
    }
    
    const { aggregatedSummary, rawResults } = aggregatedData;
    
    // Calculate capacity metrics
    const maxSustainableRPS = this.calculateMaxSustainableRPS(aggregatedSummary, rawResults);
    const maxConcurrentUsers = this.calculateMaxConcurrentUsers(machineResults);
    const responseTimeMetrics = this.calculateResponseTimeAtLoad(rawResults);
    const errorThresholdRPS = this.findErrorThresholdRPS(rawResults);
    const bottlenecks = this.identifyBottlenecks(aggregatedSummary, rawResults);
    const recommendations = this.generateRecommendations(aggregatedSummary, bottlenecks);
    const riskAssessment = this.assessRisk(aggregatedSummary, bottlenecks);
    
    return {
      maxSustainableRPS,
      maxConcurrentUsers,
      responseTimeAt50RPS: responseTimeMetrics.at50RPS,
      responseTimeAt100RPS: responseTimeMetrics.at100RPS,
      responseTimeAt200RPS: responseTimeMetrics.at200RPS,
      errorThresholdRPS,
      systemBottlenecks: bottlenecks,
      recommendations,
      riskAssessment,
    };
  }
  
  private static calculateMaxSustainableRPS(summary: any, rawResults: any[]): number {
    // Calculate based on 95th percentile response time staying under target
    const target = PRODUCTION_THRESHOLDS.responseTime.target;
    const currentRPS = summary.totalThroughputRPS;
    const current95th = summary.percentiles.p95;
    
    if (current95th <= target) {
      // Can handle more load, estimate based on linear scaling
      return Math.floor(currentRPS * (target / current95th) * 0.8); // 80% safety margin
    } else {
      // Already over target, reduce estimate
      return Math.floor(currentRPS * (target / current95th) * 0.7); // 70% safety margin
    }
  }
  
  private static calculateMaxConcurrentUsers(machineResults: any[]): number {
    // Find the maximum concurrency tested where error rate was acceptable
    let maxSafeConcurrency = 0;
    
    for (const machine of machineResults) {
      const errorRate = (machine.summary.failedRequests / machine.summary.totalRequests) * 100;
      if (errorRate <= PRODUCTION_THRESHOLDS.errorRate.warning) {
        maxSafeConcurrency = Math.max(maxSafeConcurrency, machine.testConfig.concurrency);
      }
    }
    
    return maxSafeConcurrency;
  }
  
  private static calculateResponseTimeAtLoad(rawResults: any[]): any {
    // Simulate response times at different RPS levels
    // This is a simplified model - in practice you'd run actual tests
    const currentAvg = rawResults.reduce((sum, r) => sum + r.responseTime, 0) / rawResults.length;
    
    return {
      at50RPS: currentAvg * 0.8,   // Assume lighter load = better response time
      at100RPS: currentAvg,        // Current baseline
      at200RPS: currentAvg * 1.5,  // Assume heavier load = worse response time
    };
  }
  
  private static findErrorThresholdRPS(rawResults: any[]): number {
    // Find the RPS level where errors start to increase significantly
    // This is a simplified calculation
    const failedResults = rawResults.filter(r => !r.success);
    const totalDuration = Math.max(...rawResults.map(r => r.timestamp)) - Math.min(...rawResults.map(r => r.timestamp));
    const totalRPS = rawResults.length / (totalDuration / 1000);
    
    if (failedResults.length === 0) {
      return totalRPS * 2; // No errors seen, estimate 2x current load
    } else {
      return totalRPS * 0.9; // Errors present, be conservative
    }
  }
  
  private static identifyBottlenecks(summary: any, rawResults: any[]): string[] {
    const bottlenecks: string[] = [];
    
    // Response time analysis
    if (summary.percentiles.p95 > PRODUCTION_THRESHOLDS.responseTime.acceptable) {
      bottlenecks.push('High response time latency (95th percentile)');
    }
    
    if (summary.percentiles.p99 > PRODUCTION_THRESHOLDS.responseTime.critical) {
      bottlenecks.push('Critical response time spikes (99th percentile)');
    }
    
    // Error rate analysis
    if (summary.errorRate > PRODUCTION_THRESHOLDS.errorRate.critical) {
      bottlenecks.push('High error rate indicates system overload');
    }
    
    // Throughput analysis
    if (summary.totalThroughputRPS < PRODUCTION_THRESHOLDS.throughput.minimum) {
      bottlenecks.push('Low throughput indicates processing bottleneck');
    }
    
    // Response time distribution
    const variance = this.calculateResponseTimeVariance(rawResults);
    if (variance > summary.averageResponseTime * 0.5) {
      bottlenecks.push('High response time variance indicates inconsistent performance');
    }
    
    return bottlenecks;
  }
  
  private static calculateResponseTimeVariance(rawResults: any[]): number {
    const times = rawResults.map(r => r.responseTime);
    const mean = times.reduce((a, b) => a + b, 0) / times.length;
    const variance = times.reduce((sum, time) => sum + Math.pow(time - mean, 2), 0) / times.length;
    return Math.sqrt(variance);
  }
  
  private static generateRecommendations(summary: any, bottlenecks: string[]): string[] {
    const recommendations: string[] = [];
    
    // Performance recommendations
    if (summary.averageResponseTime > PRODUCTION_THRESHOLDS.responseTime.target) {
      recommendations.push('Optimize database queries and reduce unnecessary computations');
      recommendations.push('Consider implementing caching for frequently accessed data');
      recommendations.push('Review and optimize critical code paths in the login flow');
    }
    
    if (summary.totalThroughputRPS < PRODUCTION_THRESHOLDS.throughput.target) {
      recommendations.push('Scale horizontally by adding more server instances');
      recommendations.push('Optimize connection pooling and database connections');
      recommendations.push('Consider implementing async processing for non-critical operations');
    }
    
    if (summary.errorRate > PRODUCTION_THRESHOLDS.errorRate.warning) {
      recommendations.push('Implement circuit breakers to handle downstream failures');
      recommendations.push('Add retry logic with exponential backoff');
      recommendations.push('Improve error handling and monitoring');
    }
    
    // Infrastructure recommendations
    recommendations.push('Monitor CPU and memory usage during peak loads');
    recommendations.push('Set up auto-scaling policies based on response time and error rate');
    recommendations.push('Implement comprehensive logging and monitoring');
    recommendations.push('Plan for 20-30% capacity headroom above normal traffic');
    
    return recommendations;
  }
  
  private static assessRisk(summary: any, bottlenecks: string[]): any {
    let riskLevel: 'low' | 'medium' | 'high' = 'low';
    const riskFactors: string[] = [];
    
    // High risk factors
    if (summary.errorRate > PRODUCTION_THRESHOLDS.errorRate.critical) {
      riskLevel = 'high';
      riskFactors.push('Critical error rate exceeds acceptable limits');
    }
    
    if (summary.percentiles.p99 > PRODUCTION_THRESHOLDS.responseTime.critical) {
      riskLevel = 'high';
      riskFactors.push('Response time spikes may cause user experience issues');
    }
    
    // Medium risk factors
    if (summary.averageResponseTime > PRODUCTION_THRESHOLDS.responseTime.acceptable) {
      if (riskLevel === 'low') riskLevel = 'medium';
      riskFactors.push('Average response time above acceptable threshold');
    }
    
    if (summary.errorRate > PRODUCTION_THRESHOLDS.errorRate.warning) {
      if (riskLevel === 'low') riskLevel = 'medium';
      riskFactors.push('Error rate approaching warning levels');
    }
    
    if (bottlenecks.length > 2) {
      if (riskLevel === 'low') riskLevel = 'medium';
      riskFactors.push('Multiple performance bottlenecks identified');
    }
    
    return {
      level: riskLevel,
      factors: riskFactors.length > 0 ? riskFactors : ['System performing within acceptable parameters'],
    };
  }
}

/**
 * CLI Commands
 */

async function analyzeCapacity(options: any): Promise<void> {
  try {
    console.log(chalk.blue.bold('🔍 Analyzing System Capacity...\\n'));
    
    const resultDir = options.directory || './stress-test-results';
    const analysis = PerformanceAnalyzer.analyzeCapacity(resultDir);
    
    // Display capacity summary
    console.log(chalk.green.bold('📊 Capacity Analysis Summary'));
    console.log('='.repeat(60));
    
    const capacityData = [
      ['Metric', 'Value', 'Assessment'],
      ['Max Sustainable RPS', analysis.maxSustainableRPS.toString(), getCapacityAssessment(analysis.maxSustainableRPS, 100)],
      ['Max Concurrent Users', analysis.maxConcurrentUsers.toString(), getCapacityAssessment(analysis.maxConcurrentUsers, 50)],
      ['Response Time @ 50 RPS', `${analysis.responseTimeAt50RPS.toFixed(0)}ms`, getResponseTimeAssessment(analysis.responseTimeAt50RPS)],
      ['Response Time @ 100 RPS', `${analysis.responseTimeAt100RPS.toFixed(0)}ms`, getResponseTimeAssessment(analysis.responseTimeAt100RPS)],
      ['Response Time @ 200 RPS', `${analysis.responseTimeAt200RPS.toFixed(0)}ms`, getResponseTimeAssessment(analysis.responseTimeAt200RPS)],
      ['Error Threshold RPS', analysis.errorThresholdRPS.toString(), getCapacityAssessment(analysis.errorThresholdRPS, 75)],
    ];
    
    console.log(table(capacityData));
    
    // Risk Assessment
    console.log(chalk.blue.bold('⚠️  Risk Assessment'));
    console.log('='.repeat(60));
    
    const riskColor = analysis.riskAssessment.level === 'high' ? chalk.red : 
                     analysis.riskAssessment.level === 'medium' ? chalk.yellow : chalk.green;
    
    console.log(`Risk Level: ${riskColor.bold(analysis.riskAssessment.level.toUpperCase())}`);
    console.log('\\nRisk Factors:');
    analysis.riskAssessment.factors.forEach(factor => {
      console.log(`  • ${factor}`);
    });
    
    // Bottlenecks
    if (analysis.systemBottlenecks.length > 0) {
      console.log('\\n' + chalk.yellow.bold('🚧 Identified Bottlenecks'));
      console.log('='.repeat(60));
      analysis.systemBottlenecks.forEach(bottleneck => {
        console.log(`  ❌ ${bottleneck}`);
      });
    }
    
    // Recommendations
    console.log('\\n' + chalk.green.bold('💡 Recommendations'));
    console.log('='.repeat(60));
    analysis.recommendations.forEach((rec, index) => {
      console.log(`  ${index + 1}. ${rec}`);
    });
    
    // Capacity Planning
    console.log('\\n' + chalk.blue.bold('📈 Capacity Planning Guidelines'));
    console.log('='.repeat(60));
    console.log(`🎯 Production Target: ${analysis.maxSustainableRPS} RPS with ${analysis.maxConcurrentUsers} concurrent users`);
    console.log(`📊 Safety Margin: Plan for 70-80% of maximum capacity during normal operations`);
    console.log(`⚡ Response Time SLA: Keep 95% of requests under ${PRODUCTION_THRESHOLDS.responseTime.target}ms`);
    console.log(`🚨 Alert Thresholds: Error rate > ${PRODUCTION_THRESHOLDS.errorRate.warning}%, Response time > ${PRODUCTION_THRESHOLDS.responseTime.acceptable}ms`);
    
  } catch (error) {
    console.error(chalk.red('❌ Analysis failed:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

function getCapacityAssessment(value: number, target: number): string {
  if (value >= target * 1.5) return chalk.green('Excellent');
  if (value >= target) return chalk.green('Good');
  if (value >= target * 0.7) return chalk.yellow('Acceptable');
  return chalk.red('Poor');
}

function getResponseTimeAssessment(responseTime: number): string {
  if (responseTime <= PRODUCTION_THRESHOLDS.responseTime.target) return chalk.green('Excellent');
  if (responseTime <= PRODUCTION_THRESHOLDS.responseTime.acceptable) return chalk.yellow('Acceptable');
  return chalk.red('Poor');
}

async function generateReport(options: any): Promise<void> {
  try {
    const resultDir = options.directory || './stress-test-results';
    const outputPath = options.output;
    
    console.log(chalk.blue('📋 Generating comprehensive performance report...'));
    
    // Generate standard aggregated report
    const reportPath = DistributedResultCollector.saveAggregatedReport(resultDir, outputPath);
    
    // Add capacity analysis
    const analysis = PerformanceAnalyzer.analyzeCapacity(resultDir);
    
    console.log(chalk.green('✅ Performance report generated'));
    console.log(`📄 Report location: ${chalk.cyan(reportPath)}`);
    console.log('\\n' + chalk.blue.bold('Quick Summary:'));
    console.log(`• Max Sustainable RPS: ${analysis.maxSustainableRPS}`);
    console.log(`• Risk Level: ${analysis.riskAssessment.level.toUpperCase()}`);
    console.log(`• Bottlenecks Found: ${analysis.systemBottlenecks.length}`);
    
  } catch (error) {
    console.error(chalk.red('❌ Report generation failed:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// CLI setup
program
  .name('performance-analyzer')
  .description('🔍 Advanced performance analysis tool for Omni Executor stress tests')
  .version('1.0.0');

program
  .command('capacity')
  .description('🎯 Analyze system capacity and bottlenecks')
  .option('-d, --directory <path>', 'Test results directory', './stress-test-results')
  .action(analyzeCapacity);

program
  .command('report')
  .description('📋 Generate comprehensive performance report')
  .option('-d, --directory <path>', 'Test results directory', './stress-test-results')
  .option('-o, --output <path>', 'Output report path')
  .action(generateReport);

program
  .command('example')
  .description('📖 Show analysis examples')
  .action(() => {
    console.log(chalk.blue.bold('🌟 Performance Analysis Examples\\n'));
    
    console.log(chalk.green('# Analyze capacity from test results'));
    console.log('npm run analyze capacity\\n');
    
    console.log(chalk.green('# Analyze specific results directory'));
    console.log('npm run analyze capacity -d ./my-test-results\\n');
    
    console.log(chalk.green('# Generate comprehensive report'));
    console.log('npm run analyze report -o ./performance-report.md\\n');
    
    console.log(chalk.blue('💡 Analysis includes:'));
    console.log('- Maximum sustainable RPS calculations');
    console.log('- Bottleneck identification');
    console.log('- Risk assessment and recommendations');
    console.log('- Capacity planning guidelines');
  });

// Parse CLI arguments
if (import.meta.url === `file://${process.argv[1]}`) {
  program.parse();
}