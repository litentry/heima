#!/usr/bin/env node

import 'dotenv/config';
import { JsonRpcClient } from './utils/json-rpc-client';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { sha256 } from 'js-sha256';
import chalk from 'chalk';

const ClientId = {
  Wildmeta: 'wildmeta',
  Heima: 'heima',
  Console: 'console'
};

// Calculate omni account
function calculateOmniAccount(evmAddress: string, clientId: string): string {
  const inputs: Uint8Array[] = [];
  
  // Client ID as raw bytes
  const clientIdBytes = new TextEncoder().encode(clientId);
  inputs.push(clientIdBytes);
  
  // Identity type ("evm")
  inputs.push(new TextEncoder().encode('evm'));
  
  // EVM address bytes (remove 0x prefix)
  const addressHex = evmAddress.slice(2).toLowerCase();
  const addressBytes = new Uint8Array(20);
  for (let i = 0; i < addressHex.length; i += 2) {
    addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
  }
  inputs.push(addressBytes);
  
  // Combine all inputs
  const totalLength = inputs.reduce((sum, arr) => sum + arr.length, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const input of inputs) {
    combined.set(input, offset);
    offset += input.length;
  }
  
  // Calculate SHA256 hash using js-sha256
  const hash = sha256.array(combined);
  return `0x${hash.map((b: number) => b.toString(16).padStart(2, '0')).join('')}`;
}

async function testAPIs() {
  const targetUrl = process.env.OMNI_RPC_URL || 'https://staging-dex-worker.heima.network';
  console.log(chalk.blue(`🎯 Test Target: ${targetUrl}`));
  
  const client = new JsonRpcClient(targetUrl);
  
  // Generate test wallet
  const privateKey = generatePrivateKey();
  const account = privateKeyToAccount(privateKey);
  const omniAccount = calculateOmniAccount(account.address, ClientId.Wildmeta);
  
  console.log(chalk.green(`💰 Test Wallet:`));
  console.log(`   EVM Address: ${account.address}`);
  console.log(`   Omni Account: ${omniAccount}`);
  
  try {
    // Test omni_getNextIntentId
    console.log(chalk.blue(`\n📋 Testing omni_getNextIntentId...`));
    const intentId = await client.call('omni_getNextIntentId', {
      omni_account: omniAccount
    });
    console.log(chalk.green(`✅ Result: ${intentId}`));
    
    // Test omni_getWeb3SignInMessage  
    console.log(chalk.blue(`\n🔐 Testing omni_getWeb3SignInMessage...`));
    const signInMsg = await client.call('omni_getWeb3SignInMessage', {
      client_id: ClientId.Wildmeta,
      omni_account: omniAccount
    });
    console.log(chalk.green(`✅ Result:`));
    console.log(JSON.stringify(signInMsg, null, 2));
    
    console.log(chalk.green(`\n🎉 All API tests passed!`));
    
  } catch (error) {
    console.error(chalk.red(`❌ Test failed:`), error);
    process.exit(1);
  }
}

// Run tests
testAPIs();