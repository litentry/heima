// Supported EVM-compatible chains
pub const SUPPORTED_EVM_CHAINS: &[u32] = &[
	1,      // Ethereum Mainnet
	5,      // Goerli
	10,     // Optimism
	56,     // BSC
	137,    // Polygon
	42161,  // Arbitrum One
	421613, // Arbitrum Goerli
	8453,   // Base
	84531,  // Base Goerli
];

// Maximum wallet index allowed
pub const MAX_WALLET_INDEX: u32 = 100;
