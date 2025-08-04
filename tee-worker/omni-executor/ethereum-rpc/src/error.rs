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

use alloy::transports::RpcError as AlloyRpcError;
use serde_json::Value;
use thiserror::Error;

/// Error type for RPC operations
#[derive(Error, Debug, Clone)]
pub enum RpcProviderError {
	/// URL parsing errors
	#[error("Invalid URL: {0}")]
	InvalidUrl(String),

	/// Network/transport errors
	#[error("Network error: {0}")]
	Network(String),

	/// Transaction-related errors
	#[error("Transaction error: {0}")]
	Transaction(String),

	/// JSON-RPC error response
	#[error("RPC error (code {code}): {message}")]
	JsonRpc {
		/// Error code from JSON-RPC
		code: i32,
		/// Error message
		message: String,
		/// Optional error data
		data: Option<String>,
	},

	/// Contract revert with reason
	#[error("Execution reverted: {reason}")]
	ExecutionReverted {
		/// Revert reason or data
		reason: String,
	},

	/// No wallet configured for signing
	#[error("No wallet configured")]
	NoWallet,

	/// Generic error for other cases
	#[error("{0}")]
	Generic(String),
}

impl RpcProviderError {
	/// Convert from alloy RpcError
	pub fn from_alloy_error<E>(err: AlloyRpcError<E>) -> Self
	where
		E: std::error::Error + Send + Sync + 'static,
	{
		match err {
			AlloyRpcError::ErrorResp(resp) => {
				// Extract data if available
				let data = resp.data.map(|v| match serde_json::from_str::<Value>(v.get()) {
					Ok(Value::String(s)) => s,
					Ok(v) => v.to_string(),
					Err(_) => v.get().to_string(),
				});

				// Check for common execution revert patterns
				if let Some(ref data_str) = data {
					if data_str.starts_with("0x08c379a0") || // Standard revert
					   data_str.contains("execution reverted") ||
					   resp.message.contains("execution reverted")
					{
						return Self::ExecutionReverted { reason: resp.message.to_string() };
					}
				}

				Self::JsonRpc { code: resp.code as i32, message: resp.message.to_string(), data }
			},
			AlloyRpcError::Transport(err) => Self::Network(err.to_string()),
			AlloyRpcError::LocalUsageError(err) => {
				Self::Generic(format!("Local usage error: {}", err))
			},
			AlloyRpcError::UnsupportedFeature(msg) => {
				Self::Generic(format!("Unsupported feature: {}", msg))
			},
			AlloyRpcError::NullResp => Self::Generic("Null response from RPC".to_string()),
			AlloyRpcError::SerError(err) => Self::Generic(format!("Serialization error: {}", err)),
			AlloyRpcError::DeserError { err, text } => {
				Self::Generic(format!("Deserialization error: {} (text: {})", err, text))
			},
		}
	}

	/// Check if this is a retryable error
	pub fn is_retryable(&self) -> bool {
		match self {
			// Network errors are usually retryable
			Self::Network(_) => true,
			Self::InvalidUrl(_) => false,
			Self::NoWallet => false,
			Self::ExecutionReverted { .. } => false,

			// Check JSON-RPC error codes
			Self::JsonRpc { code, message, .. } => {
				match *code {
					// Standard JSON-RPC errors that might be retryable
					-32603 => true, // Internal error
					-32099..=-32000 => {
						// Server errors - check message for specifics
						// First check if it's intrinsic gas too low (NOT retryable)
						if message.contains("intrinsic gas too low") {
							false
						} else {
							// Check for retryable errors
							message.contains("nonce too low")
								|| message.contains("replacement transaction underpriced")
								|| message.contains("transaction underpriced")
								|| message.contains("already known")
								|| message.contains("timeout")
								|| message.contains("max fee per gas less than block base fee")
								|| message.contains("gas too low")
								|| message.contains("gas price too low")
						}
					},
					// Rate limiting
					429 => true,
					_ => false,
				}
			},

			Self::Transaction(msg) => {
				// Parse transaction errors for retryable conditions
				// First check if it's intrinsic gas too low (NOT retryable)
				if msg.contains("intrinsic gas too low") {
					false
				} else {
					// Check for retryable errors
					msg.contains("nonce too low")
						|| msg.contains("replacement transaction underpriced")
						|| msg.contains("transaction underpriced")
						|| msg.contains("gas price too low")
						|| msg.contains("max fee per gas less than block base fee")
						|| msg.contains("gas too low")
				}
			},

			Self::Generic(_) => false,
		}
	}
}

// Conversion from unit type for backward compatibility
impl From<()> for RpcProviderError {
	fn from(_: ()) -> Self {
		RpcProviderError::Generic("Unknown error".to_string())
	}
}

// Allow converting to unit type for gradual migration
impl From<RpcProviderError> for () {
	fn from(_: RpcProviderError) -> Self {}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_error_patterns_retryability() {
		// Test "max fee per gas less than block base fee" - should be retryable
		let error1 = RpcProviderError::Transaction("max fee per gas less than block base fee".to_string());
		assert!(error1.is_retryable());

		let error2 = RpcProviderError::JsonRpc { 
			code: -32000, 
			message: "max fee per gas less than block base fee".to_string(),
			data: None
		};
		assert!(error2.is_retryable());

		// Test "gas too low" - should be retryable
		let error3 = RpcProviderError::Transaction("gas too low".to_string());
		assert!(error3.is_retryable());

		// Test "intrinsic gas too low" - should NOT be retryable
		let error4 = RpcProviderError::Transaction("intrinsic gas too low".to_string());
		assert!(!error4.is_retryable());

		let error5 = RpcProviderError::JsonRpc {
			code: -32000,
			message: "intrinsic gas too low".to_string(),
			data: None
		};
		assert!(!error5.is_retryable());

		// Test "transaction underpriced" - should be retryable
		let error6 = RpcProviderError::Transaction("transaction underpriced".to_string());
		assert!(error6.is_retryable());

		// Test "gas price too low" - should be retryable
		let error7 = RpcProviderError::JsonRpc {
			code: -32000,
			message: "gas price too low for the network".to_string(),
			data: None
		};
		assert!(error7.is_retryable());
	}
}

