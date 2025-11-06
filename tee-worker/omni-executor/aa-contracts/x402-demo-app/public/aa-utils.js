// Account Abstraction utilities for browser using viem
// Properly calls TEE worker for root signer and uses viem for contract interactions

// Configuration - will be loaded from server
let CONFIG = {
    CLIENT_ID: 'wildmeta',
    CHAIN_ID: 421614,
    ENTRYPOINT_ADDRESS: '0x0000000071727De22E5E9d8BAf0edAc6f37da032',
    FACTORY_ADDRESS: '',
    RPC_URL: 'https://sepolia-rollup.arbitrum.io/rpc',
    TEE_WORKER_URL: 'https://staging-dex-worker.heima.network'
};

// ABIs
let FACTORY_ABI = null;
let ACCOUNT_ABI = null;

// Viem clients
let publicClient = null;

// UserOpSigner enum - matches the contract enum
const UserOpSigner = {
    Owner: 0x00,
    RootKey: 0x01,
    SessionKey: 0x02,
    Passkey: 0x03,
};

// OwnerType enum - matches the contract enum
const OwnerType = {
    Evm: 6,  // 0x06 for EVM addresses
};

/**
 * Load configuration from server
 */
async function loadConfig() {
    try {
        const response = await fetch('/config');
        const config = await response.json();
        CONFIG = {
            CLIENT_ID: config.clientId,
            CHAIN_ID: config.chainId,
            ENTRYPOINT_ADDRESS: config.entrypointAddress,
            FACTORY_ADDRESS: config.factoryAddress,
            RPC_URL: config.rpcUrl,
            TEE_WORKER_URL: config.teeWorkerUrl || 'https://staging-dex-worker.heima.network'
        };
        console.log('Loaded config from server:', CONFIG);

        // Load ABIs
        const factoryAbiResponse = await fetch('/abis/OmniAccountFactory.json');
        const factoryAbiData = await factoryAbiResponse.json();
        FACTORY_ABI = factoryAbiData.abi;

        const accountAbiResponse = await fetch('/abis/OmniAccount.json');
        const accountAbiData = await accountAbiResponse.json();
        ACCOUNT_ABI = accountAbiData.abi;

        // Initialize viem public client
        const { createPublicClient, http } = window.viem;
        publicClient = createPublicClient({
            chain: {
                id: CONFIG.CHAIN_ID,
                name: 'Arbitrum Sepolia',
                network: 'arbitrum-sepolia',
                nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
                rpcUrls: {
                    default: { http: [CONFIG.RPC_URL] },
                    public: { http: [CONFIG.RPC_URL] },
                }
            },
            transport: http(CONFIG.RPC_URL)
        });

        console.log('Initialized viem client');
    } catch (error) {
        console.warn('Failed to load config from server, using defaults:', error);
    }
}

/**
 * Calculate OmniAccount hash from EVM address
 * Based on: sha256(clientId + "evm" + address)
 */
async function calculateOmniAccount(address, clientId = CONFIG.CLIENT_ID) {
    const encoder = new TextEncoder();

    // Prepare inputs in order: clientId, "evm", address
    const clientIdBytes = encoder.encode(clientId);
    const identityTypeBytes = encoder.encode('evm');

    // Convert address hex to bytes (remove 0x prefix)
    const addressHex = address.slice(2).toLowerCase();
    const addressBytes = new Uint8Array(20);
    for (let i = 0; i < addressHex.length; i += 2) {
        addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
    }

    // Combine all bytes
    const combined = new Uint8Array(
        clientIdBytes.length + identityTypeBytes.length + addressBytes.length
    );
    combined.set(clientIdBytes, 0);
    combined.set(identityTypeBytes, clientIdBytes.length);
    combined.set(addressBytes, clientIdBytes.length + identityTypeBytes.length);

    // Calculate SHA256 hash
    const hashBuffer = await crypto.subtle.digest('SHA-256', combined);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = '0x' + hashArray.map(b => b.toString(16).padStart(2, '0')).join('');

    console.log('OmniAccount calculation:', { address, clientId, hashHex });
    return hashHex;
}

/**
 * Get root signer from TEE worker
 */
async function getRootSignerFromTEE(omniAccountHash, chainType = 'evm', signerIndex = 0) {
    console.log('=== Getting Root Signer from TEE Worker ===');
    console.log('TEE Worker URL:', CONFIG.TEE_WORKER_URL);
    console.log('OmniAccount hash:', omniAccountHash);
    console.log('Chain type:', chainType);
    console.log('Signer index:', signerIndex);

    try {
        const response = await fetch(CONFIG.TEE_WORKER_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                method: 'omni_getSmartWalletRootSigner',
                params: {
                    omni_account: omniAccountHash,
                    chain_type: chainType,
                    wallet_index: signerIndex,
                },
                id: 1,
            }),
        });

        if (!response.ok) {
            throw new Error(`TEE worker request failed: HTTP ${response.status}`);
        }

        const result = await response.json();

        if (result.error) {
            throw new Error(`TEE worker error: ${JSON.stringify(result.error)}`);
        }

        const rootSigner = result.result;
        console.log('Root signer from TEE:', rootSigner);
        return rootSigner;
    } catch (error) {
        console.error('Error getting root signer from TEE:', error);
        throw new Error('Failed to get root signer from TEE worker: ' + error.message);
    }
}

/**
 * Get counterfactual smart wallet address using viem
 */
async function getCounterfactualAddress(omniAccountHash, walletAddress) {
    console.log('=== Getting Counterfactual Address ===');
    console.log('Factory address:', CONFIG.FACTORY_ADDRESS);
    console.log('Wallet address:', walletAddress);
    console.log('OmniAccount hash:', omniAccountHash);
    console.log('Client ID:', CONFIG.CLIENT_ID);

    try {
        // Validate inputs
        if (!CONFIG.FACTORY_ADDRESS || CONFIG.FACTORY_ADDRESS === '0x0000000000000000000000000000000000000000' || CONFIG.FACTORY_ADDRESS === '') {
            throw new Error('Factory address not configured. Please set FACTORY_ADDRESS in .env');
        }

        // Step 1: Get root signer from TEE worker
        const rootSigner = await getRootSignerFromTEE(omniAccountHash, 'evm', 0);

        // Step 2: Convert clientId to bytes
        const { stringToHex } = window.viem;
        const clientIdBytes = stringToHex(CONFIG.CLIENT_ID);

        // Step 3: Call factory.getAddress() using viem
        console.log('Calling factory.getAddress with params:', {
            oa: omniAccountHash,
            oaType: OwnerType.Evm,
            clientId: clientIdBytes,
            root: rootSigner
        });

        const address = await publicClient.readContract({
            address: CONFIG.FACTORY_ADDRESS,
            abi: FACTORY_ABI,
            functionName: 'getAddress',
            args: [omniAccountHash, OwnerType.Evm, clientIdBytes, rootSigner]
        });

        console.log('Counterfactual address from factory:', address);
        return address;
    } catch (error) {
        console.error('Error getting counterfactual address:', error);
        throw new Error('Failed to get counterfactual address: ' + error.message);
    }
}

/**
 * Get nonce from smart wallet contract
 */
async function getNonce(smartWalletAddress) {
    try {
        // Check if the smart wallet is deployed by checking code
        const code = await publicClient.getBytecode({
            address: smartWalletAddress
        });

        // If not deployed (code is undefined or '0x'), nonce is 0
        if (!code || code === '0x' || code === '0x0') {
            console.log('Smart wallet not deployed yet, using nonce 0');
            return 0n;
        }

        // Call getNonce() using viem
        const nonce = await publicClient.readContract({
            address: smartWalletAddress,
            abi: ACCOUNT_ABI,
            functionName: 'getNonce'
        });

        console.log('Fetched nonce from chain:', nonce.toString());
        return nonce;
    } catch (error) {
        console.error('Error fetching nonce:', error);
        // If there's an error (e.g., contract not deployed), return 0
        console.warn('Failed to fetch nonce, defaulting to 0');
        return 0n;
    }
}

/**
 * Generate initCode for deploying the smart wallet (if not yet deployed)
 */
async function generateInitCode(omniAccountHash, walletAddress) {
    try {
        // Get root signer from TEE
        const rootSigner = await getRootSignerFromTEE(omniAccountHash, 'evm', 0);

        // Convert clientId to bytes
        const { stringToHex, encodeFunctionData, concat } = window.viem;
        const clientIdBytes = stringToHex(CONFIG.CLIENT_ID);

        // Encode createAccount function call
        const createAccountCalldata = encodeFunctionData({
            abi: FACTORY_ABI,
            functionName: 'createAccount',
            args: [omniAccountHash, OwnerType.Evm, clientIdBytes, rootSigner]
        });

        // Combine factory address + createAccount calldata
        const initCode = concat([CONFIG.FACTORY_ADDRESS, createAccountCalldata]);

        console.log('Generated initCode:', initCode);
        return initCode;
    } catch (error) {
        console.error('Error generating initCode:', error);
        throw error;
    }
}

/**
 * Pack gas limits into accountGasLimits (bytes32)
 * Upper 128 bits: verificationGasLimit, Lower 128 bits: callGasLimit
 */
function packAccountGasLimits(callGasLimit, verificationGasLimit) {
    const packed = (verificationGasLimit << 128n) | callGasLimit;
    return '0x' + packed.toString(16).padStart(64, '0');
}

/**
 * Pack gas fees into gasFees (bytes32)
 * Upper 128 bits: maxPriorityFeePerGas, Lower 128 bits: maxFeePerGas
 */
function packGasFees(maxFeePerGas, maxPriorityFeePerGas) {
    const packed = (maxPriorityFeePerGas << 128n) | maxFeePerGas;
    return '0x' + packed.toString(16).padStart(64, '0');
}

/**
 * Build a UserOperation for native ETH transfer
 */
async function buildPaymentUserOp(params) {
    const {
        smartWalletAddress,
        recipient,
        amount, // in wei
        nonce = 0n,
        omniAccountHash,
        walletAddress,
    } = params;

    // Check if smart wallet is deployed
    const code = await publicClient.getBytecode({
        address: smartWalletAddress
    });
    const isDeployed = code && code !== '0x' && code !== '0x0';

    // Build callData for execute(recipient, amount, "0x")
    const { encodeFunctionData } = window.viem;
    const callData = encodeFunctionData({
        abi: ACCOUNT_ABI,
        functionName: 'execute',
        args: [recipient, amount, '0x']
    });

    // Generate initCode if not deployed
    let initCode = '0x';
    if (!isDeployed) {
        console.log('Smart wallet not deployed, generating initCode...');
        initCode = await generateInitCode(omniAccountHash, walletAddress);
    }

    // Use reasonable gas values for Arbitrum Sepolia
    // Higher limits for deployment, lower for regular operations
    const callGasLimit = 800000n;
    const verificationGasLimit = isDeployed ? 500000n : 1000000n;
    const preVerificationGas = isDeployed ? 50000n : 100000n;
    const maxFeePerGas = 100000000n; // 0.1 gwei
    const maxPriorityFeePerGas = 100000000n; // 0.1 gwei

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
        paymaster_and_data: '0x',
        signature: undefined, // Will be filled after signing
    };

    console.log('Built UserOp:', userOp);
    return userOp;
}

/**
 * Sign UserOperation with MetaMask using EIP-712
 */
async function signUserOperation(userOp, walletAddress, chainId) {
    if (!window.ethereum) {
        throw new Error('MetaMask not found');
    }

    // EIP-712 domain
    const domain = {
        name: 'ERC4337',
        version: '1',
        chainId: chainId,
        verifyingContract: CONFIG.ENTRYPOINT_ADDRESS,
    };

    // EIP-712 types for PackedUserOperation
    const types = {
        PackedUserOperation: [
            { name: 'sender', type: 'address' },
            { name: 'nonce', type: 'uint256' },
            { name: 'initCode', type: 'bytes' },
            { name: 'callData', type: 'bytes' },
            { name: 'accountGasLimits', type: 'bytes32' },
            { name: 'preVerificationGas', type: 'uint256' },
            { name: 'gasFees', type: 'bytes32' },
            { name: 'paymasterAndData', type: 'bytes' },
        ],
    };

    // Message to sign
    const message = {
        sender: userOp.sender,
        nonce: userOp.nonce,
        initCode: userOp.init_code,
        callData: userOp.call_data,
        accountGasLimits: userOp.account_gas_limits,
        preVerificationGas: userOp.pre_verification_gas,
        gasFees: userOp.gas_fees,
        paymasterAndData: userOp.paymaster_and_data,
    };

    try {
        // Request signature via MetaMask EIP-712
        const signature = await window.ethereum.request({
            method: 'eth_signTypedData_v4',
            params: [walletAddress, JSON.stringify({ domain, types, primaryType: 'PackedUserOperation', message })],
        });

        // Add UserOpSigner.Owner prefix (0x00)
        const prefixedSignature = '0x00' + signature.slice(2);
        return prefixedSignature;
    } catch (error) {
        console.error('Signing failed:', error);
        throw error;
    }
}
