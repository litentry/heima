use crate::config::{MAX_WALLET_INDEX, SUPPORTED_EVM_CHAINS};
use crate::detailed_error::DetailedError;
use alloy::primitives::Address;
use email_address::EmailAddress;
use std::str::FromStr;

pub fn validate_chain_id(
	chain_id: u32,
	chain_type: Option<&str>,
) -> Result<(), Box<DetailedError>> {
	let is_valid = match chain_type {
		Some("evm") => SUPPORTED_EVM_CHAINS.contains(&chain_id),
		_ => SUPPORTED_EVM_CHAINS.contains(&chain_id),
	};

	if !is_valid {
		let supported_chains: Vec<u64> = SUPPORTED_EVM_CHAINS.iter().map(|&c| c as u64).collect();
		return Err(Box::new(DetailedError::invalid_chain_id(chain_id as u64, &supported_chains)));
	}
	Ok(())
}

pub fn validate_wallet_index(index: u32) -> Result<(), Box<DetailedError>> {
	if index > MAX_WALLET_INDEX {
		return Err(Box::new(DetailedError::invalid_wallet_index(index, MAX_WALLET_INDEX)));
	}
	Ok(())
}

pub fn validate_ethereum_address(
	address: &str,
	field_name: &str,
) -> Result<Address, Box<DetailedError>> {
	Address::from_str(address).map_err(|e| {
		Box::new(
			DetailedError::invalid_address_format(
				field_name,
				address,
				"0x-prefixed 20-byte Ethereum address (40 hex chars)",
			)
			.with_reason(format!("Parse error: {}", e)),
		)
	})
}

pub fn validate_omni_account_hex(
	hex_str: &str,
	field_name: &str,
) -> Result<Vec<u8>, Box<DetailedError>> {
	let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

	hex::decode(hex_str).map_err(|e| {
		Box::new(
			DetailedError::invalid_hex_format(field_name, hex_str, Some(32))
				.with_reason(format!("Hex decode error: {}", e)),
		)
	})
}

pub fn validate_omni_account_length(
	bytes: &[u8],
	field_name: &str,
) -> Result<(), Box<DetailedError>> {
	if bytes.len() != 32 {
		return Err(Box::new(
			DetailedError::new(
				crate::error_code::INVALID_ACCOUNT_LENGTH_CODE,
				"Invalid account length",
			)
			.with_field(field_name)
			.with_expected("32 bytes")
			.with_received(format!("{} bytes", bytes.len()))
			.with_suggestion("Omni accounts must be exactly 32 bytes"),
		));
	}
	Ok(())
}

pub fn validate_amount(amount_str: &str, field_name: &str) -> Result<u128, Box<DetailedError>> {
	// Check if empty
	if amount_str.is_empty() {
		return Err(Box::new(DetailedError::invalid_amount(
			field_name,
			amount_str,
			"Amount cannot be empty",
		)));
	}

	// Parse as u128
	let amount = amount_str.parse::<u128>().map_err(|e| {
		Box::new(DetailedError::invalid_amount(
			field_name,
			amount_str,
			&format!("Failed to parse amount: {}", e),
		))
	})?;

	// Check if zero
	if amount == 0 {
		return Err(Box::new(DetailedError::invalid_amount(
			field_name,
			amount_str,
			"Amount must be greater than zero",
		)));
	}

	Ok(amount)
}

pub fn validate_token_address(
	address: &str,
	field_name: &str,
) -> Result<Address, Box<DetailedError>> {
	// For native token transfers, address might be "0x0" or similar
	if address == "0x0" || address == "0x0000000000000000000000000000000000000000" {
		return Ok(Address::ZERO);
	}

	validate_ethereum_address(address, field_name).map_err(|_e| {
		Box::new(
			DetailedError::new(
				crate::error_code::INVALID_TOKEN_ADDRESS_CODE,
				"Invalid token contract address",
			)
			.with_field(field_name)
			.with_received(address.to_string())
			.with_expected("Valid ERC20 token contract address or 0x0 for native token")
			.with_suggestion("Ensure the token address is correct for the selected chain"),
		)
	})
}

pub fn validate_email(email: &str) -> Result<(), Box<DetailedError>> {
	if !EmailAddress::is_valid(email) {
		return Err(Box::new(
			DetailedError::new(
				crate::error_code::INVALID_EMAIL_FORMAT_CODE,
				"Invalid email format",
			)
			.with_field("email")
			.with_received(email.to_string())
			.with_expected("Valid email address (e.g., user@example.com)")
			.with_suggestion("Please provide a valid email address"),
		));
	}

	// Additionally require a TLD (at least one dot after @)
	if let Some(at_pos) = email.find('@') {
		let domain = &email[at_pos + 1..];
		if !domain.contains('.') {
			return Err(Box::new(
				DetailedError::new(
					crate::error_code::INVALID_EMAIL_FORMAT_CODE,
					"Invalid email format",
				)
				.with_field("email")
				.with_received(email.to_string())
				.with_expected("Email address with a valid domain (e.g., user@example.com)")
				.with_suggestion("Email domain must include a top-level domain (TLD)"),
			));
		}
	}

	Ok(())
}

#[allow(dead_code)]
pub fn validate_hex_string(
	hex_str: &str,
	field_name: &str,
	expected_bytes: Option<usize>,
) -> Result<Vec<u8>, Box<DetailedError>> {
	let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

	let bytes = hex::decode(hex_str).map_err(|e| {
		Box::new(
			DetailedError::invalid_hex_format(field_name, hex_str, expected_bytes)
				.with_reason(format!("Hex decode error: {}", e)),
		)
	})?;

	if let Some(expected) = expected_bytes {
		if bytes.len() != expected {
			return Err(Box::new(
				DetailedError::invalid_hex_format(field_name, hex_str, Some(expected))
					.with_received(format!("{} bytes", bytes.len())),
			));
		}
	}

	Ok(bytes)
}

pub fn validate_user_operations(
	operations: &[executor_core::types::SerializablePackedUserOperation],
) -> Result<(), Box<DetailedError>> {
	if operations.is_empty() {
		return Err(Box::new(
			DetailedError::new(
				crate::error_code::MISSING_REQUIRED_FIELD_CODE,
				"User operations cannot be empty",
			)
			.with_field("user_operations")
			.with_expected("At least one user operation")
			.with_suggestion("Provide at least one user operation to submit"),
		));
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
		assert!(validate_chain_id(1, Some("evm")).is_ok());
		assert!(validate_chain_id(137, Some("evm")).is_ok());
		assert!(validate_chain_id(42161, Some("evm")).is_ok());
		assert!(validate_chain_id(10, Some("evm")).is_ok());
		assert!(validate_chain_id(8453, Some("evm")).is_ok());
		assert!(validate_chain_id(56, Some("evm")).is_ok());

		// Valid EVM chain IDs - Testnets
		assert!(validate_chain_id(11155111, Some("evm")).is_ok());
		assert!(validate_chain_id(421614, Some("evm")).is_ok());
		assert!(validate_chain_id(11155420, Some("evm")).is_ok());
		assert!(validate_chain_id(80001, Some("evm")).is_ok());
		assert!(validate_chain_id(84532, Some("evm")).is_ok());
		assert!(validate_chain_id(97, Some("evm")).is_ok());
		assert!(validate_chain_id(1337, Some("evm")).is_ok());
	}

	#[test]
	fn test_validate_chain_id_evm_invalid() {
		// Invalid EVM chain IDs
		assert!(validate_chain_id(999, Some("evm")).is_err());
		assert!(validate_chain_id(5, Some("evm")).is_err());
		assert!(validate_chain_id(421613, Some("evm")).is_err());
		assert!(validate_chain_id(84531, Some("evm")).is_err());
	}

	#[test]
	fn test_validate_chain_id_any_type() {
		// Any type should check EVM chains
		assert!(validate_chain_id(1, None).is_ok()); // Ethereum Mainnet
		assert!(validate_chain_id(11155111, None).is_ok()); // Sepolia
		assert!(validate_chain_id(1337, None).is_ok()); // Local Anvil
		assert!(validate_chain_id(999, None).is_err()); // Invalid chain
		assert!(validate_chain_id(5, None).is_err());
	}

	#[test]
	fn test_validate_chain_id_no_memory_leak() {
		// This test ensures the memory leak fix is working
		// Previously this would leak memory on every call with None chain_type
		for _ in 0..100 {
			let _ = validate_chain_id(999, None);
		}
		// If memory leak was present, this would cause issues in valgrind/miri
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
		assert!(validate_wallet_index(9999).is_err());
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
	fn test_validate_omni_account_hex_valid() {
		let hex_32_bytes = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
		let result = validate_omni_account_hex(hex_32_bytes, "test");
		assert!(result.is_ok());
		assert_eq!(result.unwrap().len(), 32);
	}

	#[test]
	fn test_validate_omni_account_hex_without_prefix() {
		let hex_32_bytes = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
		let result = validate_omni_account_hex(hex_32_bytes, "test");
		assert!(result.is_ok());
		assert_eq!(result.unwrap().len(), 32);
	}

	#[test]
	fn test_validate_omni_account_hex_invalid() {
		let invalid_hex = "0xGGGG567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
		assert!(validate_omni_account_hex(invalid_hex, "test").is_err());
	}

	#[test]
	fn test_validate_omni_account_length_valid() {
		let bytes_32 = vec![0u8; 32];
		assert!(validate_omni_account_length(&bytes_32, "test").is_ok());
	}

	#[test]
	fn test_validate_omni_account_length_invalid() {
		let bytes_31 = vec![0u8; 31];
		let bytes_33 = vec![0u8; 33];
		assert!(validate_omni_account_length(&bytes_31, "test").is_err());
		assert!(validate_omni_account_length(&bytes_33, "test").is_err());
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

	#[test]
	fn test_validate_hex_string_valid() {
		let hex_8_bytes = "0x1234567890abcdef";
		let result = validate_hex_string(hex_8_bytes, "test", Some(8));
		assert!(result.is_ok());
		assert_eq!(result.unwrap().len(), 8);
	}

	#[test]
	fn test_validate_hex_string_wrong_length() {
		let hex_8_bytes = "0x1234567890abcdef";
		// Expecting 10 bytes but providing 8
		assert!(validate_hex_string(hex_8_bytes, "test", Some(10)).is_err());
	}

	#[test]
	fn test_validate_hex_string_no_length_requirement() {
		let hex_any = "0x12345678";
		let result = validate_hex_string(hex_any, "test", None);
		assert!(result.is_ok());
		assert_eq!(result.unwrap().len(), 4);
	}
}
