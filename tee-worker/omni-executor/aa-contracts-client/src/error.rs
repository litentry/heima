// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use thiserror::Error;

/// Main error type for AA contract client operations
#[derive(Error, Debug, Clone)]
pub enum AaContractError {
	/// RPC-related errors
	#[error("RPC error: {0}")]
	Rpc(#[from] RpcError),

	/// Contract-specific errors
	#[error("Contract error: {0}")]
	Contract(#[from] ContractError),

	/// Validation errors
	#[error("Validation error: {0}")]
	Validation(String),

	/// Transaction errors
	#[error("Transaction error: {0}")]
	Transaction(String),

	/// Generic errors for backward compatibility
	#[error("Operation failed: {0}")]
	Generic(String),
}

/// RPC-specific errors that can occur during network communication
#[derive(Error, Debug, Clone)]
pub enum RpcError {
	/// Connection refused or node unavailable
	#[error("Failed to connect to RPC endpoint: {endpoint}")]
	ConnectionFailed { endpoint: String },

	/// Rate limiting error
	#[error("Rate limited by RPC provider")]
	RateLimited,

	/// Transaction underpriced (gas price too low)
	#[error("Transaction underpriced")]
	TransactionUnderpriced,

	/// Nonce too low
	#[error("Nonce too low")]
	NonceTooLow,

	/// Generic RPC error with code
	#[error("RPC error (code {code}): {message}")]
	Generic { code: i32, message: String },
}

/// Contract-specific errors that occur during smart contract interaction
#[derive(Error, Debug, Clone)]
pub enum ContractError {
	/// Insufficient funds for transaction
	#[error("Insufficient funds")]
	InsufficientFunds,

	/// Contract execution reverted
	#[error("Execution reverted: {reason}")]
	ExecutionReverted { reason: String },

	/// Invalid signature
	#[error("Invalid signature")]
	InvalidSignature,

	/// Gas estimation failed
	#[error("Gas estimation failed: {reason}")]
	GasEstimationFailed { reason: String },
}

impl RpcError {
	/// Check if this error is retryable
	pub fn is_retryable(&self) -> bool {
		match self {
			RpcError::ConnectionFailed { .. } => true,
			RpcError::RateLimited => true,
			RpcError::TransactionUnderpriced => true,
			RpcError::NonceTooLow => true,
			RpcError::Generic { code, .. } => {
				// Common retryable error codes
				matches!(
					*code,
					-32603 | // Internal JSON-RPC error
					-32000 | // Server error
					-32005 // Limit exceeded
				)
			},
		}
	}
}

impl ContractError {
	/// Check if this error is retryable
	pub fn is_retryable(&self) -> bool {
		match self {
			// These errors won't be fixed by retrying
			ContractError::InsufficientFunds => false,
			ContractError::ExecutionReverted { .. } => false,
			ContractError::InvalidSignature => false,

			// Gas estimation might succeed on retry if network conditions change
			ContractError::GasEstimationFailed { .. } => true,
		}
	}
}

impl AaContractError {
	/// Check if this error is retryable
	pub fn is_retryable(&self) -> bool {
		match self {
			AaContractError::Rpc(e) => e.is_retryable(),
			AaContractError::Contract(e) => e.is_retryable(),
			// Other error types are generally not retryable
			_ => false,
		}
	}
}

// Conversion from RpcProviderError
impl From<ethereum_rpc::RpcProviderError> for AaContractError {
	fn from(err: ethereum_rpc::RpcProviderError) -> Self {
		match err {
			ethereum_rpc::RpcProviderError::Network(msg) => {
				AaContractError::Rpc(RpcError::ConnectionFailed { endpoint: msg })
			},
			ethereum_rpc::RpcProviderError::Transaction(msg) => {
				AaContractError::Generic(format!("Transaction error: {}", msg))
			},
			ethereum_rpc::RpcProviderError::JsonRpc { code, message, .. } => {
				AaContractError::Rpc(RpcError::Generic { code, message })
			},
			ethereum_rpc::RpcProviderError::ExecutionReverted { reason, .. } => {
				AaContractError::Contract(ContractError::ExecutionReverted { reason })
			},
			ethereum_rpc::RpcProviderError::Generic(msg) => AaContractError::Generic(msg),
			_ => AaContractError::Generic("RPC provider error".to_string()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_rpc_error_retryability() {
		// Retryable errors
		assert!(RpcError::ConnectionFailed { endpoint: "http://localhost:8545".to_string() }
			.is_retryable());
		assert!(RpcError::RateLimited.is_retryable());
		assert!(RpcError::TransactionUnderpriced.is_retryable());
		assert!(RpcError::NonceTooLow.is_retryable());
		assert!(RpcError::Generic { code: -32603, message: "Internal error".to_string() }
			.is_retryable());

		// Non-retryable errors
		assert!(!RpcError::Generic { code: -32601, message: "Method not found".to_string() }
			.is_retryable());
	}

	#[test]
	fn test_contract_error_retryability() {
		// Non-retryable errors
		assert!(!ContractError::InsufficientFunds.is_retryable());
		assert!(!ContractError::ExecutionReverted { reason: "Transfer failed".to_string() }
			.is_retryable());
		assert!(!ContractError::InvalidSignature.is_retryable());

		// Retryable errors
		assert!(ContractError::GasEstimationFailed { reason: "Network congestion".to_string() }
			.is_retryable());
	}
}
