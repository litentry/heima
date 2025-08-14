// Supported EVM-compatible chains
pub const SUPPORTED_EVM_CHAINS: &[u32] = &[
	1,        // Ethereum Mainnet
	11155111, // Sepolia
	42161,    // Arbitrum
	421614,   // Arbitrum Sepolia
	10,       // Optimism
	11155420, // Optimism Sepolia
	137,      // Polygon
	80001,    // Polygon Mumbai
	8453,     // Base
	84532,    // Base Sepolia
	1337,     // Local Anvil
];

// Maximum wallet index allowed
pub const MAX_WALLET_INDEX: u32 = 100;
