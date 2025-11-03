use crate::config::{MAX_WALLET_INDEX, SUPPORTED_EVM_CHAINS};
use crate::detailed_error::DetailedError;
use crate::RpcResult;
use alloy::primitives::Address;
use email_address::EmailAddress;
use std::str::FromStr;

pub fn validate_chain_id(chain_id: u32) -> RpcResult<()> {
	if !SUPPORTED_EVM_CHAINS.contains(&chain_id) {
		return Err(DetailedError::invalid_chain_id(chain_id as u64).to_rpc_error());
	}
	Ok(())
}

pub fn validate_wallet_index(index: u32) -> RpcResult<()> {
	if index > MAX_WALLET_INDEX {
		return Err(DetailedError::invalid_wallet_index(index, MAX_WALLET_INDEX).to_rpc_error());
	}
	Ok(())
}

pub fn validate_ethereum_address(address: &str, field_name: &str) -> RpcResult<Address> {
	Address::from_str(address).map_err(|e| {
		DetailedError::invalid_address_format(
			field_name,
			address,
			"0x-prefixed 20-byte Ethereum address (40 hex chars)",
		)
		.with_reason(format!("Parse error: {}", e))
		.to_rpc_error()
	})
}

pub fn validate_amount(amount_str: &str, field_name: &str) -> RpcResult<u128> {
	// Check if empty
	if amount_str.is_empty() {
		return Err(DetailedError::invalid_amount(
			field_name,
			amount_str,
			"Amount cannot be empty",
		)
		.to_rpc_error());
	}

	// Parse as u128
	let amount = amount_str.parse::<u128>().map_err(|e| {
		DetailedError::invalid_amount(
			field_name,
			amount_str,
			&format!("Failed to parse amount: {}", e),
		)
		.to_rpc_error()
	})?;

	// Check if zero
	if amount == 0 {
		return Err(DetailedError::invalid_amount(
			field_name,
			amount_str,
			"Amount must be greater than zero",
		)
		.to_rpc_error());
	}

	Ok(amount)
}

pub fn validate_token_address(address: &str, field_name: &str) -> RpcResult<Address> {
	// For native token transfers, address might be "0x0" or similar
	if address == "0x0" || address == "0x0000000000000000000000000000000000000000" {
		return Ok(Address::ZERO);
	}

	validate_ethereum_address(address, field_name).map_err(|_e| {
		DetailedError::new(
			crate::error_code::INVALID_TOKEN_ADDRESS_CODE,
			"Invalid token contract address",
		)
		.with_field(field_name)
		.with_received(address.to_string())
		.with_expected("Valid ERC20 token contract address or 0x0 for native token")
		.with_suggestion("Ensure the token address is correct for the selected chain")
		.to_rpc_error()
	})
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
		validate_ethereum_address(&op.sender, &format!("user_operations[{}].sender", index))?;
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
	fn test_validate_ethereum_address_valid() {
		let addresses = vec![
			"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9",
			"0x0000000000000000000000000000000000000000",
			"0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		];

		for addr in addresses {
			assert!(validate_ethereum_address(addr, "test").is_ok());
		}
	}

	#[test]
	fn test_validate_ethereum_address_invalid() {
		let invalid_addresses = vec![
			"not_an_address",
			"0x",
			"0xGGGG", // Invalid hex
			"",
		];

		for addr in invalid_addresses {
			assert!(validate_ethereum_address(addr, "test").is_err());
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
	fn test_validate_token_address_native() {
		// Native token addresses
		assert_eq!(validate_token_address("0x0", "test").unwrap(), Address::ZERO);
		assert_eq!(
			validate_token_address("0x0000000000000000000000000000000000000000", "test").unwrap(),
			Address::ZERO
		);
	}

	#[test]
	fn test_validate_token_address_erc20() {
		let erc20_addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9";
		assert!(validate_token_address(erc20_addr, "test").is_ok());
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
}
