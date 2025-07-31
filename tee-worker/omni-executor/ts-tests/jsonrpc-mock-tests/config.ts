// Test configuration constants
export const TEST_CONFIG = {
    // Network configuration
    RPC_URL: process.env.TEST_RPC_URL || 'http://127.0.0.1:8545',
    CHAIN_ID: process.env.TEST_CHAIN_ID ? parseInt(process.env.TEST_CHAIN_ID) : 31337,

    // Test accounts from Anvil/Hardhat default test mnemonic:
    // "test test test test test test test test test test test junk"
    // These addresses are deterministically generated and have public private keys
    ACCOUNTS: {
        DEPLOYER: {
            address: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266' as const, // Account #0
            privateKey: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80' as const,
        },
        USER: {
            address: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8' as const, // Account #1
            privateKey: '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d' as const,
        },
        TEE_WORKER: {
            address: '0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC' as const, // Account #2
            privateKey: '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a' as const,
        },
    },

    // Contract addresses - these are deployed by local-deploy.sh
    CONTRACTS: {
        ENTRY_POINT: process.env.TEST_ENTRY_POINT_ADDRESS || '0x5FbDB2315678afecb367f032d93F642f64180aa3',
        OMNI_ACCOUNT_FACTORY: process.env.TEST_FACTORY_ADDRESS || '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
        SIMPLE_PAYMASTER: process.env.TEST_PAYMASTER_ADDRESS || '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0',
        TEST_USDC: process.env.TEST_USDC_ADDRESS || '0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9',
        TEST_USDT: process.env.TEST_USDT_ADDRESS || '0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9',
    },

    // TEE Worker RPC configuration
    TEE_WORKER: {
        RPC_URL: process.env.TEE_WORKER_RPC_URL || 'http://localhost:2100',
        CLIENT_ID: 'wildmeta' as const,
        CHAIN_TYPE: 'Evm' as const,
        SIGNER_INDEX: 0,
    },

    // Gas configuration for UserOperations
    GAS_CONFIG: {
        CALL_GAS_LIMIT: BigInt(2000000),
        VERIFICATION_GAS_LIMIT: BigInt(3000000),
        PRE_VERIFICATION_GAS: BigInt(100000),
        MAX_FEE_PER_GAS: BigInt(20000000000), // 20 gwei
        MAX_PRIORITY_FEE_PER_GAS: BigInt(1000000000), // 1 gwei
    },

    // Token amounts
    AMOUNTS: {
        FUNDING_ETH: '1', // ETH amount to fund OmniAccount
        MINT_TOKENS: '1000', // Amount of test tokens to mint
        TRANSFER_TOKENS: '500', // Amount to transfer to OmniAccount
        SEND_TOKENS: '100', // Amount to send in test transaction
    },

    // UserOp signer types
    SIGNER_TYPES: {
        OWNER: 0x00,
        ROOT_KEY: 0x01,
        SESSION_KEY: 0x02,
        PASSKEY: 0x03,
    } as const,

    // Test timeouts
    TIMEOUTS: {
        DEFAULT: 120000, // 2 minutes
        TRANSACTION: 30000, // 30 seconds
    },
} as const;

// Utility type for contract addresses
export type ContractAddress = `0x${string}`;

// Environment validation
export function validateTestEnvironment() {
    const requiredEnvVars = ['TEST_RPC_URL', 'TEST_ENTRY_POINT_ADDRESS', 'TEST_FACTORY_ADDRESS'];

    const missing = requiredEnvVars.filter((envVar) => envVar.startsWith('TEST_') && !process.env[envVar]);

    if (missing.length > 0) {
        console.warn('⚠️  Missing environment variables (using defaults):', missing.join(', '));
        console.warn('   Set these variables or run local-deploy.sh to deploy contracts');
    }

    return {
        rpcUrl: TEST_CONFIG.RPC_URL,
        chainId: TEST_CONFIG.CHAIN_ID,
        contracts: TEST_CONFIG.CONTRACTS,
    };
}
