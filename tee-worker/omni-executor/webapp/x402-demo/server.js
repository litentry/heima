import express from 'express';
import cors from 'cors';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { readFileSync } from 'fs';
import dotenv from 'dotenv';
import { createPublicClient, http, encodeFunctionData, concat } from 'viem';
import { arbitrumSepolia } from 'viem/chains';

// Load environment variables from .env file
dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const app = express();
const PORT = process.env.PORT || 3001;
const OMNI_EXECUTOR_URL = process.env.OMNI_EXECUTOR_URL || 'http://localhost:8080';
const PAYMENT_AMOUNT_ETH = process.env.PAYMENT_AMOUNT_ETH || '0.00001';
const SERVER_WALLET_ADDRESS = process.env.SERVER_WALLET_ADDRESS || '0x0000000000000000000000000000000000000000';
const CHAIN_ID = process.env.CHAIN_ID || '421614'; // Arbitrum Sepolia
const FACTORY_ADDRESS = process.env.FACTORY_ADDRESS;
const ENTRYPOINT_ADDRESS = process.env.ENTRYPOINT_ADDRESS;
const PAYMASTER_ADDRESS = process.env.PAYMASTER_ADDRESS || '0x0000000000000000000000000000000000000000';
const USE_PAYMASTER = PAYMASTER_ADDRESS !== '0x0000000000000000000000000000000000000000' && PAYMASTER_ADDRESS !== '';
const CLIENT_ID = process.env.CLIENT_ID || 'wildmeta';

// Load ABIs
const FACTORY_ABI = JSON.parse(readFileSync(join(__dirname, 'public/abis/OmniAccountFactory.json'), 'utf-8')).abi;
const ACCOUNT_ABI = JSON.parse(readFileSync(join(__dirname, 'public/abis/OmniAccount.json'), 'utf-8')).abi;

// Initialize viem client
const publicClient = createPublicClient({
    chain: arbitrumSepolia,
    transport: http(process.env.RPC_URL || 'https://sepolia-rollup.arbitrum.io/rpc')
});

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.static(join(__dirname, 'public')));

/**
 * Main endpoint - serves the HTML page
 */
app.get('/', (req, res) => {
    res.sendFile(join(__dirname, 'public', 'index.html'));
});

/**
 * X402 protected endpoint
 * Returns 402 Payment Required if no valid payment is provided
 */
app.get('/x402', async (req, res) => {
    const paymentHeader = req.headers['x-payment'];

    // If no payment header, return 402 with payment requirements
    if (!paymentHeader) {
        return res.status(402).json({
            error: 'Payment Required',
            payment: {
                amount: PAYMENT_AMOUNT_ETH,
                currency: 'ETH',
                chainId: CHAIN_ID,
                recipient: SERVER_WALLET_ADDRESS,
                network: 'arbitrum-sepolia'
            }
        });
    }

    try {
        // Decode the payment (UserOp) from header
        const userOp = JSON.parse(Buffer.from(paymentHeader, 'base64').toString('utf-8'));

        console.log('Received payment UserOp:', userOp);

        // Step 1: Verify the UserOp with omni-executor (JSON-RPC call)
        const verifyResponse = await fetch(OMNI_EXECUTOR_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                method: 'omni_verifyUserOp',
                params: {
                    user_operation: userOp,
                    chain_id: parseInt(CHAIN_ID),
                },
                id: 1,
            }),
        });

        if (!verifyResponse.ok) {
            console.error('Verification request failed:', verifyResponse.status);
            return res.status(400).json({
                error: 'Payment verification request failed',
                details: `HTTP ${verifyResponse.status}`
            });
        }

        const verifyResult = await verifyResponse.json();

        // Check for JSON-RPC error
        if (verifyResult.error) {
            console.error('Verification failed:', verifyResult.error);
            return res.status(400).json({
                error: 'Payment verification failed',
                details: verifyResult.error
            });
        }

        // Check if UserOp is valid
        if (!verifyResult.result || !verifyResult.result.valid) {
            console.error('UserOp is invalid:', verifyResult.result);
            return res.status(400).json({
                error: 'Payment verification failed',
                details: verifyResult.result?.message || 'UserOp is invalid'
            });
        }

        console.log('Verification successful:', verifyResult.result);

        // Step 2: Settle the UserOp (actually submit to chain) - JSON-RPC call
        const settleResponse = await fetch(OMNI_EXECUTOR_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                method: 'omni_settleUserOp',
                params: {
                    user_operation: userOp,
                    chain_id: parseInt(CHAIN_ID),
                },
                id: 2,
            }),
        });

        if (!settleResponse.ok) {
            console.error('Settlement request failed:', settleResponse.status);
            return res.status(500).json({
                error: 'Payment settlement request failed',
                details: `HTTP ${settleResponse.status}`
            });
        }

        const settleResult = await settleResponse.json();

        // Check for JSON-RPC error
        if (settleResult.error) {
            console.error('Settlement failed:', settleResult.error);
            return res.status(500).json({
                error: 'Payment settlement failed',
                details: settleResult.error
            });
        }

        console.log('Settlement successful:', settleResult.result);

        // Step 3: Return protected content with transaction hash
        return res.json({
            message: 'Payment successful! Here is your protected content.',
            transactionHash: settleResult.result.transaction_hash,
            content: {
                data: 'This is the x402-protected content!',
                timestamp: new Date().toISOString(),
                paidAmount: PAYMENT_AMOUNT_ETH,
                chainId: CHAIN_ID
            }
        });

    } catch (error) {
        console.error('Error processing payment:', error);
        return res.status(500).json({
            error: 'Internal server error',
            details: error.message
        });
    }
});

/**
 * Helper: Get root signer from TEE worker
 */
async function getRootSignerFromTEE(omniAccountHash) {
    const response = await fetch(OMNI_EXECUTOR_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'omni_getSmartWalletRootSigner',
            params: {
                omni_account: omniAccountHash,
                chain_type: 'evm',
                wallet_index: 0,
            },
            id: 1,
        }),
    });

    const result = await response.json();
    if (result.error) {
        throw new Error(`TEE worker error: ${JSON.stringify(result.error)}`);
    }
    return result.result;
}

/**
 * Helper: Convert string to hex bytes
 */
function stringToHex(str) {
    return '0x' + Buffer.from(str, 'utf-8').toString('hex');
}

/**
 * Helper: Pack gas limits
 */
function packAccountGasLimits(callGasLimit, verificationGasLimit) {
    const packed = (BigInt(verificationGasLimit) << 128n) | BigInt(callGasLimit);
    return '0x' + packed.toString(16).padStart(64, '0');
}

/**
 * Helper: Pack gas fees
 */
function packGasFees(maxFeePerGas, maxPriorityFeePerGas) {
    const packed = (BigInt(maxPriorityFeePerGas) << 128n) | BigInt(maxFeePerGas);
    return '0x' + packed.toString(16).padStart(64, '0');
}

/**
 * Endpoint: Get counterfactual address
 */
app.post('/api/getCounterfactualAddress', async (req, res) => {
    try {
        const { omniAccountHash, walletAddress } = req.body;

        if (!omniAccountHash || !walletAddress) {
            return res.status(400).json({ error: 'Missing required parameters' });
        }

        // Get root signer from TEE
        const rootSigner = await getRootSignerFromTEE(omniAccountHash);

        // Convert clientId to bytes
        const clientIdBytes = stringToHex(CLIENT_ID);

        // Call factory.getAddress()
        const address = await publicClient.readContract({
            address: FACTORY_ADDRESS,
            abi: FACTORY_ABI,
            functionName: 'getAddress',
            args: [omniAccountHash, 6, clientIdBytes, rootSigner] // 6 = OwnerType.Evm
        });

        res.json({ address, rootSigner });
    } catch (error) {
        console.error('Error getting counterfactual address:', error);
        res.status(500).json({ error: error.message });
    }
});

/**
 * Endpoint: Build UserOperation
 */
app.post('/api/buildUserOp', async (req, res) => {
    try {
        const { smartWalletAddress, recipient, amount, omniAccountHash } = req.body;

        if (!smartWalletAddress || !recipient || !amount || !omniAccountHash) {
            return res.status(400).json({ error: 'Missing required parameters' });
        }

        // Check if smart wallet is deployed
        const code = await publicClient.getBytecode({ address: smartWalletAddress });
        const isDeployed = code && code !== '0x' && code !== '0x0';

        // Check balance
        const balance = await publicClient.getBalance({ address: smartWalletAddress });
        console.log(`Smart wallet balance: ${balance} wei (${Number(balance) / 1e18} ETH)`);

        // Get nonce
        let nonce = 0n;
        if (isDeployed) {
            nonce = await publicClient.readContract({
                address: smartWalletAddress,
                abi: ACCOUNT_ABI,
                functionName: 'getNonce'
            });
        }

        // Check if we need to warn about insufficient balance
        if (balance === 0n && !isDeployed) {
            console.warn('WARNING: Smart wallet has 0 balance and is not deployed. This UserOp will likely fail without a paymaster.');
        }

        // Build callData
        const callData = encodeFunctionData({
            abi: ACCOUNT_ABI,
            functionName: 'execute',
            args: [recipient, BigInt(amount), '0x']
        });

        // Generate initCode if not deployed
        let initCode = '0x';
        if (!isDeployed) {
            const rootSigner = await getRootSignerFromTEE(omniAccountHash);
            const clientIdBytes = stringToHex(CLIENT_ID);
            const createAccountCalldata = encodeFunctionData({
                abi: FACTORY_ABI,
                functionName: 'createAccount',
                args: [omniAccountHash, 6, clientIdBytes, rootSigner]
            });
            initCode = concat([FACTORY_ADDRESS, createAccountCalldata]);
        }

        // Build paymaster data if using paymaster
        let paymasterAndData = '0x';
        if (USE_PAYMASTER) {
            // For EntryPoint v0.7 with packed UserOp format, paymasterAndData includes:
            // paymaster address (20 bytes) + verificationGasLimit (16 bytes/128 bits) + postOpGasLimit (16 bytes/128 bits) + paymasterData (variable)
            const paymasterVerificationGasLimit = 300000n;
            const paymasterPostOpGasLimit = 50000n;

            // Convert to hex, each padded to 16 bytes (32 hex chars) as uint128
            const verificationGasHex = paymasterVerificationGasLimit.toString(16).padStart(32, '0'); // 16 bytes = 32 hex chars
            const postOpGasHex = paymasterPostOpGasLimit.toString(16).padStart(32, '0'); // 16 bytes = 32 hex chars

            paymasterAndData = PAYMASTER_ADDRESS.toLowerCase().replace('0x', '') + verificationGasHex + postOpGasHex;
            paymasterAndData = '0x' + paymasterAndData;

            console.log('Using paymaster:', PAYMASTER_ADDRESS);
            console.log('Paymaster data:', paymasterAndData);
        } else if (balance === 0n && !isDeployed) {
            return res.status(400).json({
                error: 'Smart wallet has insufficient balance and no paymaster configured. Please send ETH to the smart wallet address first.',
                smartWalletAddress,
                balance: '0'
            });
        }

        // Use reasonable gas values for Arbitrum Sepolia
        // Higher limits for deployment, lower for regular operations
        const callGasLimit = 800000n;
        const verificationGasLimit = isDeployed ? 500000n : 1000000n;
        const preVerificationGas = isDeployed ? 50000n : 100000n;

        // Get current gas prices
        // const feeData = await publicClient.estimateFeesPerGas();
        // const maxFeePerGas = feeData.maxFeePerGas || 100000000n; // Default to 0.1 gwei
        // const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || 100000000n; // Default to 0.1 gwei
        const maxFeePerGas = 300000000n;
        const maxPriorityFeePerGas = 100000000n;

        console.log('Gas parameters:', {
            callGasLimit: callGasLimit.toString(),
            verificationGasLimit: verificationGasLimit.toString(),
            preVerificationGas: preVerificationGas.toString(),
            maxFeePerGas: maxFeePerGas.toString(),
            maxPriorityFeePerGas: maxPriorityFeePerGas.toString(),
            isDeployed
        });

        const userOp = {
            sender: smartWalletAddress,
            nonce: Number(nonce),
            init_code: initCode,
            call_data: callData,
            account_gas_limits: packAccountGasLimits(callGasLimit, verificationGasLimit),
            pre_verification_gas: Number(preVerificationGas),
            gas_fees: packGasFees(maxFeePerGas, maxPriorityFeePerGas),
            paymaster_and_data: paymasterAndData,
        };

        console.log('Built UserOp:', userOp);
        res.json({ userOp, isDeployed, balance: balance.toString(), usingPaymaster: USE_PAYMASTER });
    } catch (error) {
        console.error('Error building UserOp:', error);
        res.status(500).json({ error: error.message });
    }
});

/**
 * Configuration endpoint for client
 */
app.get('/config', (req, res) => {
    res.json({
        chainId: parseInt(CHAIN_ID),
        rpcUrl: process.env.RPC_URL || 'https://sepolia-rollup.arbitrum.io/rpc',
        entrypointAddress: process.env.ENTRYPOINT_ADDRESS || '0x0000000071727De22E5E9d8BAf0edAc6f37da032',
        factoryAddress: process.env.FACTORY_ADDRESS || '',
        paymasterAddress: process.env.PAYMASTER_ADDRESS || '0x0000000000000000000000000000000000000000',
        clientId: process.env.CLIENT_ID || 'wildmeta',
        teeWorkerUrl: OMNI_EXECUTOR_URL,
    });
});

/**
 * Health check endpoint
 */
app.get('/health', (req, res) => {
    res.json({
        status: 'ok',
        omniExecutor: OMNI_EXECUTOR_URL,
        chainId: CHAIN_ID
    });
});

// Start server
app.listen(PORT, () => {
    console.log(`\n🚀 X402 Demo Server running on http://localhost:${PORT}`);
    console.log(`📡 Omni-Executor URL: ${OMNI_EXECUTOR_URL}`);
    console.log(`⛓️  Chain ID: ${CHAIN_ID} (Arbitrum Sepolia)`);
    console.log(`💰 Payment Amount: ${PAYMENT_AMOUNT_ETH} ETH`);
    console.log(`📬 Payment Recipient: ${SERVER_WALLET_ADDRESS}`);
    console.log(`💳 Paymaster: ${USE_PAYMASTER ? PAYMASTER_ADDRESS : 'Not configured (users must fund their wallets)'}`);
    console.log(`💳 Entrypoint: ${ENTRYPOINT_ADDRESS}\n`);
});
