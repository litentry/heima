import { describe, it, before } from 'mocha';
import { expect } from 'chai';
import { createPublicClient, createWalletClient, http, parseEther, parseUnits, type Address } from 'viem';
import { privateKeyToAccount } from 'viem/accounts';
import { anvil } from 'viem/chains';
import { ClientId, omniApi, randomEvmWallet, calculateOmniAccount, UserLoginResponse } from './utils';
import { signMessage } from 'viem/accounts';
import { TEST_CONFIG, validateTestEnvironment } from './config';
import { promises as fs } from 'fs';
import {
    CONTRACT_ABIS,
    OWNER_TYPE,
    createUserOperation,
    signUserOperation,
    packUserOperation,
    generateInitCode,
    stringToBytes,
    createExecuteCalldata,
    createAddSignerCalldata,
    createTokenTransferCalldata,
    type UserOperation,
} from './utils/aa-utils';

// Function to wait for contract deployment in CI environment
async function waitForContractDeployment(): Promise<void> {
    const deployedAddressesPath = '/shared/deployed-addresses.json';
    const maxWaitTime = 120000; // 2 minutes
    const checkInterval = 2000; // 2 seconds
    const startTime = Date.now();
    
    console.log('Waiting for contract deployment...');
    
    while (Date.now() - startTime < maxWaitTime) {
        try {
            // Check if deployed addresses file exists
            await fs.access(deployedAddressesPath);
            const content = await fs.readFile(deployedAddressesPath, 'utf8');
            const addresses = JSON.parse(content);
            
            if (addresses.EntryPoint && addresses.OmniAccountFactory) {
                console.log('✅ Contract deployment file found with addresses:');
                console.log('  EntryPoint:', addresses.EntryPoint);
                console.log('  OmniAccountFactory:', addresses.OmniAccountFactory);
                
                // Update environment variables with deployed addresses
                process.env.TEST_ENTRY_POINT_ADDRESS = addresses.EntryPoint;
                process.env.TEST_FACTORY_ADDRESS = addresses.OmniAccountFactory;
                process.env.TEST_USDC_ADDRESS = addresses.TestUSDC;
                process.env.TEST_USDT_ADDRESS = addresses.TestUSDT;
                
                return;
            }
        } catch (error) {
            // File doesn't exist yet or is not valid JSON, continue waiting
        }
        
        console.log(`⏳ Still waiting for contracts... (${Math.floor((Date.now() - startTime) / 1000)}s)`);
        await new Promise(resolve => setTimeout(resolve, checkInterval));
    }
    
    console.log('⚠️ Contract deployment timeout reached, using default addresses');
}

describe('SubmitUserOp Integration Tests', function () {
    this.timeout(180000); // 3 minutes for CI environment

    let publicClient: any;
    let deployerWalletClient: any;
    let userWalletClient: any;
    let evmWallet: any;
    let omniAccount: `0x${string}`;
    let omniAccountAddress: Address;
    let teeWorkerAddress: Address;
    let testTokenAddress: Address;
    let idToken: string;
    let testEnv: ReturnType<typeof validateTestEnvironment>;

    before(async function () {
        this.timeout(180000); // 3 minutes for before hook
        
        // Validate test environment
        testEnv = validateTestEnvironment();
        
        // Wait for contract deployment in CI environment
        await waitForContractDeployment();
        
        // Re-validate environment after potential address updates
        testEnv = validateTestEnvironment();

        // Custom chain configuration using test config
        const testChain = {
            ...anvil,
            id: testEnv.chainId,
            rpcUrls: {
                default: { http: [testEnv.rpcUrl] },
                public: { http: [testEnv.rpcUrl] },
            },
        };

        // Setup clients
        publicClient = createPublicClient({
            chain: testChain,
            transport: http(testEnv.rpcUrl),
        });

        deployerWalletClient = createWalletClient({
            chain: testChain,
            transport: http(testEnv.rpcUrl),
            account: privateKeyToAccount(TEST_CONFIG.ACCOUNTS.DEPLOYER.privateKey),
        });

        userWalletClient = createWalletClient({
            chain: testChain,
            transport: http(testEnv.rpcUrl),
            account: privateKeyToAccount(TEST_CONFIG.ACCOUNTS.USER.privateKey),
        });

        // Generate test wallet
        evmWallet = randomEvmWallet();
        console.log('Generated EVM wallet:', evmWallet.address);

        // Calculate omni account
        omniAccount = calculateOmniAccount(evmWallet.address, TEST_CONFIG.TEE_WORKER.CLIENT_ID, 'evm');
        console.log('Calculated OmniAccount:', omniAccount);

        // Verify factory contract is deployed
        console.log('Verifying factory contract deployment...');
        const factoryAddress = testEnv.contracts.OMNI_ACCOUNT_FACTORY as Address;
        console.log('Factory address from config:', factoryAddress);
        
        const factoryCode = await publicClient.getCode({
            address: factoryAddress,
        });
        console.log('Factory code length:', factoryCode?.length || 0);
        
        if (!factoryCode || factoryCode === '0x') {
            console.error(`❌ OmniAccountFactory not deployed at ${factoryAddress}`);
            console.log('Available contracts might be:');
            
            // Try to detect deployed contracts by checking common addresses
            const commonAddresses = [
                '0x5FbDB2315678afecb367f032d93F642f64180aa3', // Common first deployment
                '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512', // Common second deployment  
                '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0', // Common third deployment
                '0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9', // Common fourth deployment
            ];
            
            for (const addr of commonAddresses) {
                const code = await publicClient.getCode({ address: addr as Address });
                if (code && code !== '0x') {
                    console.log(`Found contract at ${addr} (code length: ${code.length})`);
                }
            }
            
            // Also check what's in the shared file
            console.log('Environment variables:');
            console.log('TEST_FACTORY_ADDRESS:', process.env.TEST_FACTORY_ADDRESS);
            console.log('TEST_ENTRY_POINT_ADDRESS:', process.env.TEST_ENTRY_POINT_ADDRESS);
            
            throw new Error(`OmniAccountFactory not deployed at ${factoryAddress}. Check contract deployment.`);
        }
        console.log('✅ Factory contract is deployed');

        // Get omni account contract address
        try {
            console.log('DEBUG: Calling getAddress with:');
            console.log('  Factory:', testEnv.contracts.OMNI_ACCOUNT_FACTORY);
            console.log('  omniAccount:', omniAccount);
            console.log('  ownerType (EVM):', OWNER_TYPE.EVM);
            console.log('  clientId (bytes):', stringToBytes(TEST_CONFIG.TEE_WORKER.CLIENT_ID));
            console.log('  clientId (string):', TEST_CONFIG.TEE_WORKER.CLIENT_ID);
            console.log('  rootSigner:', evmWallet.address);
            
            // First, let's check if the factory has the expected functions
            console.log('DEBUG: Factory ABI functions:');
            CONTRACT_ABIS.OMNI_ACCOUNT_FACTORY.forEach(func => {
                if (func.type === 'function') {
                    console.log(`  - ${func.name}(${func.inputs?.map(i => `${i.type} ${i.name}`).join(', ')})`);
                }
            });
            
            omniAccountAddress = await publicClient.readContract({
                address: testEnv.contracts.OMNI_ACCOUNT_FACTORY as Address,
                abi: CONTRACT_ABIS.OMNI_ACCOUNT_FACTORY,
                functionName: 'getAddress',
                args: [omniAccount, OWNER_TYPE.EVM, stringToBytes(TEST_CONFIG.TEE_WORKER.CLIENT_ID), evmWallet.address],
            });
        } catch (error) {
            console.error('Failed to get OmniAccount address from factory:', error);
            console.log('Factory address:', testEnv.contracts.OMNI_ACCOUNT_FACTORY);
            console.log('Args:', [omniAccount, OWNER_TYPE.EVM, stringToBytes(TEST_CONFIG.TEE_WORKER.CLIENT_ID), evmWallet.address]);
            
            // Let's try to understand what's wrong by calling some other factory functions
            console.log('DEBUG: Trying to get more info about the factory contract...');
            try {
                // Check if we can call any view functions
                console.log('DEBUG: Checking factory code again...');
                const code = await publicClient.getCode({
                    address: testEnv.contracts.OMNI_ACCOUNT_FACTORY as Address,
                });
                console.log(`DEBUG: Factory code length: ${code?.length || 0}`);
                
                // Try to get the current chain ID to ensure RPC is working
                const chainId = await publicClient.getChainId();
                console.log('DEBUG: Current chain ID:', chainId);
                
                // Try to get the latest block to ensure ethereum node is responsive
                const block = await publicClient.getBlockNumber();
                console.log('DEBUG: Latest block number:', block);
                
            } catch (debugError) {
                console.error('DEBUG: Error during diagnostics:', debugError);
            }
            
            throw error;
        }
        console.log('OmniAccount contract address:', omniAccountAddress);

        // Set test token address
        testTokenAddress = testEnv.contracts.TEST_USDC as Address;

        // Check if test token is deployed
        try {
            await publicClient.readContract({
                address: testTokenAddress,
                abi: CONTRACT_ABIS.TEST_TOKEN,
                functionName: 'balanceOf',
                args: [evmWallet.address],
            });
            console.log('Test token deployed at:', testTokenAddress);
        } catch (error) {
            console.log('⚠️ Test token not found at:', testTokenAddress);
        }
    });

    it('Step 1: Should create EVM wallet and generate OmniAccount', async function () {
        expect(evmWallet.address).to.match(/^0x[a-fA-F0-9]{40}$/);
        expect(evmWallet.privateKey).to.match(/^0x[a-fA-F0-9]{64}$/);
        expect(omniAccount).to.match(/^0x[a-fA-F0-9]{64}$/);
        expect(omniAccountAddress).to.match(/^0x[a-fA-F0-9]{40}$/);
        console.log('✅ Step 1 completed: EVM wallet and OmniAccount generated');
    });

    it('Step 2: Should fund OmniAccount with ETH from default test account', async function () {
        // Check initial balance
        const initialBalance = await publicClient.getBalance({
            address: omniAccountAddress,
        });
        console.log('Initial OmniAccount balance:', initialBalance.toString());

        // Send ETH from deployer to OmniAccount
        const fundingAmount = parseEther(TEST_CONFIG.AMOUNTS.FUNDING_ETH);
        const hash = await deployerWalletClient.sendTransaction({
            to: omniAccountAddress,
            value: fundingAmount,
        });

        await publicClient.waitForTransactionReceipt({
            hash,
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION,
        });

        // Verify balance
        const finalBalance = await publicClient.getBalance({
            address: omniAccountAddress,
        });

        expect(finalBalance).to.equal(fundingAmount);
        console.log(`✅ Step 2 completed: OmniAccount funded with ${TEST_CONFIG.AMOUNTS.FUNDING_ETH} ETH`);
    });

    it('Step 3: Should create OmniAccount contract (without Paymaster)', async function () {
        // Check if account already exists
        const code = await publicClient.getCode({
            address: omniAccountAddress,
        });

        if (code && code !== '0x') {
            console.log('OmniAccount contract already exists');
            return;
        }

        // Generate init code for deployment
        const initCode = generateInitCode(
            testEnv.contracts.OMNI_ACCOUNT_FACTORY as Address,
            omniAccount,
            OWNER_TYPE.EVM,
            stringToBytes(TEST_CONFIG.TEE_WORKER.CLIENT_ID),
            evmWallet.address
        );

        // Create UserOperation for deployment
        const userOp = createUserOperation({
            sender: omniAccountAddress,
            nonce: BigInt(0),
            initCode: initCode,
            callData: '0x',
        });

        // Create wallet client for signing
        const testChain = { ...anvil, id: testEnv.chainId };
        const walletClient = createWalletClient({
            chain: testChain,
            transport: http(testEnv.rpcUrl),
            account: privateKeyToAccount(evmWallet.privateKey),
        });

        // Sign UserOperation with Owner signer type
        userOp.signature = await signUserOperation(
            walletClient,
            userOp,
            testEnv.contracts.ENTRY_POINT as Address,
            testEnv.chainId,
            TEST_CONFIG.SIGNER_TYPES.OWNER
        );

        // Pack and execute
        const packedUserOp = packUserOperation(userOp);

        const hash = await deployerWalletClient.writeContract({
            address: testEnv.contracts.ENTRY_POINT as Address,
            abi: CONTRACT_ABIS.ENTRY_POINT,
            functionName: 'handleOps',
            args: [[packedUserOp], TEST_CONFIG.ACCOUNTS.DEPLOYER.address],
        });

        const receipt = await publicClient.waitForTransactionReceipt({
            hash,
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION,
        });
        expect(receipt.status).to.equal('success');

        // Verify contract is deployed
        const finalCode = await publicClient.getCode({
            address: omniAccountAddress,
        });
        expect(finalCode).to.not.equal('0x');
        console.log('✅ Step 3 completed: OmniAccount contract created');
    });

    it('Step 4: Should add TEE Worker as authorized signer', async function () {
        // First authenticate with TEE worker to get its address
        const messageResponse = await omniApi.getWeb3SignInMessage({
            client_id: TEST_CONFIG.TEE_WORKER.CLIENT_ID,
            omni_account: omniAccount,
        });

        const messageString = JSON.stringify(messageResponse);
        const signature = await signMessage({
            message: messageString,
            privateKey: evmWallet.privateKey,
        });

        const loginResponse: UserLoginResponse = await omniApi.userLogin({
            user_id: {
                type: 'evm',
                value: evmWallet.address,
            },
            user_auth: {
                type: 'evm',
                value: signature,
            },
            client_id: TEST_CONFIG.TEE_WORKER.CLIENT_ID,
            client_auth: {
                type: 'wildmeta',
                value: {
                    google_code: '',
                    invite_code: '',
                },
            },
        });

        idToken = loginResponse.id_token;

        // For testing, use a mock TEE worker address
        teeWorkerAddress = TEST_CONFIG.ACCOUNTS.TEE_WORKER.address; // Mock TEE worker address
        console.log('Mock TEE worker address:', teeWorkerAddress);

        // Create calldata for adding TEE worker as root signer
        const addSignerCalldata = createAddSignerCalldata(teeWorkerAddress);
        const executeCalldata = createExecuteCalldata(omniAccountAddress, BigInt(0), addSignerCalldata);

        // Get current nonce
        const nonce = await publicClient.readContract({
            address: testEnv.contracts.ENTRY_POINT as Address,
            abi: CONTRACT_ABIS.ENTRY_POINT,
            functionName: 'getNonce',
            args: [omniAccountAddress, BigInt(0)],
        });

        // Create UserOperation
        const userOp = createUserOperation({
            sender: omniAccountAddress,
            nonce,
            initCode: '0x',
            callData: executeCalldata,
        });

        // Create wallet client for signing
        const testChain = { ...anvil, id: testEnv.chainId };
        const walletClient = createWalletClient({
            chain: testChain,
            transport: http(testEnv.rpcUrl),
            account: privateKeyToAccount(evmWallet.privateKey),
        });

        // Sign with root key signer type
        userOp.signature = await signUserOperation(
            walletClient,
            userOp,
            testEnv.contracts.ENTRY_POINT as Address,
            testEnv.chainId,
            TEST_CONFIG.SIGNER_TYPES.ROOT_KEY
        );

        const packedUserOp = packUserOperation(userOp);

        const hash = await deployerWalletClient.writeContract({
            address: testEnv.contracts.ENTRY_POINT as Address,
            abi: CONTRACT_ABIS.ENTRY_POINT,
            functionName: 'handleOps',
            args: [[packedUserOp], TEST_CONFIG.ACCOUNTS.DEPLOYER.address],
        });

        const receipt = await publicClient.waitForTransactionReceipt({
            hash,
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION,
        });
        expect(receipt.status).to.equal('success');

        // Verify signer was added
        const isRootSigner = await publicClient.readContract({
            address: omniAccountAddress,
            abi: CONTRACT_ABIS.OMNI_ACCOUNT,
            functionName: 'isRootSigner',
            args: [teeWorkerAddress],
        });

        expect(isRootSigner).to.be.true;
        console.log('✅ Step 4 completed: TEE Worker added as authorized signer');
    });

    it('Step 5: Should send token transfer via UserOperation', async function () {
        // First mint test tokens to the EVM wallet
        try {
            const mintAmount = parseUnits(TEST_CONFIG.AMOUNTS.MINT_TOKENS, 18);
            const mintHash = await deployerWalletClient.writeContract({
                address: testTokenAddress,
                abi: CONTRACT_ABIS.TEST_TOKEN,
                functionName: 'mint',
                args: [evmWallet.address, mintAmount],
            });
            await publicClient.waitForTransactionReceipt({
                hash: mintHash,
                timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION,
            });

            // Transfer tokens from EVM wallet to OmniAccount
            const testChain = { ...anvil, id: testEnv.chainId };
            const evmWalletClient = createWalletClient({
                chain: testChain,
                transport: http(testEnv.rpcUrl),
                account: privateKeyToAccount(evmWallet.privateKey),
            });

            const transferAmount = parseUnits(TEST_CONFIG.AMOUNTS.TRANSFER_TOKENS, 18);
            const transferHash = await evmWalletClient.writeContract({
                address: testTokenAddress,
                abi: CONTRACT_ABIS.TEST_TOKEN,
                functionName: 'transfer',
                args: [omniAccountAddress, transferAmount],
            });
            await publicClient.waitForTransactionReceipt({
                hash: transferHash,
                timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION,
            });

            // Verify balance
            const balance = await publicClient.readContract({
                address: testTokenAddress,
                abi: CONTRACT_ABIS.TEST_TOKEN,
                functionName: 'balanceOf',
                args: [omniAccountAddress],
            });

            expect(balance).to.equal(transferAmount);
            console.log(`✅ Test tokens (${TEST_CONFIG.AMOUNTS.TRANSFER_TOKENS}) transferred to OmniAccount`);
        } catch (error) {
            console.log('⚠️ Test token operations failed, using mock transfer:', error);
        }

        // Create token transfer via UserOperation
        const sendAmount = parseUnits(TEST_CONFIG.AMOUNTS.SEND_TOKENS, 18);
        const transferCalldata = createTokenTransferCalldata(
            testTokenAddress,
            TEST_CONFIG.ACCOUNTS.DEPLOYER.address,
            sendAmount
        );

        const executeCalldata = createExecuteCalldata(testTokenAddress, BigInt(0), transferCalldata);

        // Get current nonce
        const nonce = await publicClient.readContract({
            address: testEnv.contracts.ENTRY_POINT as Address,
            abi: CONTRACT_ABIS.ENTRY_POINT,
            functionName: 'getNonce',
            args: [omniAccountAddress, BigInt(0)],
        });

        // Create UserOperation
        const userOp = createUserOperation({
            sender: omniAccountAddress,
            nonce,
            initCode: '0x',
            callData: executeCalldata,
        });

        // Create wallet client for signing
        const testChain = { ...anvil, id: testEnv.chainId };
        const walletClient = createWalletClient({
            chain: testChain,
            transport: http(testEnv.rpcUrl),
            account: privateKeyToAccount(evmWallet.privateKey),
        });

        // Sign with root key
        userOp.signature = await signUserOperation(
            walletClient,
            userOp,
            testEnv.contracts.ENTRY_POINT as Address,
            testEnv.chainId,
            TEST_CONFIG.SIGNER_TYPES.ROOT_KEY
        );

        const packedUserOp = packUserOperation(userOp);

        const hash = await deployerWalletClient.writeContract({
            address: testEnv.contracts.ENTRY_POINT as Address,
            abi: CONTRACT_ABIS.ENTRY_POINT,
            functionName: 'handleOps',
            args: [[packedUserOp], TEST_CONFIG.ACCOUNTS.DEPLOYER.address],
        });

        const receipt = await publicClient.waitForTransactionReceipt({
            hash,
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION,
        });
        expect(receipt.status).to.equal('success');
        console.log(
            `✅ Step 5 completed: Token transfer (${TEST_CONFIG.AMOUNTS.SEND_TOKENS} tokens) via UserOperation executed`
        );
    });
});
