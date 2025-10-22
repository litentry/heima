use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Information about estimated token cost for ERC20 paymaster operations
#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq, Eq)]
pub struct TokenCostEstimate {
	/// The ERC20 token address used for gas payment
	pub token_address: String,
	/// Token amount in smallest unit (e.g., wei for 18 decimal tokens)
	pub amount: u128,
	/// Token decimals for frontend formatting
	pub decimals: u8,
	/// Exchange rate used for calculation (tokens per ETH * 10^18)
	pub exchange_rate: u128,
}

/// Gas estimation response for UserOperation
#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq, Eq)]
pub struct GasEstimateResponse {
	pub call_gas_limit: u128,
	pub verification_gas_limit: u128,
	pub pre_verification_gas: u128,
	pub paymaster_verification_gas_limit: u128,
	pub paymaster_post_op_gas_limit: u128,
	pub max_fee_per_gas: u128,
	pub max_priority_fee_per_gas: u128,
	pub estimated_token_cost: Option<TokenCostEstimate>,
}
