// Account Abstraction utilities for browser using viem
// Properly calls TEE worker for root signer and uses viem for contract interactions

const { keccak256, encodeAbiParameters, parseAbiParameters, toHex, hashTypedData } = window.viem;


/**
 * Calculate OmniAccount hash from EVM address
 * Based on: sha256(clientId + "evm" + address)
 */
export async function calculateOmniAccount(address, clientId = CONFIG.CLIENT_ID) {
    try {
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

        console.log('  ✓ OmniAccount hash:', hashHex);
        return hashHex;
    } catch (error) {
        console.error('❌ Failed to calculate OmniAccount:', error);
        throw error;
    }
}

/**
 * Get root signer from TEE worker
 */
export async function getRootSignerFromTEE(teeWorkerUrl, omniAccountHash, chainType = 'evm', signerIndex = 0) {
    try {
        const response = await fetch(teeWorkerUrl, {
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
            console.error(`❌ TEE worker request failed: HTTP ${response.status}`);
            throw new Error(`TEE worker request failed: HTTP ${response.status}`);
        }

        const result = await response.json();

        if (result.error) {
            console.error('❌ TEE worker returned error:', result.error);
            throw new Error(`TEE worker error: ${JSON.stringify(result.error)}`);
        }

        const rootSigner = result.result;
        console.log('  ✓ Root signer:', rootSigner);
        return rootSigner;
    } catch (error) {
        console.error('❌ Failed to get root signer from TEE:', error);
        throw new Error('Failed to get root signer from TEE worker: ' + error.message);
    }
}

/**
 * Get counterfactual smart wallet address using viem
 */
export async function getCounterfactualAddress(publicClient, factoryAddress, factoryAbi, omniAccountHash, ownerType, clientIdBytes, rootSigner) {
    try {
        // Validate inputs
        if (!factoryAddress || factoryAddress === '0x0000000000000000000000000000000000000000' || factoryAddress === '') {
            throw new Error('Factory address not configured. Please set FACTORY_ADDRESS in .env');
        }

        const address = await publicClient.readContract({
            address: factoryAddress,
            abi: factoryAbi,
            functionName: 'getAddress',
            args: [omniAccountHash, ownerType, clientIdBytes, rootSigner]
        });

        console.log('  ✓ Counterfactual address:', address);
        return address;
    } catch (error) {
        console.error('❌ Failed to get counterfactual address:', error);
        throw new Error('Failed to get counterfactual address: ' + error.message);
    }
}

/**
 * Get nonce from smart wallet contract
 */
async function getNonce(publicClient, accountAbi, smartWalletAddress) {
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
            abi: accountAbi,
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
async function generateInitCode(factoryAddress, factoryAbi, omniAccountHash, ownerType, clientIdBytes, rootSigner) {
    try {
        // Encode createAccount function call
        const createAccountCalldata = encodeFunctionData({
            abi: factoryAbi,
            functionName: 'createAccount',
            args: [omniAccountHash, ownerType, clientIdBytes, rootSigner]
        });

        // Combine factory address + createAccount calldata
        const initCode = concat([factoryAddress, createAccountCalldata]);

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



async function getUserOpHash(publicClient, entrypointAddress, userOp) {
    // Convert userOp fields to camelCase for consistency with the Rust example and ABI encoding
    const transformedUserOp = {
        sender: userOp.sender,
        nonce: userOp.nonce,
        initCode: userOp.init_code,
        callData: userOp.call_data,
        accountGasLimits: userOp.account_gas_limits,
        preVerificationGas: userOp.pre_verification_gas,
        gasFees: userOp.gas_fees,
        paymasterAndData: userOp.paymaster_and_data,
    };

    // 1. Define the PackedUserOperation TypeHash
    const packedUserOpTypeHash = keccak256(
        toHex(
            new TextEncoder().encode(
                "PackedUserOperation(address sender,uint256 nonce,bytes initCode,bytes callData,bytes32 accountGasLimits,uint256 preVerificationGas,bytes32 gasFees,bytes paymasterAndData)"
            )
        )
    );

    // 2. Hash dynamic fields
    const initCodeHash = keccak256(transformedUserOp.initCode);
    const callDataHash = keccak256(transformedUserOp.callData);
    const paymasterAndDataHash = keccak256(transformedUserOp.paymasterAndData);

    // 3. Create a struct for hashing
    const userOpForHashing = {
        typeHash: packedUserOpTypeHash,
        sender: transformedUserOp.sender,
        nonce: transformedUserOp.nonce,
        initCode: initCodeHash,
        callData: callDataHash,
        accountGasLimits: transformedUserOp.accountGasLimits,
        preVerificationGas: transformedUserOp.preVerificationGas,
        gasFees: transformedUserOp.gasFees,
        paymasterAndData: paymasterAndDataHash,
    };

    // 4. ABI-encode the struct (hash(userOp))
    const encodedUserOp = encodeAbiParameters(
        parseAbiParameters([
            'bytes32 typeHash',
            'address sender',
            'uint256 nonce',
            'bytes32 initCode',
            'bytes32 callData',
            'bytes32 accountGasLimits',
            'uint256 preVerificationGas',
            'bytes32 gasFees',
            'bytes32 paymasterAndData',
        ]),
        [
            userOpForHashing.typeHash,
            userOpForHashing.sender,
            userOpForHashing.nonce,
            userOpForHashing.initCode,
            userOpForHashing.callData,
            userOpForHashing.accountGasLimits,
            userOpForHashing.preVerificationGas,
            userOpForHashing.gasFees,
            userOpForHashing.paymasterAndData,
        ]
    );

    const userOpStructHash = keccak256(encodedUserOp);

    return userOpStructHash;
}


/**
 * Sign UserOperation with MetaMask using EIP-712
 *
 * IMPORTANT: Uses viem's walletClient.signTypedData() instead of calling MetaMask's RPC directly.
 * This ensures proper type serialization and consistency between hash calculation and signing.
 *
 * @param publicClient - viem public client
 * @param entrypointAddress - EntryPoint contract address
 * @param userOpSigner - UserOpSigner enum (Owner, RootKey, etc.)
 * @param userOp - User operation to sign
 * @param walletAddress - Wallet address for signing
 * @returns Signature with UserOpSigner prefix (0x00 for Owner)
 */
export async function signUserOperation(publicClient, entrypointAddress, userOpSigner, userOp, walletAddress) {
    if (!window.ethereum) {
        throw new Error('MetaMask not found');
    }

    // EIP-712 typed data for PackedUserOperation (ERC-4337 v0.7)
    // Must match the EntryPoint contract's PACKED_USEROP_TYPEHASH
    const typedData = {
        domain: {
            name: 'ERC4337',
            version: '1',
            chainId: publicClient.chain.id,
            verifyingContract: entrypointAddress,
        },
        types: {
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
        },
        primaryType: 'PackedUserOperation',
        message: {
            sender: userOp.sender,
            nonce: userOp.nonce,
            initCode: userOp.init_code,
            callData: userOp.call_data,
            accountGasLimits: userOp.account_gas_limits,
            preVerificationGas: userOp.pre_verification_gas,
            gasFees: userOp.gas_fees,
            paymasterAndData: userOp.paymaster_and_data,
        }
    };

    // Calculate the EIP-712 digest for debugging
    const eip712Digest = hashTypedData(typedData);
    console.log('=== EIP-712 Signing ===');
    console.log('UserOp hash (EIP-712):', eip712Digest);
    console.log('Domain:', { name: typedData.domain.name, version: typedData.domain.version, chainId: typedData.domain.chainId });
    console.log('Sender:', typedData.message.sender);
    console.log('Nonce:', typedData.message.nonce);
    console.log('======================');

    try {
        // Use viem's walletClient.signTypedData for proper EIP-712 signing
        // This ensures type serialization consistency between hash calculation and signing
        const { createWalletClient, custom } = window.viem;

        const walletClient = createWalletClient({
            account: walletAddress,
            chain: publicClient.chain,
            transport: custom(window.ethereum),
        });

        console.log('  📝 Requesting signature from MetaMask...');
        const signature = await walletClient.signTypedData({
            account: walletAddress,
            domain: typedData.domain,
            types: typedData.types,
            primaryType: typedData.primaryType,
            message: typedData.message,
        });
        console.log('  ✓ Signature received');

        // Add UserOpSigner prefix (0x00 for Owner, 0x01 for RootKey, etc.)
        const prefixedSignature = `0x${userOpSigner.Owner.toString(16).padStart(2, '0')}${signature.slice(2)}`;
        console.log('  ✓ Added UserOpSigner.Owner prefix (0x00)');
        return prefixedSignature;
    } catch (error) {
        console.error('❌ Signing failed:', error);
        if (error.code === 4001) {
            throw new Error('User rejected signature request in MetaMask');
        }
        throw error;
    }
} 
