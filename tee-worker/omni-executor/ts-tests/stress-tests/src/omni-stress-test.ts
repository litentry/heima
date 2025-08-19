#!/usr/bin/env node

import { program } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { table } from 'table';
import { StressTestEngine } from './core/stress-engine';
import { DashboardServer } from './dashboard/server';
import { DistributedResultCollector } from './distributed/result-collector';
import { PerformanceThresholds } from './types/index';

// Default performance thresholds for Omni Executor
const DEFAULT_THRESHOLDS: PerformanceThresholds = {
  responseTime: {
    target: 500,      // 500ms target
    acceptable: 1000, // 1s acceptable
    critical: 3000,   // 3s critical
  },
  throughput: {
    target: 100,      // 100 RPS target
    minimum: 10,      // 10 RPS minimum
  },
  errorRate: {
    warning: 1,       // 1% warning
    critical: 5,      // 5% critical
  },
  systemLoad: {
    cpu: 80,          // 80% CPU warning
    memory: 85,       // 85% memory warning
    disk: 90,         // 90% disk warning
  },
};

/**
 * Run a distributed stress test for Omni UserLogin
 */
async function runOmniLoginStressTest(options: any): Promise<void> {
  const spinner = ora('🚀 Initializing Omni UserLogin stress test...').start();
  
  // Setup graceful shutdown
  let testEngine: StressTestEngine | null = null;
  const gracefulShutdown = async (signal: string) => {
    console.log(chalk.yellow(`\n⚠️  Received ${signal}, gracefully shutting down...`));
    if (testEngine) {
      console.log('🛑 Stopping test engine...');
      await testEngine.stopTest();
      await new Promise(resolve => setTimeout(resolve, 1000)); // Give it time to cleanup
    }
    console.log(chalk.blue('👋 Test stopped. Partial results may be available in ./stress-test-results/'));
    process.exit(0);
  };
  
  process.on('SIGINT', () => gracefulShutdown('SIGINT'));
  process.on('SIGTERM', () => gracefulShutdown('SIGTERM'));
  
  try {
    const config = {
      testName: options.name || `Omni Login Stress Test - ${new Date().toISOString()}`,
      targetUrl: options.url || process.env.OMNI_RPC_URL || 'http://localhost:2100/',
      duration: parseInt(options.duration) || 60,
      concurrency: parseInt(options.concurrency) || 10,
      rampUpTime: parseInt(options.rampUp) || 10,
      rampDownTime: 10,
      requestTimeout: 30000,
      retries: 3,
      outputFormat: ['json', 'console'] as ('json' | 'console' | 'excel')[],
      outputPath: './stress-test-results',
      enableDashboard: false,
      dashboardPort: 3000,
      testType: 'omni_userLogin',
    };
    
    spinner.succeed('✅ Configuration validated');
    
    console.log(chalk.blue('📋 Test Configuration:'));
    const configTable = [
      ['Parameter', 'Value'],
      ['Test Name', config.testName],
      ['Target URL', config.targetUrl],
      ['Duration', `${config.duration}s`],
      ['Concurrency', config.concurrency.toString()],
      ['Ramp Up Time', `${config.rampUpTime}s`],
      ['Test Type', config.testType],
    ];
    console.log(table(configTable));
    
    // Validate target URL connectivity
    spinner.start('🔗 Testing target URL connectivity...');
    try {
      const response = await fetch(config.targetUrl, { 
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'omni_getShieldingKey',
          params: [],
          id: 1
        })
      });
      
      if (response.ok) {
        spinner.succeed('✅ Target URL is reachable');
      } else {
        spinner.warn(`⚠️  Target returned ${response.status}, continuing anyway...`);
      }
    } catch (error) {
      spinner.warn(`⚠️  Cannot reach target (${error}), continuing anyway...`);
    }
    
    // Start the test
    const engine = new StressTestEngine();
    testEngine = engine;
    
    // Setup event listeners
    engine.on('testStarted', ({ sessionId }) => {
      console.log(chalk.green(`🏁 Test started with session ID: ${chalk.cyan(sessionId)}`));
    });
    
    let requestCount = 0;
    let errorCount = 0;
    let lastUpdate = Date.now();
    
    engine.on('requestCompleted', () => {
      requestCount++;
      if (Date.now() - lastUpdate > 2000) { // Update every 2 seconds
        console.log(chalk.gray(`📊 Progress: ${requestCount} completed, ${errorCount} errors`));
        lastUpdate = Date.now();
      }
    });
    
    engine.on('requestFailed', () => {
      errorCount++;
      if (Date.now() - lastUpdate > 2000) {
        console.log(chalk.gray(`📊 Progress: ${requestCount} completed, ${chalk.red(errorCount + ' errors')}`));
        lastUpdate = Date.now();
      }
    });
    
    engine.on('testCompleted', ({ summary }) => {
      console.log(chalk.green('🎉 Test completed successfully!'));
      displayOmniTestSummary(summary);
      analyzePerformanceThresholds(summary, DEFAULT_THRESHOLDS);
    });
    
    engine.on('testFailed', ({ error }) => {
      console.error(chalk.red('💥 Test failed:'), error);
      process.exit(1);
    });
    
    // Run the stress test
    await engine.startTest(config);
    
  } catch (error) {
    spinner.fail('❌ Failed to run stress test');
    console.error(chalk.red('Error:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

/**
 * Start the dashboard server for real-time monitoring
 */
async function startDashboard(options: any): Promise<void> {
  const port = parseInt(options.port) || 3000;
  const spinner = ora('🌐 Starting dashboard server...').start();
  
  try {
    const dashboard = new DashboardServer(port);
    await dashboard.start();
    
    spinner.succeed(`✅ Dashboard server started`);
    console.log(`🔗 Dashboard URL: ${chalk.cyan(`http://localhost:${port}`)}`);
    console.log('📊 Open this URL in your browser to monitor tests in real-time');
    console.log('💡 Press Ctrl+C to stop the server');
    
    // Keep the process alive
    process.on('SIGINT', async () => {
      console.log('\\n⏹️  Stopping dashboard server...');
      await dashboard.stop();
      process.exit(0);
    });
    
    // Prevent the process from exiting
    await new Promise(() => {});
    
  } catch (error) {
    spinner.fail('❌ Failed to start dashboard server');
    console.error(chalk.red('Error:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

/**
 * Aggregate results from multiple machines
 */
async function aggregateResults(options: any): Promise<void> {
  const resultDir = options.directory || './stress-test-results';
  const spinner = ora('📊 Aggregating distributed test results...').start();
  
  try {
    const reportPath = DistributedResultCollector.saveAggregatedReport(resultDir, options.output);
    spinner.succeed('✅ Aggregated report generated');
    console.log(`📋 Report saved to: ${chalk.cyan(reportPath)}`);
    
  } catch (error) {
    spinner.fail('❌ Failed to aggregate results');
    console.error(chalk.red('Error:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

/**
 * Display test summary with Omni-specific metrics
 */
function displayOmniTestSummary(summary: any): void {
  console.log('\\n' + chalk.blue.bold('📊 Omni UserLogin Performance Summary'));
  console.log('='.repeat(60));
  
  // Overall Performance
  const overallData = [
    ['Metric', 'Value', 'Status'],
    ['Total Requests', summary.totalRequests.toLocaleString(), getStatusIcon(true)],
    ['Success Rate', `${((summary.successfulRequests / summary.totalRequests) * 100).toFixed(2)}%`, getStatusIcon(summary.successfulRequests / summary.totalRequests > 0.95)],
    ['Average Response Time', `${summary.responseTime.average.toFixed(2)}ms`, getStatusIcon(summary.responseTime.average < DEFAULT_THRESHOLDS.responseTime.acceptable)],
    ['Throughput (RPS)', summary.throughput.rps.toFixed(2), getStatusIcon(summary.throughput.rps > DEFAULT_THRESHOLDS.throughput.minimum)],
    ['Error Rate', `${summary.errorRate.total.toFixed(2)}%`, getStatusIcon(summary.errorRate.total < DEFAULT_THRESHOLDS.errorRate.warning)],
  ];
  
  console.log(table(overallData));
  
  // Response Time Distribution
  console.log(chalk.blue('⚡ Response Time Distribution:'));
  const responseTimeData = [
    ['Percentile', 'Response Time', 'Status'],
    ['50th (Median)', `${summary.responseTime.p50.toFixed(2)}ms`, getStatusIcon(summary.responseTime.p50 < DEFAULT_THRESHOLDS.responseTime.target)],
    ['90th', `${summary.responseTime.p90.toFixed(2)}ms`, getStatusIcon(summary.responseTime.p90 < DEFAULT_THRESHOLDS.responseTime.acceptable)],
    ['95th', `${summary.responseTime.p95.toFixed(2)}ms`, getStatusIcon(summary.responseTime.p95 < DEFAULT_THRESHOLDS.responseTime.acceptable)],
    ['99th', `${summary.responseTime.p99.toFixed(2)}ms`, getStatusIcon(summary.responseTime.p99 < DEFAULT_THRESHOLDS.responseTime.critical)],
    ['Maximum', `${summary.responseTime.max.toFixed(2)}ms`, getStatusIcon(summary.responseTime.max < DEFAULT_THRESHOLDS.responseTime.critical)],
  ];
  
  console.log(table(responseTimeData));
}

/**
 * Analyze performance against thresholds
 */
function analyzePerformanceThresholds(summary: any, thresholds: PerformanceThresholds): void {
  console.log(chalk.blue.bold('🎯 Performance Analysis'));
  console.log('='.repeat(60));
  
  const issues: string[] = [];
  
  // Response Time Analysis
  if (summary.responseTime.average > thresholds.responseTime.critical) {
    issues.push(`❌ Critical: Average response time (${summary.responseTime.average.toFixed(2)}ms) exceeds critical threshold (${thresholds.responseTime.critical}ms)`);
  } else if (summary.responseTime.average > thresholds.responseTime.acceptable) {
    issues.push(`⚠️  Warning: Average response time (${summary.responseTime.average.toFixed(2)}ms) exceeds acceptable threshold (${thresholds.responseTime.acceptable}ms)`);
  }
  
  // Throughput Analysis  
  if (summary.throughput.rps < thresholds.throughput.minimum) {
    issues.push(`❌ Critical: Throughput (${summary.throughput.rps.toFixed(2)} RPS) below minimum requirement (${thresholds.throughput.minimum} RPS)`);
  } else if (summary.throughput.rps < thresholds.throughput.target) {
    issues.push(`⚠️  Warning: Throughput (${summary.throughput.rps.toFixed(2)} RPS) below target (${thresholds.throughput.target} RPS)`);
  }
  
  // Error Rate Analysis
  if (summary.errorRate.total > thresholds.errorRate.critical) {
    issues.push(`❌ Critical: Error rate (${summary.errorRate.total.toFixed(2)}%) exceeds critical threshold (${thresholds.errorRate.critical}%)`);
  } else if (summary.errorRate.total > thresholds.errorRate.warning) {
    issues.push(`⚠️  Warning: Error rate (${summary.errorRate.total.toFixed(2)}%) exceeds warning threshold (${thresholds.errorRate.warning}%)`);
  }
  
  if (issues.length === 0) {
    console.log(chalk.green('✅ All performance metrics within acceptable thresholds!'));
    console.log(chalk.green('🚀 System can handle the current load comfortably'));
  } else {
    console.log(chalk.yellow('⚠️  Performance issues detected:'));
    issues.forEach(issue => console.log(`  ${issue}`));
  }
  
  // Capacity recommendations
  console.log('\\n' + chalk.blue.bold('💡 Capacity Recommendations:'));
  const maxSustainableRPS = Math.floor(summary.throughput.rps * 0.8); // 80% of observed
  const recommendedConcurrency = Math.ceil(summary.concurrency * 0.7); // 70% of tested
  
  console.log(`📈 Maximum sustainable RPS: ~${maxSustainableRPS}`);
  console.log(`👥 Recommended max concurrency: ~${recommendedConcurrency}`);
  console.log(`⚡ Response time target: <${thresholds.responseTime.target}ms for 95% of requests`);
}

function getStatusIcon(isGood: boolean): string {
  return isGood ? chalk.green('✅') : chalk.red('❌');
}

// CLI Configuration
program
  .name('omni-stress-test')
  .description('🚀 Professional stress testing tool for Omni Executor UserLogin API')
  .version('1.0.0');

// Main stress test command
program
  .command('run')
  .description('🎯 Run Omni UserLogin stress test')
  .option('-n, --name <string>', 'Test name', 'Omni UserLogin Stress Test')
  .option('-d, --duration <number>', 'Test duration in seconds', '60')
  .option('-c, --concurrency <number>', 'Number of concurrent users', '10')
  .option('-r, --ramp-up <number>', 'Ramp up time in seconds', '10')
  .option('-u, --url <string>', 'Target RPC URL', process.env.OMNI_RPC_URL || 'http://localhost:2100/')
  .action(runOmniLoginStressTest);

// Dashboard command
program
  .command('dashboard')
  .description('🌐 Start real-time monitoring dashboard')
  .option('-p, --port <number>', 'Dashboard port', '3000')
  .action(startDashboard);

// Aggregate results command
program
  .command('aggregate')
  .description('📊 Aggregate distributed test results')
  .option('-d, --directory <path>', 'Results directory', './stress-test-results')
  .option('-o, --output <path>', 'Output report path')
  .action(aggregateResults);

// Quick test command
program
  .command('quick')
  .description('⚡ Run a quick 30-second test')
  .option('-u, --url <string>', 'Target RPC URL', process.env.OMNI_RPC_URL || 'http://localhost:2100/')
  .action((options) => {
    runOmniLoginStressTest({
      name: 'Quick Omni Login Test',
      duration: '30',
      concurrency: '5',
      rampUp: '5',
      url: options.url
    });
  });

// Example command
program
  .command('example')
  .description('📖 Show usage examples')
  .action(() => {
    console.log(chalk.blue.bold('🌟 Omni UserLogin Stress Test Examples\\n'));
    
    console.log(chalk.green('# Quick test (30 seconds, 5 concurrent users)'));
    console.log('npm run stress quick\\n');
    
    console.log(chalk.green('# Standard test (60 seconds, 10 concurrent users)'));
    console.log('npm run stress run\\n');
    
    console.log(chalk.green('# High load test (5 minutes, 50 concurrent users)'));
    console.log('npm run stress run -d 300 -c 50 -r 30\\n');
    
    console.log(chalk.green('# Custom target URL'));
    console.log('npm run stress run -u http://production-server:2100/\\n');
    
    console.log(chalk.green('# Start monitoring dashboard'));
    console.log('npm run stress dashboard\\n');
    
    console.log(chalk.green('# Aggregate distributed results'));
    console.log('npm run stress aggregate -d ./test-results\\n');
    
    console.log(chalk.blue('💡 Pro Tips:'));
    console.log('- Run multiple instances across different machines for distributed testing');
    console.log('- Use the dashboard for real-time monitoring during tests');
    console.log('- Aggregate results from multiple machines for comprehensive analysis');
    console.log('- Monitor system resources during high-load tests');
  });

// Export class for use in other modules
export class OmniStressTest {
  async run(config: any): Promise<void> {
    return runOmniLoginStressTest(config);
  }
  
  async startDashboard(port: number = 3000): Promise<void> {
    return startDashboard({ port });
  }
  
  async aggregateResults(directory: string = './stress-test-results'): Promise<void> {
    return aggregateResults({ directory });
  }
}

// Parse CLI arguments only if this file is run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  program.parse();
}