#!/usr/bin/env node

import { program } from 'commander';
import chalk from 'chalk';
import { table } from 'table';
import { QPSRampEngine, QPSRampConfig } from './core/qps-ramp-engine';

/**
 * QPS Ramp Test CLI - Gradually increase QPS to find system limits
 */

async function runQPSRampTest(options: any): Promise<void> {
  console.log(chalk.blue.bold('QPS Ramp Test - Find System Capacity Limits\\n'));
  
  const config: QPSRampConfig = {
    testName: options.name || `QPS Ramp Test - ${new Date().toISOString()}`,
    targetUrl: options.url || process.env.OMNI_RPC_URL || 'http://localhost:2100/',
    startQPS: parseInt(options.startQps) || 1,
    maxQPS: parseInt(options.maxQps) || 50,
    stepSize: parseInt(options.stepSize) || 5,
    stepDuration: parseInt(options.stepDuration) || 30,
    testType: options.testType || 'omni_userLogin',
    outputPath: options.output || './stress-test-results',
  };

  // Validate configuration
  if (config.startQPS >= config.maxQPS) {
    console.error(chalk.red('❌ Start QPS must be less than max QPS'));
    process.exit(1);
  }

  if (config.stepSize <= 0) {
    console.error(chalk.red('❌ Step size must be greater than 0'));
    process.exit(1);
  }

  console.log(chalk.blue('QPS Ramp Configuration:'));
  const configTable = [
    ['Parameter', 'Value'],
    ['Test Name', config.testName],
    ['Target URL', config.targetUrl],
    ['QPS Range', `${config.startQPS} → ${config.maxQPS}`],
    ['Step Size', config.stepSize.toString()],
    ['Step Duration', `${config.stepDuration}s`],
    ['Test Type', config.testType],
    ['Estimated Duration', `${Math.ceil((config.maxQPS - config.startQPS) / config.stepSize + 1) * config.stepDuration}s`],
  ];
  console.log(table(configTable));

  try {
    const engine = new QPSRampEngine();

    // Setup event listeners
    engine.on('testStarted', ({ startTime }) => {
      console.log(chalk.green(`QPS ramp test started at ${new Date(startTime).toLocaleTimeString()}`));
    });

    engine.on('stepCompleted', (stepResult) => {
      const statusIcon = stepResult.errorRate < 5 ? '✅' : stepResult.errorRate < 20 ? '⚠️' : '❌';
      console.log(
        `${statusIcon} QPS ${stepResult.qps}: ` +
        `${stepResult.successfulRequests}/${stepResult.totalRequests} requests, ` +
        `${stepResult.errorRate.toFixed(1)}% errors, ` +
        `${stepResult.averageResponseTime.toFixed(0)}ms avg response`
      );

      if (stepResult.systemBreakpoint) {
        console.log(chalk.yellow(`System stability degraded at ${stepResult.qps} QPS`));
      }
    });

    engine.on('testCompleted', (result) => {
      console.log(chalk.green('\\nQPS Ramp Test completed!'));
      displayQPSResults(result);
    });

    engine.on('testFailed', ({ error }) => {
      console.error(chalk.red('QPS ramp test failed:'), error);
      process.exit(1);
    });

    // Run the test
    await engine.startQPSRampTest(config);

  } catch (error) {
    console.error(chalk.red('❌ Failed to run QPS ramp test:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

function displayQPSResults(result: any): void {
  console.log('\\n' + chalk.blue.bold('QPS Ramp Test Results'));
  console.log('='.repeat(80));

  // Summary table
  const summaryData = [
    ['Metric', 'Value'],
    ['Max Sustainable QPS', result.maxSustainableQPS.toString()],
    ['Breakpoint QPS', result.breakpointQPS?.toString() || 'Not reached'],
    ['Total Requests', result.summary.totalRequests.toLocaleString()],
    ['Overall Success Rate', `${result.summary.overallSuccessRate.toFixed(1)}%`],
    ['Total Test Duration', `${result.summary.totalDuration.toFixed(0)}s`],
  ];
  console.log(table(summaryData));

  // Step-by-step results
  console.log('\\n' + chalk.blue.bold('Step-by-Step Results'));
  const stepHeaders = ['QPS', 'Requests', 'Success Rate', 'Avg Response', 'P95 Response', 'Error Rate', 'Status'];
  const stepData = [stepHeaders];
  
  result.steps.forEach((step: any) => {
    const status = step.errorRate < 5 ? chalk.green('✅ Good') : 
                   step.errorRate < 20 ? chalk.yellow('⚠️  Warning') : 
                   chalk.red('❌ Poor');
    
    stepData.push([
      step.qps.toString(),
      `${step.successfulRequests}/${step.totalRequests}`,
      `${((step.successfulRequests / step.totalRequests) * 100).toFixed(1)}%`,
      `${step.averageResponseTime.toFixed(0)}ms`,
      `${step.p95ResponseTime.toFixed(0)}ms`,
      `${step.errorRate.toFixed(1)}%`,
      status,
    ]);
  });
  
  console.log(table(stepData));

  // Error analysis
  const allErrors = new Map<string, number>();
  result.steps.forEach((step: any) => {
    Object.entries(step.detailedErrors).forEach(([errorType, count]: [string, any]) => {
      allErrors.set(errorType, (allErrors.get(errorType) || 0) + count);
    });
  });

  if (allErrors.size > 0) {
    console.log('\\n' + chalk.blue.bold('Error Analysis'));
    const errorData = [['Error Type', 'Count', 'Percentage']];
    const totalErrors = Array.from(allErrors.values()).reduce((sum, count) => sum + count, 0);
    
    allErrors.forEach((count, errorType) => {
      const percentage = ((count / totalErrors) * 100).toFixed(1);
      errorData.push([errorType, count.toString(), `${percentage}%`]);
    });
    
    console.log(table(errorData));
  }

  // Recommendations
  console.log('\\n' + chalk.blue.bold('Recommendations'));
  result.summary.recommendations.forEach((rec: string, index: number) => {
    console.log(`  ${index + 1}. ${rec}`);
  });

  // Capacity planning
  console.log('\\n' + chalk.blue.bold('Capacity Planning Guidelines'));
  const safeQPS = Math.floor(result.maxSustainableQPS * 0.7);
  const warningQPS = Math.floor(result.maxSustainableQPS * 0.85);
  
  console.log(`Production Safe QPS: ${safeQPS} (70% of max sustainable)`);
  console.log(`Warning Threshold: ${warningQPS} (85% of max sustainable)`);
  console.log(`Critical Threshold: ${result.maxSustainableQPS} (maximum sustainable)`);
  
  if (result.breakpointQPS) {
    console.log(`Avoid QPS Above: ${result.breakpointQPS} (system becomes unstable)`);
  }
}

async function runSpecificAPITest(options: any): Promise<void> {
  const testConfigs = {
    'shielding-key': {
      testType: 'omni_getShieldingKey',
      name: 'Shielding Key QPS Test',
      maxQPS: 100,
    },
    'add-wallet': {
      testType: 'omni_addWallet', 
      name: 'Add Wallet QPS Test',
      maxQPS: 30, // Lower max since this requires authentication
    },
    'mixed': {
      testType: 'mixed',
      name: 'Mixed API QPS Test',
      maxQPS: 80,
    },
    'weighted-mixed': {
      testType: 'weighted_mixed',
      name: 'Weighted Mixed API QPS Test',
      maxQPS: 80,
    },
  };

  const testType = options.api;
  const testConfig = testConfigs[testType as keyof typeof testConfigs];
  
  if (!testConfig) {
    console.error(chalk.red(`❌ Unknown API test type: ${testType}`));
    console.log(chalk.blue('Available options: shielding-key, add-wallet, mixed, weighted-mixed'));
    process.exit(1);
  }

  await runQPSRampTest({
    ...options,
    name: testConfig.name,
    testType: testConfig.testType,
    maxQps: options.maxQps || testConfig.maxQPS,
  });
}

// CLI setup
program
  .name('qps-ramp-test')
  .description('📈 QPS Ramp Test - Gradually increase load to find system capacity limits')
  .version('1.0.0');

// Main QPS ramp test command
program
  .command('run')
  .description('🚀 Run QPS ramp test')
  .option('-n, --name <string>', 'Test name')
  .option('-u, --url <string>', 'Target RPC URL', process.env.OMNI_RPC_URL || 'http://localhost:2100/')
  .option('--start-qps <number>', 'Starting QPS', '1')
  .option('--max-qps <number>', 'Maximum QPS', '50')
  .option('--step-size <number>', 'QPS increase per step', '5')
  .option('--step-duration <number>', 'Duration for each step (seconds)', '30')
  .option('--test-type <string>', 'Test type: omni_userLogin, omni_getShieldingKey, omni_addWallet, mixed, weighted_mixed', 'omni_userLogin')
  .option('-o, --output <path>', 'Output directory', './stress-test-results')
  .action(runQPSRampTest);

// API-specific test commands
program
  .command('api <api>')
  .description('🎯 Run QPS test for specific API (shielding-key, add-wallet, mixed, weighted-mixed)')
  .option('-u, --url <string>', 'Target RPC URL', process.env.OMNI_RPC_URL || 'http://localhost:2100/')
  .option('--start-qps <number>', 'Starting QPS', '1')
  .option('--max-qps <number>', 'Maximum QPS (will use API-specific default if not specified)')
  .option('--step-size <number>', 'QPS increase per step', '5')
  .option('--step-duration <number>', 'Duration for each step (seconds)', '30')
  .option('-o, --output <path>', 'Output directory', './stress-test-results')
  .action(runSpecificAPITest);

// Quick test commands
program
  .command('quick-login')
  .description('⚡ Quick QPS test for UserLogin (1-20 QPS)')
  .option('-u, --url <string>', 'Target RPC URL', process.env.OMNI_RPC_URL || 'http://localhost:2100/')
  .action((options) => {
    runQPSRampTest({
      name: 'Quick UserLogin QPS Test',
      url: options.url || process.env.OMNI_RPC_URL,
      startQps: '1',
      maxQps: '20',
      stepSize: '3',
      stepDuration: '20',
      testType: 'omni_userLogin',
    });
  });

program
  .command('quick-shielding')
  .description('⚡ Quick QPS test for ShieldingKey (1-30 QPS)')
  .option('-u, --url <string>', 'Target RPC URL', process.env.OMNI_RPC_URL || 'http://localhost:2100/')
  .action((options) => {
    runQPSRampTest({
      name: 'Quick ShieldingKey QPS Test',
      url: options.url || process.env.OMNI_RPC_URL,
      startQps: '1',
      maxQps: '30',
      stepSize: '5',
      stepDuration: '20',
      testType: 'omni_getShieldingKey',
    });
  });

// Example command
program
  .command('example')
  .description('📖 Show QPS ramp test examples')
  .action(() => {
    console.log(chalk.blue.bold('🌟 QPS Ramp Test Examples\\n'));
    
    console.log(chalk.green('# Basic QPS ramp test (1-50 QPS, 5 QPS steps)'));
    console.log('npm run qps-test run\\n');
    
    console.log(chalk.green('# Custom QPS range and steps'));
    console.log('npm run qps-test run --start-qps 2 --max-qps 100 --step-size 10 --step-duration 45\\n');
    
    console.log(chalk.green('# Test specific APIs'));
    console.log('npm run qps-test api shielding-key');
    console.log('npm run qps-test api add-wallet');
    console.log('npm run qps-test api mixed\\n');
    
    console.log(chalk.green('# Quick tests'));
    console.log('npm run qps-test quick-login');
    console.log('npm run qps-test quick-shielding\\n');
    
    console.log(chalk.blue('💡 Tips:'));
    console.log('- Start with lower QPS ranges for initial testing');
    console.log('- Use longer step durations for more accurate results');
    console.log('- Monitor system resources during high QPS tests');
    console.log('- Mixed tests provide more realistic load patterns');
  });

// Parse CLI arguments
if (import.meta.url === `file://${process.argv[1]}`) {
  program.parse();
}