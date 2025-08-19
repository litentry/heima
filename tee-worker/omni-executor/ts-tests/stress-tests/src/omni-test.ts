#!/usr/bin/env node

import 'dotenv/config';
import { program } from 'commander';
import chalk from 'chalk';
import { table } from 'table';
import { QPSRampEngine, QPSRampConfig } from './core/qps-ramp-engine';
import { OmniStressTest } from './omni-stress-test';

/**
 * Unified Omni Executor Test Suite
 * Combines stress testing, QPS ramping, and performance analysis
 */

// Load environment configuration
const DEFAULT_URL = process.env.OMNI_RPC_URL || 'http://localhost:2100/';

async function runStandardStressTest(options: any): Promise<void> {
  console.log(chalk.blue.bold('🚀 Standard Stress Test\n'));
  console.log(chalk.gray('📝 Note: Press Ctrl+C to gracefully stop the test at any time\n'));
  
  const stressTest = new OmniStressTest();
  
  const config = {
    testName: options.name || `Stress Test - ${new Date().toISOString()}`,
    targetUrl: options.url || DEFAULT_URL,
    duration: parseInt(options.duration) || 300,
    concurrency: parseInt(options.concurrency) || 10,
    testType: options.testType || 'omni_userLogin',
    outputPath: options.output || './test-results',
  };

  console.log(chalk.blue('Test Configuration:'));
  const configTable = [
    ['Parameter', 'Value'],
    ['Test Name', config.testName],
    ['Target URL', config.targetUrl],
    ['Duration', `${config.duration}s`],
    ['Concurrency', config.concurrency.toString()],
    ['Test Type', config.testType],
    ['Output Path', config.outputPath],
  ];
  console.log(table(configTable));

  try {
    await stressTest.run(config);
    console.log(chalk.green('\n✅ Standard stress test completed successfully!'));
  } catch (error) {
    console.error(chalk.red('❌ Stress test failed:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

async function runQPSRampTest(options: any): Promise<void> {
  console.log(chalk.blue.bold('📈 QPS Ramp Test\n'));
  
  const config: QPSRampConfig = {
    testName: options.name || `QPS Ramp Test - ${new Date().toISOString()}`,
    targetUrl: options.url || DEFAULT_URL,
    startQPS: parseInt(options.startQps) || 1,
    maxQPS: parseInt(options.maxQps) || 50,
    stepSize: parseInt(options.stepSize) || 5,
    stepDuration: parseInt(options.stepDuration) || 30,
    testType: options.testType || 'omni_userLogin',
    outputPath: options.output || './test-results',
  };

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

    const result = await engine.startQPSRampTest(config);
    
    console.log(chalk.green('\n✅ QPS ramp test completed!'));
    console.log(`Max Sustainable QPS: ${result.maxSustainableQPS}`);
    if (result.breakpointQPS) {
      console.log(`Breakpoint QPS: ${result.breakpointQPS}`);
    }
    console.log(`Production Safe QPS: ${Math.floor(result.maxSustainableQPS * 0.7)} (recommended)`);

  } catch (error) {
    console.error(chalk.red('❌ QPS ramp test failed:'), error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

async function runComprehensiveTest(options: any): Promise<void> {
  console.log(chalk.blue.bold('🌟 Comprehensive Test Suite\n'));
  console.log(chalk.yellow('Running all API tests with QPS analysis...\n'));

  const baseUrl = options.url || DEFAULT_URL;
  const testScenarios: QPSRampConfig[] = [
    {
      testName: 'UserLogin QPS Analysis',
      targetUrl: baseUrl,
      startQPS: 1,
      maxQPS: 50,
      stepSize: 5,
      stepDuration: 30,
      testType: 'omni_userLogin',
      outputPath: './test-results/user-login',
    },
    {
      testName: 'ShieldingKey QPS Analysis',
      targetUrl: baseUrl,
      startQPS: 2,
      maxQPS: 80,
      stepSize: 8,
      stepDuration: 25,
      testType: 'omni_getShieldingKey',
      outputPath: './test-results/shielding-key',
    },
    {
      testName: 'AddWallet QPS Analysis',
      targetUrl: baseUrl,
      startQPS: 1,
      maxQPS: 20,
      stepSize: 2,
      stepDuration: 40,
      testType: 'omni_addWallet',
      outputPath: './test-results/add-wallet',
    },
    {
      testName: 'Mixed API QPS Analysis',
      targetUrl: baseUrl,
      startQPS: 2,
      maxQPS: 60,
      stepSize: 6,
      stepDuration: 35,
      testType: 'weighted_mixed',
      outputPath: './test-results/mixed-apis',
    },
  ];

  const results: any[] = [];
  const totalEstimatedTime = testScenarios.reduce((total, config) => {
    const steps = Math.ceil((config.maxQPS - config.startQPS) / config.stepSize) + 1;
    return total + (steps * config.stepDuration);
  }, 0);

  console.log(chalk.yellow(`Estimated total time: ${(totalEstimatedTime / 60).toFixed(1)} minutes`));
  console.log('Press Ctrl+C to cancel, or wait 5 seconds to continue...\n');
  await new Promise(resolve => setTimeout(resolve, 5000));

  for (let i = 0; i < testScenarios.length; i++) {
    const config = testScenarios[i];
    console.log(chalk.blue(`\n=== Running Test ${i + 1}/${testScenarios.length}: ${config.testName} ===`));
    
    try {
      const engine = new QPSRampEngine();
      
      engine.on('stepCompleted', (stepResult) => {
        const status = stepResult.errorRate < 5 ? '✅' : stepResult.errorRate < 20 ? '⚠️' : '❌';
        console.log(
          `  ${status} QPS ${stepResult.qps}: ` +
          `${stepResult.successfulRequests}/${stepResult.totalRequests} ` +
          `(${stepResult.errorRate.toFixed(1)}% errors, ` +
          `${stepResult.averageResponseTime.toFixed(0)}ms avg)`
        );
      });

      const result = await engine.startQPSRampTest(config);
      results.push(result);

      console.log(chalk.green(`✅ ${config.testName} completed`));
      console.log(`   Max Sustainable QPS: ${result.maxSustainableQPS}`);
      if (result.breakpointQPS) {
        console.log(`   Breakpoint QPS: ${result.breakpointQPS}`);
      }

    } catch (error) {
      console.error(chalk.red(`❌ ${config.testName} failed:`), error);
      results.push({ error: error instanceof Error ? error.message : String(error) });
    }

    // Brief pause between tests
    if (i < testScenarios.length - 1) {
      console.log(chalk.gray('  Pausing 30 seconds before next test...'));
      await new Promise(resolve => setTimeout(resolve, 30000));
    }
  }

  // Generate comprehensive summary
  console.log(chalk.blue.bold('\n🎯 Comprehensive Test Summary'));
  console.log('='.repeat(80));

  const apiCapacities: { [key: string]: number } = {};
  
  results.forEach((result, index) => {
    if (result.error) {
      console.log(chalk.red(`${testScenarios[index].testName}: Test failed`));
      return;
    }

    const config = testScenarios[index];
    apiCapacities[config.testType] = result.maxSustainableQPS;

    console.log(chalk.green(`${config.testName}:`));
    console.log(`   Max Sustainable QPS: ${result.maxSustainableQPS}`);
    console.log(`   Recommended Safe QPS: ${Math.floor(result.maxSustainableQPS * 0.7)}`);
    console.log(`   Total Requests: ${result.summary.totalRequests.toLocaleString()}`);
    console.log(`   Success Rate: ${result.summary.overallSuccessRate.toFixed(1)}%\n`);
  });

  // System capacity recommendations
  if (Object.keys(apiCapacities).length > 0) {
    console.log(chalk.blue.bold('🚀 Production Recommendations'));
    console.log('='.repeat(80));

    Object.entries(apiCapacities).forEach(([apiType, maxQPS]) => {
      const safeQPS = Math.floor(maxQPS * 0.7);
      const warningQPS = Math.floor(maxQPS * 0.85);

      console.log(chalk.cyan(`${apiType}:`));
      console.log(`  Safe Production QPS: ${safeQPS}`);
      console.log(`  Warning Threshold: ${warningQPS}`);
      console.log(`  Critical Threshold: ${maxQPS}`);
      console.log();
    });

    const overallMaxQPS = Math.min(...Object.values(apiCapacities));
    console.log(chalk.yellow.bold('Overall System Capacity:'));
    console.log(`  Recommended maximum mixed load: ${Math.floor(overallMaxQPS * 0.6)} QPS`);
    console.log(`  This ensures stability across all API endpoints.\n`);
  }

  console.log(chalk.green.bold('✅ Comprehensive test suite completed!'));
  console.log('All detailed results saved to ./test-results/ directories');
}

// CLI setup
program
  .name('omni-test')
  .description('🎯 Unified Omni Executor Test Suite');

// Standard stress test
program
  .command('stress')
  .description('🔥 Run standard stress test')
  .option('-n, --name <string>', 'Test name')
  .option('-u, --url <string>', 'Target RPC URL', DEFAULT_URL)
  .option('-d, --duration <number>', 'Test duration in seconds', '300')
  .option('-c, --concurrency <number>', 'Concurrent connections', '10')
  .option('--test-type <string>', 'Test type: omni_userLogin, omni_getShieldingKey, omni_addWallet', 'omni_userLogin')
  .option('-o, --output <path>', 'Output directory', './test-results')
  .action(runStandardStressTest);

// QPS ramp test
program
  .command('qps')
  .description('📈 Run QPS ramp test')
  .option('-n, --name <string>', 'Test name')
  .option('-u, --url <string>', 'Target RPC URL', DEFAULT_URL)
  .option('--start-qps <number>', 'Starting QPS', '1')
  .option('--max-qps <number>', 'Maximum QPS', '50')
  .option('--step-size <number>', 'QPS increase per step', '5')
  .option('--step-duration <number>', 'Duration for each step (seconds)', '30')
  .option('--test-type <string>', 'Test type: omni_userLogin, omni_getShieldingKey, omni_addWallet, mixed, weighted_mixed', 'omni_userLogin')
  .option('-o, --output <path>', 'Output directory', './test-results')
  .action(runQPSRampTest);

// Comprehensive test
program
  .command('comprehensive')
  .alias('all')
  .description('🌟 Run comprehensive test suite (all APIs with QPS analysis)')
  .option('-u, --url <string>', 'Target RPC URL', DEFAULT_URL)
  .action(runComprehensiveTest);

// Quick test commands
program
  .command('quick-login')
  .description('⚡ Quick QPS test for UserLogin (1-20 QPS)')
  .option('-u, --url <string>', 'Target RPC URL', DEFAULT_URL)
  .action((options) => {
    runQPSRampTest({
      name: 'Quick UserLogin QPS Test',
      url: options.url,
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
  .option('-u, --url <string>', 'Target RPC URL', DEFAULT_URL)
  .action((options) => {
    runQPSRampTest({
      name: 'Quick ShieldingKey QPS Test',
      url: options.url,
      startQps: '1',
      maxQps: '30',
      stepSize: '5',
      stepDuration: '20',
      testType: 'omni_getShieldingKey',
    });
  });

// Example and help
program
  .command('examples')
  .description('📖 Show usage examples')
  .action(() => {
    console.log(chalk.blue.bold('🌟 Omni Executor Test Examples\n'));
    
    console.log(chalk.green('# Standard stress test (5 minutes, 10 concurrent)'));
    console.log('npm run test stress\n');
    
    console.log(chalk.green('# QPS ramp test (1-50 QPS)'));
    console.log('npm run test qps\n');
    
    console.log(chalk.green('# Comprehensive test suite (all APIs)'));
    console.log('npm run test comprehensive\n');
    
    console.log(chalk.green('# Quick tests'));
    console.log('npm run test quick-login');
    console.log('npm run test quick-shielding\n');
    
    console.log(chalk.green('# Custom QPS test'));
    console.log('npm run test qps --start-qps 2 --max-qps 100 --step-size 10\n');
    
    console.log(chalk.blue('Environment Variables:'));
    console.log('  OMNI_RPC_URL - Target endpoint (default: http://localhost:2100/)\n');
    
    console.log(chalk.yellow('💡 Tips:'));
    console.log('- Use "comprehensive" for complete system analysis');
    console.log('- Use "qps" for capacity planning');
    console.log('- Use "stress" for sustained load testing');
  });

// Parse CLI arguments
if (import.meta.url === `file://${process.argv[1]}`) {
  program.parse();
}