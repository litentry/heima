use executor_primitives::Hash;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

// Simple replacement for TransactionStatus
#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum TransactionStatus<Hash> {
	InBlock(Hash),
	Finalized(Hash),
	Invalid,
	Dropped,
}

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

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum NativeTaskOk {
	ExtrinsicReport {
		extrinsic_hash: Hash,
		block_hash: Option<Hash>,
		status: TransactionStatus<Hash>,
	},
	AuthToken(String),
	RequestIntentResult {
		intent_id: u32,
		success: bool,
	},
	SubmitUserOp(Option<String>), // transaction_hash
	EstimateUserOpGas {
		call_gas_limit: u128,
		verification_gas_limit: u128,
		pre_verification_gas: u128,
		paymaster_verification_gas_limit: u128,
		paymaster_post_op_gas_limit: u128,
		max_fee_per_gas: u128,
		max_priority_fee_per_gas: u128,
		estimated_token_cost: Option<TokenCostEstimate>,
	},
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTaskError {
	UnauthorizedSender,
	AuthTokenCreationFailed,
	InternalError(Option<String>),
	InvalidMemberIdentity,
	ValidationDataVerificationFailed,
	UnsupportedIdentityType,
	IntentNonceMismatch,
	UnsupportedChain,
	ChainNotSupported(u64),
	InvalidUserOperation(String),
	GasEstimationFailed,
	SignatureServiceUnavailable,
}
