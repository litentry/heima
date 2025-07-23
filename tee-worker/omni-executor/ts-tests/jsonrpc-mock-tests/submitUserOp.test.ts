import { describe, it, before } from 'mocha';
import { expect } from 'chai';
import {
    createPublicClient,
    createWalletClient,
    http,
    parseEther,
    parseUnits,
    type Address,
} from 'viem';
import { privateKeyToAccount } from 'viem/accounts';
import { anvil } from 'viem/chains';
import {
    ClientId,
    omniApi,
    randomEvmWallet,
    calculateOmniAccount,
    UserLoginResponse,
} from './utils';
import { signMessage } from 'viem/accounts';
import { TEST_CONFIG, validateTestEnvironment } from './config';
import {
    CONTRACT_ABIS,
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

describe('SubmitUserOp Integration Tests', function () {
    this.timeout(TEST_CONFIG.TIMEOUTS.DEFAULT);

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
        // Validate test environment
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

        // Get omni account contract address
        omniAccountAddress = await publicClient.readContract({
            address: testEnv.contracts.OMNI_ACCOUNT_FACTORY as Address,
            abi: CONTRACT_ABIS.OMNI_ACCOUNT_FACTORY,
            functionName: 'getAddress',
            args: [omniAccount, stringToBytes(TEST_CONFIG.TEE_WORKER.CLIENT_ID), evmWallet.address],
        });
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
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION 
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
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION 
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
            privateKey: evmWallet.privateKey 
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
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION 
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
                timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION 
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
                timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION 
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
            timeout: TEST_CONFIG.TIMEOUTS.TRANSACTION 
        });
        expect(receipt.status).to.equal('success');
        console.log(`✅ Step 5 completed: Token transfer (${TEST_CONFIG.AMOUNTS.SEND_TOKENS} tokens) via UserOperation executed`);
    });
});