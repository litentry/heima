#!/usr/bin/env node

import { program } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { table } from 'table';
import { StressTestEngine } from './core/stress-engine';
import { DashboardServer } from './dashboard/server';
import { DataStorage } from './storage/data-storage';
import { StressTestConfig } from './types/index';

const DEFAULT_CONFIG: StressTestConfig = {
  testName: 'Default Stress Test',
  duration: 60,
  concurrency: 10,
  rampUpTime: 10,
  rampDownTime: 10,
  targetUrl: process.env.OMNI_RPC_URL || 'http://localhost:2100/',
  requestTimeout: 30000,
  retries: 3,
  outputFormat: ['json', 'console'],
  outputPath: './stress-test-results',
  enableDashboard: false,
  dashboardPort: 3000
};

async function runStressTest(config: StressTestConfig): Promise<void> {
  const spinner = ora('Initializing stress test...').start();
  
  try {
    const engine = new StressTestEngine();
    
    // Setup event listeners for console output
    engine.on('testStarting', ({ sessionId, config }) => {
      spinner.succeed(`Test initialized with session ID: ${chalk.cyan(sessionId)}`);
      console.log(chalk.blue('Test Configuration:'));
      console.log(`  Test Name: ${config.testName}`);
      console.log(`  Duration: ${config.duration}s`);
      console.log(`  Concurrency: ${config.concurrency}`);
      console.log(`  Ramp Up: ${config.rampUpTime}s`);
      console.log(`  Target URL: ${config.targetUrl}`);
      console.log();
    });
    
    engine.on('testStarted', () => {
      spinner.text = 'Running stress test...';
      spinner.start();
    });
    
    let requestCount = 0;
    let errorCount = 0;
    
    engine.on('requestCompleted', () => {
      requestCount++;
      if (requestCount % 10 === 0) {
        spinner.text = `Processed ${requestCount} requests, ${errorCount} errors`;
      }
    });
    
    engine.on('requestFailed', () => {
      errorCount++;
      if ((requestCount + errorCount) % 10 === 0) {
        spinner.text = `Processed ${requestCount} requests, ${errorCount} errors`;
      }
    });
    
    engine.on('testCompleted', ({ summary }) => {
      spinner.succeed('Stress test completed successfully!');
      console.log();
      displaySummary(summary);
    });
    
    engine.on('testFailed', ({ error }) => {
      spinner.fail('Stress test failed');
      console.error(chalk.red('Error:'), error);
    });
    
    // Start the test
    await engine.startTest(config);
    
  } catch (error) {
    spinner.fail('Failed to run stress test');
    console.error(chalk.red('Error:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

function displaySummary(summary: any): void {
  console.log(chalk.green.bold('📊 Test Summary'));
  console.log('='.repeat(50));
  
  // Response Time Metrics
  console.log(chalk.blue('📈 Response Time Metrics:'));
  const responseTimeData = [
    ['Metric', 'Value'],
    ['Average', `${summary.responseTime.average.toFixed(2)}ms`],
    ['Minimum', `${summary.responseTime.min.toFixed(2)}ms`],
    ['Maximum', `${summary.responseTime.max.toFixed(2)}ms`],
    ['50th Percentile', `${summary.responseTime.p50.toFixed(2)}ms`],
    ['90th Percentile', `${summary.responseTime.p90.toFixed(2)}ms`],
    ['95th Percentile', `${summary.responseTime.p95.toFixed(2)}ms`],
    ['99th Percentile', `${summary.responseTime.p99.toFixed(2)}ms`]
  ];
  console.log(table(responseTimeData));
  
  // Throughput Metrics
  console.log(chalk.blue('🚀 Throughput Metrics:'));
  const throughputData = [
    ['Metric', 'Value'],
    ['Requests per Second', summary.throughput.rps.toFixed(2)],
    ['Transactions per Second', summary.throughput.tps.toFixed(2)],
    ['Data Transfer Rate', `${(summary.throughput.dataTransferRate / 1024).toFixed(2)} KB/s`],
    ['Peak Concurrency', summary.throughput.peakConcurrency.toString()]
  ];
  console.log(table(throughputData));
  
  // Error Rate Metrics
  console.log(chalk.blue('❌ Error Rate Metrics:'));
  const errorData = [
    ['Metric', 'Value'],
    ['Total Error Rate', `${summary.errorRate.total.toFixed(2)}%`],
    ['HTTP 4xx Errors', `${summary.errorRate.http4xx.toFixed(2)}%`],
    ['HTTP 5xx Errors', `${summary.errorRate.http5xx.toFixed(2)}%`],
    ['Timeout Errors', `${summary.errorRate.timeout.toFixed(2)}%`],
    ['Connection Errors', `${summary.errorRate.connection.toFixed(2)}%`]
  ];
  console.log(table(errorData));
  
  // Overall Stats
  console.log(chalk.blue('📊 Overall Statistics:'));
  const overallData = [
    ['Metric', 'Value'],
    ['Total Requests', summary.totalRequests.toString()],
    ['Successful Requests', summary.successfulRequests.toString()],
    ['Failed Requests', summary.failedRequests.toString()],
    ['Success Rate', `${((summary.successfulRequests / summary.totalRequests) * 100).toFixed(2)}%`],
    ['Test Duration', `${summary.testDuration.toFixed(2)}s`]
  ];
  console.log(table(overallData));
}

async function startDashboard(port: number): Promise<void> {
  const spinner = ora('Starting dashboard server...').start();
  
  try {
    const dashboard = new DashboardServer(port);
    await dashboard.start();
    spinner.succeed(`Dashboard server started at ${chalk.cyan(`http://localhost:${port}`)}`);
    console.log('Press Ctrl+C to stop the server');
    
    // Keep the process alive
    process.on('SIGINT', async () => {
      console.log('\nStopping dashboard server...');
      await dashboard.stop();
      process.exit(0);
    });
    
    // Prevent the process from exiting
    await new Promise(() => {});
    
  } catch (error) {
    spinner.fail('Failed to start dashboard server');
    console.error(chalk.red('Error:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

async function listSessions(): Promise<void> {
  try {
    const storage = new DataStorage();
    const sessionIds = await storage.listTestSessions();
    
    if (sessionIds.length === 0) {
      console.log(chalk.yellow('No test sessions found.'));
      return;
    }
    
    console.log(chalk.blue(`Found ${sessionIds.length} test sessions:`));
    
    const sessions = [];
    for (const id of sessionIds.slice(0, 10)) { // Show latest 10
      const session = await storage.loadTestSession(id);
      if (session) {
        sessions.push([
          id.substring(0, 20) + '...',
          session.config.testName,
          session.status,
          new Date(session.startTime).toLocaleString(),
          session.results.length.toString(),
          session.summary ? `${session.summary.successfulRequests}/${session.summary.totalRequests}` : 'N/A'
        ]);
      }
    }
    
    const headers = ['Session ID', 'Test Name', 'Status', 'Start Time', 'Requests', 'Success/Total'];
    console.log(table([headers, ...sessions]));
    
  } catch (error) {
    console.error(chalk.red('Error:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// CLI Setup
program
  .name('omni-stress-test')
  .description('Professional stress testing framework for Omni Executor')
  .version('1.0.0');

program
  .command('run')
  .description('Run a stress test')
  .option('-n, --name <string>', 'Test name', DEFAULT_CONFIG.testName)
  .option('-d, --duration <number>', 'Test duration in seconds', DEFAULT_CONFIG.duration.toString())
  .option('-c, --concurrency <number>', 'Number of concurrent users', DEFAULT_CONFIG.concurrency.toString())
  .option('-r, --ramp-up <number>', 'Ramp up time in seconds', DEFAULT_CONFIG.rampUpTime.toString())
  .option('-u, --url <string>', 'Target URL', process.env.OMNI_RPC_URL || DEFAULT_CONFIG.targetUrl)
  .option('-t, --timeout <number>', 'Request timeout in milliseconds', DEFAULT_CONFIG.requestTimeout.toString())
  .option('--retries <number>', 'Number of retries for failed requests', DEFAULT_CONFIG.retries.toString())
  .option('-o, --output <path>', 'Output directory path', DEFAULT_CONFIG.outputPath)
  .option('--dashboard', 'Enable real-time dashboard')
  .option('--dashboard-port <number>', 'Dashboard server port', DEFAULT_CONFIG.dashboardPort.toString())
  .action(async (options) => {
    const config: StressTestConfig = {
      testName: options.name,
      duration: parseInt(options.duration),
      concurrency: parseInt(options.concurrency),
      rampUpTime: parseInt(options.rampUp),
      rampDownTime: DEFAULT_CONFIG.rampDownTime,
      targetUrl: options.url,
      requestTimeout: parseInt(options.timeout),
      retries: parseInt(options.retries),
      outputFormat: ['json', 'excel', 'console'],
      outputPath: options.output,
      enableDashboard: !!options.dashboard,
      dashboardPort: parseInt(options.dashboardPort)
    };
    
    await runStressTest(config);
  });

program
  .command('dashboard')
  .description('Start the dashboard server')
  .option('-p, --port <number>', 'Port to run dashboard on', '3000')
  .action(async (options) => {
    await startDashboard(parseInt(options.port));
  });

program
  .command('list')
  .description('List previous test sessions')
  .action(listSessions);

program
  .command('export')
  .description('Export test sessions to JSON')
  .option('-s, --sessions <ids...>', 'Session IDs to export (space separated)')
  .action(async (options) => {
    try {
      const storage = new DataStorage();
      const exportPath = await storage.exportSessionsToJson(options.sessions);
      console.log(chalk.green(`Data exported to: ${exportPath}`));
    } catch (error) {
      console.error(chalk.red('Error:'), error instanceof Error ? error.message : String(error));
      process.exit(1);
    }
  });

// Add example command
program
  .command('example')
  .description('Show example usage')
  .action(() => {
    console.log(chalk.blue('Example Usage:'));
    console.log('');
    console.log('  # Run a basic stress test');
    console.log('  npm run stress -- run -n "API Load Test" -d 120 -c 50');
    console.log('');
    console.log('  # Run with dashboard enabled');
    console.log('  npm run stress -- run --dashboard -d 300 -c 100');
    console.log('');
    console.log('  # Start dashboard server only');
    console.log('  npm run dashboard');
    console.log('');
    console.log('  # List previous sessions');
    console.log('  npm run stress -- list');
    console.log('');
    console.log('  # Export sessions to JSON');
    console.log('  npm run stress -- export -s session-id-1 session-id-2');
  });

// Parse CLI arguments
program.parse();