use crate::config::{MAX_WALLET_INDEX, SUPPORTED_EVM_CHAINS};
use crate::detailed_error::DetailedError;
use crate::RpcResult;
use alloy::primitives::Address;
use email_address::EmailAddress;
use std::str::FromStr;
use tracing::error;

/// Generic function to parse a string into a numeric type (f64, u128, etc.)
/// Returns RpcResult with field name in error message for better debugging
///
/// # Example
/// ```ignore
/// let value: f64 = parse_as(&some_string, "collateral_size")?;
/// let count: u128 = parse_as(&count_string, "hedge_open_cloid")?;
/// ```
pub fn parse_as<T>(value: &str, field_name: &str) -> RpcResult<T>
where
	T: FromStr,
	T::Err: std::fmt::Display,
{
	value.parse::<T>().map_err(|e| {
		DetailedError::parse_error(&format!("Failed to parse {}: {}", field_name, e))
			.with_field(field_name)
			.with_received(value.to_string())
			.to_rpc_error()
	})
}

/// Generic function to parse JSON-RPC params into a typed struct
/// Returns RpcResult with consistent error handling
///
/// # Example
/// ```ignore
/// let params = parse_rpc_params::<RequestLoanTestParams>(params)?;
/// ```
pub fn parse_rpc_params<T: serde::de::DeserializeOwned>(
	params: jsonrpsee::types::Params,
) -> RpcResult<T> {
	params.parse::<T>().map_err(|e| {
		error!("Failed to parse RPC params: {:?}", e);
		DetailedError::parse_error("Invalid JSON format or missing required fields").to_rpc_error()
	})
}

pub fn validate_chain_id(chain_id: u32) -> RpcResult<()> {
	if !SUPPORTED_EVM_CHAINS.contains(&chain_id) {
		return Err(DetailedError::invalid_chain_id(chain_id as u64).to_rpc_error());
	}
	Ok(())
}

pub fn validate_wallet_index(index: u32) -> RpcResult<()> {
	if index > MAX_WALLET_INDEX {
		return Err(DetailedError::invalid_params(
			"wallet_index",
			&format!("exceed {}", MAX_WALLET_INDEX),
		)
		.to_rpc_error());
	}
	Ok(())
}

pub fn validate_evm_address(address: &str, field: &str) -> RpcResult<Address> {
	Address::from_str(address).map_err(|e| {
		DetailedError::invalid_params(field, &format!("invalid evm address: {:?}", e))
			.to_rpc_error()
	})
}

pub fn validate_amount(amount: &str, field: &str) -> RpcResult<u128> {
	let amount = parse_as::<u128>(amount, field)?;

	if amount == 0 {
		return Err(DetailedError::invalid_params(field, "expect non-zero").to_rpc_error());
	}

	Ok(amount)
}

pub fn validate_email(email: &str) -> RpcResult<()> {
	if !EmailAddress::is_valid(email) {
		return Err(DetailedError::new(
			crate::error_code::INVALID_EMAIL_FORMAT_CODE,
			"Invalid email format",
		)
		.with_field("email")
		.with_received(email.to_string())
		.with_expected("Valid email address (e.g., user@example.com)")
		.with_suggestion("Please provide a valid email address")
		.to_rpc_error());
	}

	// Additionally require a TLD (at least one dot after @)
	if let Some(at_pos) = email.find('@') {
		let domain = &email[at_pos + 1..];
		if !domain.contains('.') {
			return Err(DetailedError::new(
				crate::error_code::INVALID_EMAIL_FORMAT_CODE,
				"Invalid email format",
			)
			.with_field("email")
			.with_received(email.to_string())
			.with_expected("Email address with a valid domain (e.g., user@example.com)")
			.with_suggestion("Email domain must include a top-level domain (TLD)")
			.to_rpc_error());
		}
	}

	Ok(())
}

pub fn validate_user_operations(
	operations: &[executor_core::types::SerializablePackedUserOperation],
) -> RpcResult<()> {
	if operations.is_empty() {
		return Err(DetailedError::new(
			crate::error_code::MISSING_REQUIRED_FIELD_CODE,
			"User operations cannot be empty",
		)
		.with_field("user_operations")
		.with_expected("At least one user operation")
		.with_suggestion("Provide at least one user operation to submit")
		.to_rpc_error());
	}

	// Validate each operation's sender address
	for (index, op) in operations.iter().enumerate() {
		validate_evm_address(&op.sender, &format!("user_operations[{}].sender", index))?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_validate_chain_id_evm_valid() {
		// Valid EVM chain IDs - Mainnets
		assert!(validate_chain_id(1).is_ok());
		assert!(validate_chain_id(137).is_ok());
		assert!(validate_chain_id(42161).is_ok());
		assert!(validate_chain_id(10).is_ok());
		assert!(validate_chain_id(8453).is_ok());
		assert!(validate_chain_id(56).is_ok());

		// Valid EVM chain IDs - Testnets
		assert!(validate_chain_id(11155111).is_ok());
		assert!(validate_chain_id(421614).is_ok());
		assert!(validate_chain_id(11155420).is_ok());
		assert!(validate_chain_id(80001).is_ok());
		assert!(validate_chain_id(84532).is_ok());
		assert!(validate_chain_id(97).is_ok());
		assert!(validate_chain_id(31337).is_ok());
	}

	#[test]
	fn test_validate_chain_id_evm_invalid() {
		// Invalid EVM chain IDs
		assert!(validate_chain_id(5).is_err());
		assert!(validate_chain_id(421613).is_err());
		assert!(validate_chain_id(84531).is_err());
	}

	#[test]
	fn test_validate_wallet_index_valid() {
		assert!(validate_wallet_index(0).is_ok());
		assert!(validate_wallet_index(50).is_ok());
		assert!(validate_wallet_index(100).is_ok());
	}

	#[test]
	fn test_validate_wallet_index_invalid() {
		assert!(validate_wallet_index(101).is_err());
		assert!(validate_wallet_index(1000).is_err());
	}

	#[test]
	fn test_validate_evm_address_valid() {
		let addresses = vec![
			"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9",
			"0x0000000000000000000000000000000000000000",
			"0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		];

		for addr in addresses {
			assert!(validate_evm_address(addr, "test").is_ok());
		}
	}

	#[test]
	fn test_validate_evm_address_invalid() {
		let invalid_addresses = vec![
			"not_an_address",
			"0x",
			"0xGGGG", // Invalid hex
			"",
		];

		for addr in invalid_addresses {
			assert!(validate_evm_address(addr, "test").is_err());
		}
	}

	#[test]
	fn test_validate_amount_valid() {
		assert_eq!(validate_amount("100", "test").unwrap(), 100u128);
		assert_eq!(validate_amount("999999999", "test").unwrap(), 999999999u128);
	}

	#[test]
	fn test_validate_amount_invalid() {
		assert!(validate_amount("", "test").is_err()); // Empty
		assert!(validate_amount("0", "test").is_err()); // Zero
		assert!(validate_amount("abc", "test").is_err()); // Non-numeric
		assert!(validate_amount("-100", "test").is_err()); // Negative
	}

	#[test]
	fn test_validate_email_valid() {
		let valid_emails = vec![
			"test@example.com",
			"user.name@example.co.uk",
			"test+tag@example.org",
			"test_123@test-domain.com",
		];

		for email in valid_emails {
			assert!(validate_email(email).is_ok(), "Failed for: {}", email);
		}
	}

	#[test]
	fn test_validate_email_invalid() {
		let invalid_emails =
			vec!["notanemail", "@example.com", "test@", "test@.com", "test@example", ""];

		for email in invalid_emails {
			assert!(validate_email(email).is_err(), "Should fail for: {}", email);
		}
	}

	#[test]
	fn test_parse_as_f64() {
		// Valid f64 parsing
		let result: f64 = parse_as("123.456", "test_field").unwrap();
		assert_eq!(result, 123.456);

		let result: f64 = parse_as("0.001", "test_field").unwrap();
		assert_eq!(result, 0.001);

		// Invalid f64 parsing
		let result: Result<f64, _> = parse_as("not_a_number", "test_field");
		assert!(result.is_err());
	}

	#[test]
	fn test_parse_as_u128() {
		// Valid u128 parsing
		let result: u128 = parse_as("123456789", "test_field").unwrap();
		assert_eq!(result, 123456789u128);

		// Invalid u128 parsing
		let result: Result<u128, _> = parse_as("123.456", "test_field");
		assert!(result.is_err());

		let result: Result<u128, _> = parse_as("-100", "test_field");
		assert!(result.is_err());
	}

	#[test]
	fn test_parse_as_u32() {
		// Valid u32 parsing
		let result: u32 = parse_as("42161", "chain_id").unwrap();
		assert_eq!(result, 42161u32);

		// Invalid u32 parsing (overflow)
		let result: Result<u32, _> = parse_as("999999999999", "chain_id");
		assert!(result.is_err());
	}
}
