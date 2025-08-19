import { writeFileSync, readFileSync, existsSync, mkdirSync } from 'fs';
import { join } from 'path';
import { hostname } from 'os';
import { StressTestResult, DistributedTestConfig } from '../types/index';

export interface MachineTestResult {
  machineId: string;
  hostname: string;
  testConfig: DistributedTestConfig;
  startTime: number;
  endTime: number;
  results: StressTestResult[];
  systemInfo: {
    platform: string;
    arch: string;
    cpus: number;
    memory: number;
    nodeVersion: string;
  };
  summary: {
    totalRequests: number;
    successfulRequests: number;
    failedRequests: number;
    averageResponseTime: number;
    minResponseTime: number;
    maxResponseTime: number;
    throughputRPS: number;
    errorRate: number;
  };
}

export class DistributedResultCollector {
  private outputDir: string;
  private machineId: string;
  
  constructor(outputDir: string = './stress-test-results') {
    this.outputDir = outputDir;
    this.machineId = this.generateMachineId();
    
    // Ensure output directory exists
    if (!existsSync(this.outputDir)) {
      mkdirSync(this.outputDir, { recursive: true });
    }
  }
  
  private generateMachineId(): string {
    const hostName = hostname();
    const timestamp = Date.now();
    const random = Math.random().toString(36).substr(2, 6);
    return `${hostName}-${timestamp}-${random}`;
  }
  
  private getSystemInfo() {
    return {
      platform: process.platform,
      arch: process.arch,
      cpus: require('os').cpus().length,
      memory: Math.round(require('os').totalmem() / 1024 / 1024 / 1024), // GB
      nodeVersion: process.version,
    };
  }
  
  private calculateSummary(results: StressTestResult[]) {
    const totalRequests = results.length;
    const successfulRequests = results.filter(r => r.success).length;
    const failedRequests = totalRequests - successfulRequests;
    
    const responseTimes = results.map(r => r.responseTime);
    const averageResponseTime = responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length || 0;
    const minResponseTime = Math.min(...responseTimes) || 0;
    const maxResponseTime = Math.max(...responseTimes) || 0;
    
    const testDuration = (Math.max(...results.map(r => r.timestamp)) - Math.min(...results.map(r => r.timestamp))) / 1000;
    const throughputRPS = totalRequests / testDuration || 0;
    const errorRate = (failedRequests / totalRequests) * 100 || 0;
    
    return {
      totalRequests,
      successfulRequests,
      failedRequests,
      averageResponseTime,
      minResponseTime,
      maxResponseTime,
      throughputRPS,
      errorRate,
    };
  }
  
  saveResults(
    testConfig: DistributedTestConfig,
    results: StressTestResult[],
    startTime: number,
    endTime: number
  ): string {
    const machineResult: MachineTestResult = {
      machineId: this.machineId,
      hostname: hostname(),
      testConfig,
      startTime,
      endTime,
      results,
      systemInfo: this.getSystemInfo(),
      summary: this.calculateSummary(results),
    };
    
    const filename = `stress-test-${this.machineId}.json`;
    const filepath = join(this.outputDir, filename);
    
    writeFileSync(filepath, JSON.stringify(machineResult, null, 2));
    
    console.log(`✅ Results saved to: ${filepath}`);
    console.log(`📊 Summary: ${machineResult.summary.successfulRequests}/${machineResult.summary.totalRequests} requests successful`);
    console.log(`⚡ Throughput: ${machineResult.summary.throughputRPS.toFixed(2)} RPS`);
    console.log(`📈 Avg Response Time: ${machineResult.summary.averageResponseTime.toFixed(2)}ms`);
    
    return filepath;
  }
  
  static loadMachineResults(resultDir: string): MachineTestResult[] {
    const results: MachineTestResult[] = [];
    
    if (!existsSync(resultDir)) {
      return results;
    }
    
    const files = require('fs').readdirSync(resultDir);
    
    for (const file of files) {
      if (file.startsWith('stress-test-') && file.endsWith('.json')) {
        try {
          const filepath = join(resultDir, file);
          const content = readFileSync(filepath, 'utf8');
          const result = JSON.parse(content) as MachineTestResult;
          results.push(result);
        } catch (error) {
          console.warn(`Failed to load result file ${file}:`, error);
        }
      }
    }
    
    return results;
  }
  
  static aggregateResults(machineResults: MachineTestResult[]) {
    if (machineResults.length === 0) {
      return null;
    }
    
    const allResults = machineResults.flatMap(mr => mr.results);
    const totalRequests = allResults.length;
    const successfulRequests = allResults.filter(r => r.success).length;
    const failedRequests = totalRequests - successfulRequests;
    
    const responseTimes = allResults.map(r => r.responseTime);
    const averageResponseTime = responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length || 0;
    const minResponseTime = Math.min(...responseTimes) || 0;
    const maxResponseTime = Math.max(...responseTimes) || 0;
    
    // Calculate percentiles
    const sortedTimes = responseTimes.sort((a, b) => a - b);
    const p50 = sortedTimes[Math.floor(sortedTimes.length * 0.5)] || 0;
    const p90 = sortedTimes[Math.floor(sortedTimes.length * 0.9)] || 0;
    const p95 = sortedTimes[Math.floor(sortedTimes.length * 0.95)] || 0;
    const p99 = sortedTimes[Math.floor(sortedTimes.length * 0.99)] || 0;
    
    const earliestStart = Math.min(...machineResults.map(mr => mr.startTime));
    const latestEnd = Math.max(...machineResults.map(mr => mr.endTime));
    const totalTestDuration = (latestEnd - earliestStart) / 1000;
    
    const totalThroughputRPS = totalRequests / totalTestDuration || 0;
    const errorRate = (failedRequests / totalRequests) * 100 || 0;
    
    // Machine-specific stats
    const machineStats = machineResults.map(mr => ({
      machineId: mr.machineId,
      hostname: mr.hostname,
      requests: mr.summary.totalRequests,
      successRate: ((mr.summary.successfulRequests / mr.summary.totalRequests) * 100).toFixed(2) + '%',
      avgResponseTime: mr.summary.averageResponseTime.toFixed(2) + 'ms',
      throughput: mr.summary.throughputRPS.toFixed(2) + ' RPS',
      systemInfo: mr.systemInfo,
    }));
    
    return {
      aggregatedSummary: {
        totalMachines: machineResults.length,
        totalRequests,
        successfulRequests,
        failedRequests,
        averageResponseTime,
        minResponseTime,
        maxResponseTime,
        percentiles: { p50, p90, p95, p99 },
        totalThroughputRPS,
        errorRate,
        testDuration: totalTestDuration,
      },
      machineStats,
      rawResults: allResults,
    };
  }
  
  static generateReport(aggregatedData: any): string {
    const { aggregatedSummary, machineStats } = aggregatedData;
    
    let report = '# Distributed Stress Test Report\\n\\n';
    report += `## Overall Summary\\n`;
    report += `- **Total Machines**: ${aggregatedSummary.totalMachines}\\n`;
    report += `- **Total Requests**: ${aggregatedSummary.totalRequests.toLocaleString()}\\n`;
    report += `- **Success Rate**: ${((aggregatedSummary.successfulRequests / aggregatedSummary.totalRequests) * 100).toFixed(2)}%\\n`;
    report += `- **Total Throughput**: ${aggregatedSummary.totalThroughputRPS.toFixed(2)} RPS\\n`;
    report += `- **Test Duration**: ${aggregatedSummary.testDuration.toFixed(2)}s\\n`;
    report += `- **Error Rate**: ${aggregatedSummary.errorRate.toFixed(2)}%\\n\\n`;
    
    report += `## Response Time Statistics\\n`;
    report += `- **Average**: ${aggregatedSummary.averageResponseTime.toFixed(2)}ms\\n`;
    report += `- **Minimum**: ${aggregatedSummary.minResponseTime.toFixed(2)}ms\\n`;
    report += `- **Maximum**: ${aggregatedSummary.maxResponseTime.toFixed(2)}ms\\n`;
    report += `- **50th Percentile**: ${aggregatedSummary.percentiles.p50.toFixed(2)}ms\\n`;
    report += `- **90th Percentile**: ${aggregatedSummary.percentiles.p90.toFixed(2)}ms\\n`;
    report += `- **95th Percentile**: ${aggregatedSummary.percentiles.p95.toFixed(2)}ms\\n`;
    report += `- **99th Percentile**: ${aggregatedSummary.percentiles.p99.toFixed(2)}ms\\n\\n`;
    
    report += `## Machine Performance\\n`;
    machineStats.forEach((machine: any, index: number) => {
      report += `### Machine ${index + 1}: ${machine.hostname}\\n`;
      report += `- **Machine ID**: ${machine.machineId}\\n`;
      report += `- **Requests**: ${machine.requests.toLocaleString()}\\n`;
      report += `- **Success Rate**: ${machine.successRate}\\n`;
      report += `- **Avg Response Time**: ${machine.avgResponseTime}\\n`;
      report += `- **Throughput**: ${machine.throughput}\\n`;
      report += `- **System**: ${machine.systemInfo.platform} ${machine.systemInfo.arch}, ${machine.systemInfo.cpus} CPUs, ${machine.systemInfo.memory}GB RAM\\n\\n`;
    });
    
    return report;
  }
  
  static saveAggregatedReport(resultDir: string, outputPath?: string): string {
    const machineResults = DistributedResultCollector.loadMachineResults(resultDir);
    
    if (machineResults.length === 0) {
      throw new Error('No machine results found in the specified directory');
    }
    
    const aggregatedData = DistributedResultCollector.aggregateResults(machineResults);
    if (!aggregatedData) {
      throw new Error('Failed to aggregate results');
    }
    
    const report = DistributedResultCollector.generateReport(aggregatedData);
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const reportPath = outputPath || join(resultDir, `aggregated-report-${timestamp}.md`);
    const jsonPath = join(resultDir, `aggregated-data-${timestamp}.json`);
    
    writeFileSync(reportPath, report);
    writeFileSync(jsonPath, JSON.stringify(aggregatedData, null, 2));
    
    console.log(`📊 Aggregated report saved to: ${reportPath}`);
    console.log(`📊 Aggregated data saved to: ${jsonPath}`);
    
    return reportPath;
  }
}