use crate::detailed_error::DetailedError;
use crate::RpcResult;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tracing::error;

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

pub trait RpcResultExt<T> {
	fn map_err_internal(self, context: &str) -> RpcResult<T>;
	fn map_err_parse(self, context: &str) -> RpcResult<T>;
}

pub trait RpcOptionExt<T> {
	fn ok_or_internal(self, msg: &str) -> RpcResult<T>;
	fn ok_or_parse(self, msg: &str) -> RpcResult<T>;
}

impl<T, E: std::fmt::Debug> RpcResultExt<T> for Result<T, E> {
	fn map_err_internal(self, context: &str) -> RpcResult<T> {
		self.map_err(|e| {
			let msg = format!("{}: {:?}", context, e);
			error!(msg);
			DetailedError::internal_error(&msg).to_rpc_error()
		})
	}

	fn map_err_parse(self, context: &str) -> RpcResult<T> {
		self.map_err(|e| {
			let msg = format!("{}: {:?}", context, e);
			error!(msg);
			DetailedError::parse_error(&msg).to_rpc_error()
		})
	}
}

impl<T> RpcOptionExt<T> for Option<T> {
	fn ok_or_internal(self, msg: &str) -> RpcResult<T> {
		self.ok_or_else(|| {
			error!(msg);
			DetailedError::internal_error(msg).to_rpc_error()
		})
	}

	fn ok_or_parse(self, msg: &str) -> RpcResult<T> {
		self.ok_or_else(|| {
			error!(msg);
			DetailedError::parse_error(msg).to_rpc_error()
		})
	}
}
