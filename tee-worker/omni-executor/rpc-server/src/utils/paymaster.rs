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

use alloy::primitives::{Address, Bytes};
use oe_client_binance::BinancePaymasterApi;
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::utils::types::{GasEstimateResponse, TokenCostEstimate};

// ============================================================================
// ERC20 Paymaster Exchange Rate Processing
// ============================================================================

// Constants for ERC20 paymaster processing
// paymasterAndData format: paymaster_address (20) + validation_gas_limit (16) + postop_gas_limit (16) + paymaster_data
// paymaster_data: token(20) + exchangeRate(32) + validUntil(32) + validAfter(32)
pub const MIN_ERC20_PAYMASTER_DATA_LENGTH: usize = 52 + 20 + 32 + 32 + 32; // 168 bytes minimum
pub const PAYMASTER_DATA_OFFSET: usize = 52; // paymaster address (20) + validation_gas_limit (16) + postop_gas_limit (16)
pub const EXCHANGE_RATE_FEE_PERCENT: f64 = 0.0; // 0% fee for now
pub const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000; // 1e18 wei

// Whitelisted paymaster addresses that we support (deployed by heima), the logic is:
// - Unsigned userOp: If paymaster specified, **must** be whitelisted
// - Signed userOp: Only allowed if paymasterAndData is empty (technically we could still relay it, but we are a private bundler and don't want to relay arbitrary userOps)
//
// These addresses are assumed to be the same across all EVM chains
//
// Note: the SimplePaymaster should be gradually deprecated in favor of the ERC20PaymasterV1.
//       Theoretically user could construct an unsiged userOp to use SimplePaymaster "freely"
//
// TODO: add ERC20 paymaster addresses
pub const WHITELISTED_PAYMASTER_ADDRESSES: &[&str] = &[
	// SimplePaymaster
	"0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912", // staging-v1
	"0xD4dCB31763CBA7295bA4023E9411CB6db607DE07", // prod-v1
	// ERC20PaymasterV1
	"0xA8535e013236E04FAD5dc03eCc4c05A464c01f38", // staging
];

// Decode ERC20 paymaster data from paymasterAndData
// Format: paymaster_address(20) + validation_gas_limit(16) + postop_gas_limit(16) + token(20) + exchangeRate(32) + validUntil(32) + validAfter(32)
pub fn decode_erc20_paymaster_data(paymaster_and_data: &[u8]) -> Option<(Address, u128, u64, u64)> {
	if paymaster_and_data.len() < MIN_ERC20_PAYMASTER_DATA_LENGTH {
		return None;
	}

	// Extract token address from paymaster data (bytes 52-71)
	let token_address =
		Address::from_slice(&paymaster_and_data[PAYMASTER_DATA_OFFSET..PAYMASTER_DATA_OFFSET + 20]);

	// Extract exchange rate (bytes 72-103, 32 bytes, full u256 but we take as u128)
	let rate_bytes = &paymaster_and_data[PAYMASTER_DATA_OFFSET + 20..PAYMASTER_DATA_OFFSET + 52];
	let mut exchange_rate_bytes = [0u8; 16];
	exchange_rate_bytes.copy_from_slice(&rate_bytes[16..32]); // Last 16 bytes for u128
	let exchange_rate = u128::from_be_bytes(exchange_rate_bytes);

	// Extract validUntil (bytes 104-135, 32 bytes, take as u64)
	let valid_until_bytes =
		&paymaster_and_data[PAYMASTER_DATA_OFFSET + 52..PAYMASTER_DATA_OFFSET + 84];
	let mut until_bytes = [0u8; 8];
	until_bytes.copy_from_slice(&valid_until_bytes[24..32]); // Last 8 bytes for u64
	let valid_until = u64::from_be_bytes(until_bytes);

	// Extract validAfter (bytes 136-167, 32 bytes, take as u64)
	let valid_after_bytes =
		&paymaster_and_data[PAYMASTER_DATA_OFFSET + 84..PAYMASTER_DATA_OFFSET + 116];
	let mut after_bytes = [0u8; 8];
	after_bytes.copy_from_slice(&valid_after_bytes[24..32]); // Last 8 bytes for u64
	let valid_after = u64::from_be_bytes(after_bytes);

	Some((token_address, exchange_rate, valid_until, valid_after))
}

// Encode ERC20 paymaster data with updated exchange rate, preserving validUntil and validAfter
pub fn encode_erc20_paymaster_data(
	original_data: &[u8],
	new_exchange_rate: u128,
	valid_until: u64,
	valid_after: u64,
) -> Vec<u8> {
	let mut updated_data = original_data.to_vec();

	// Skip token address (20 bytes), update exchange rate (32 bytes, big-endian u128 in last 16 bytes)
	let rate_start = PAYMASTER_DATA_OFFSET + 20;
	let rate_bytes = [0u8; 16]
		.iter()
		.chain(&new_exchange_rate.to_be_bytes())
		.copied()
		.collect::<Vec<u8>>();
	updated_data[rate_start..rate_start + 32].copy_from_slice(&rate_bytes);

	// Update validUntil (32 bytes)
	let valid_until_start = rate_start + 32;
	let valid_until_bytes =
		[0u8; 24].iter().chain(&valid_until.to_be_bytes()).copied().collect::<Vec<u8>>();
	updated_data[valid_until_start..valid_until_start + 32].copy_from_slice(&valid_until_bytes);

	// Update validAfter (32 bytes)
	let valid_after_start = valid_until_start + 32;
	let valid_after_bytes =
		[0u8; 24].iter().chain(&valid_after.to_be_bytes()).copied().collect::<Vec<u8>>();
	updated_data[valid_after_start..valid_after_start + 32].copy_from_slice(&valid_after_bytes);

	updated_data
}

/// Extract paymaster address from paymasterAndData field
/// Returns None if the data is too short to contain a valid paymaster address
pub fn extract_paymaster_address(paymaster_and_data: &Bytes) -> Option<Address> {
	if paymaster_and_data.len() < 20 {
		return None;
	}

	// First 20 bytes contain the paymaster address
	Some(Address::from_slice(&paymaster_and_data[0..20]))
}

/// Check if a paymaster address is in our whitelist
/// If so, the userOp must be unsigned to allow exchange rate processing
pub fn is_whitelisted_paymaster(paymaster_address: &Address, whitelisted: &[Address]) -> bool {
	whitelisted.contains(paymaster_address)
}

/// Parse whitelisted paymaster addresses from const strings to Address types
/// Returns empty vec if any address fails to parse (defensive programming)
pub fn parse_whitelisted_paymasters() -> Vec<Address> {
	WHITELISTED_PAYMASTER_ADDRESSES
		.iter()
		.filter_map(|addr_str| addr_str.parse().ok())
		.collect()
}

// Comprehensive token mapping with expanded support
#[derive(Debug, Clone)]
pub struct TokenInfo {
	pub decimals: u8,
	pub binance_pair: &'static str,
}

// Get supported tokens - organized by chain for better scalability
pub fn get_supported_tokens() -> HashMap<(u64, &'static str), TokenInfo> {
	let mut tokens = HashMap::new();

	// Ethereum mainnet (chain_id 1)
	tokens.insert(
		(1, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);
	tokens.insert(
		(1, "0xdac17f958d2ee523a2206206994597c13d831ec7"), // USDT
		TokenInfo { decimals: 6, binance_pair: "ETHUSDT" },
	);
	tokens.insert(
		(1, "0x6b175474e89094c44da98b954eedeac495271d0f"), // DAI
		TokenInfo { decimals: 18, binance_pair: "ETHDAI" },
	);

	// Arbitrum One (chain_id 42161)
	tokens.insert(
		(42161, "0xaf88d065e77c8cc2239327c5edb3a432268e5831"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);
	tokens.insert(
		(42161, "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9"), // USDT
		TokenInfo { decimals: 6, binance_pair: "ETHUSDT" },
	);

	// Arbitrum Sepolia (chain_id 421614)
	tokens.insert(
		(421614, "0x75faf114eafb1bdbe2f0316df893fd58ce46aa4d"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);

	// BNB Smart Chain (chain_id 56)
	tokens.insert(
		(56, "0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d"), // USDC
		TokenInfo { decimals: 18, binance_pair: "ETHUSDC" },
	);
	tokens.insert(
		(56, "0x55d398326f99059ff775485246999027b3197955"), // USDT
		TokenInfo { decimals: 18, binance_pair: "ETHUSDT" },
	);

	// Base (Chain ID 8453)
	tokens.insert(
		(8453, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);

	tokens
}

// Map token address to Binance symbol and return (symbol, decimals) from hardcoded mapping
pub async fn get_token_info_from_mapping(
	token_address: &Address,
	chain_id: u64,
) -> Option<(String, u8)> {
	let tokens = get_supported_tokens();
	let address_str = token_address.to_string().to_lowercase();

	// Look up token info from our comprehensive mapping
	if let Some(token_info) = tokens.get(&(chain_id, address_str.as_str())) {
		// Use hardcoded values from our mapping
		return Some((token_info.binance_pair.to_string(), token_info.decimals));
	}

	debug!(
		"Token address {} on chain {} is not supported. Supported tokens: {:?}",
		token_address,
		chain_id,
		tokens.keys().filter(|(cid, _)| *cid == chain_id).collect::<Vec<_>>()
	);
	None
}

// Calculate exchange rate using Binance API
pub async fn calculate_exchange_rate_with_binance(
	oe_client_binance: &dyn BinancePaymasterApi,
	token_symbol: &str,
	token_decimals: u8,
) -> Result<u128, String> {
	// Query price from Binance
	// For ETHUSDC, this returns how many USDC for 1 ETH (e.g., 4479.99)
	let price_str = oe_client_binance
		.get_symbol_price(token_symbol)
		.await
		.map_err(|e| format!("Failed to get price for {}: {:?}", token_symbol, e))?;

	let tokens_per_eth: f64 = price_str
		.parse()
		.map_err(|e| format!("Failed to parse price '{}': {}", price_str, e))?;

	if tokens_per_eth <= 0.0 {
		return Err(format!("Invalid price from Binance: {}", tokens_per_eth));
	}

	// Apply fee percentage (increase rate to charge more tokens)
	let tokens_per_eth_with_fee = tokens_per_eth * (1.0 + EXCHANGE_RATE_FEE_PERCENT / 100.0);

	// Calculate exchange rate according to ERC20PaymasterV1.sol:
	// exchangeRate = tokensPerEth * 10^tokenDecimals
	// Example: For 6-decimal USDC at $4000/ETH: rate = 4000 * 10^6 = 4000000000
	// This rate is used as: requiredTokenAmount = (maxCost * exchangeRate) / 1e18
	// Where maxCost is in wei, so 1 ETH of gas = 10^18 wei
	// Result: (10^18 * 4000000000) / 10^18 = 4000000000 USDC units = 4000 USDC ✓

	let exchange_rate_f64 = tokens_per_eth_with_fee * (10_f64.powi(token_decimals as i32));

	if exchange_rate_f64 <= 0.0 || exchange_rate_f64 >= u128::MAX as f64 {
		return Err(format!("Exchange rate {} is out of valid range", exchange_rate_f64));
	}

	info!(
		"Calculated exchange rate for {}: {} tokens per ETH -> rate {} (with {}% fee)",
		token_symbol, tokens_per_eth, exchange_rate_f64 as u128, EXCHANGE_RATE_FEE_PERCENT
	);

	Ok(exchange_rate_f64 as u128)
}

// Process ERC20 paymaster data
pub async fn process_erc20_paymaster_data(
	oe_client_binance: &dyn BinancePaymasterApi,
	paymaster_and_data: &Bytes,
	chain_id: u64,
) -> Result<Option<Bytes>, String> {
	let data_bytes = paymaster_and_data.as_ref();

	// Try to decode as ERC20 paymaster data
	if let Some((token_address, original_rate, valid_until, valid_after)) =
		decode_erc20_paymaster_data(data_bytes)
	{
		info!(
			"Detected ERC20 paymaster with token {} (original rate: {}, valid_until: {}, valid_after: {})",
			token_address, original_rate, valid_until, valid_after
		);

		// Get token symbol and decimals from hardcoded mapping
		let (token_symbol, token_decimals) = match get_token_info_from_mapping(
			&token_address,
			chain_id,
		)
		.await
		{
			Some((symbol, decimals)) => (symbol, decimals),
			None => {
				return Err(format!(
						"Token address {} on chain {} is not supported for price queries. Consider adding it to the supported tokens list or ensure it has a trading pair on Binance.",
						token_address, chain_id
					));
			},
		};

		// Calculate new exchange rate
		let new_exchange_rate =
			calculate_exchange_rate_with_binance(oe_client_binance, &token_symbol, token_decimals)
				.await?;

		// Encode updated paymaster data with new exchange rate, keeping original timestamps
		let updated_paymaster_data = encode_erc20_paymaster_data(
			data_bytes,
			new_exchange_rate,
			valid_until, // Keep original validUntil
			valid_after, // Keep original validAfter
		);

		info!(
			"Updated ERC20 paymaster exchange rate: {} -> {} for token {}",
			original_rate, new_exchange_rate, token_address
		);

		Ok(Some(Bytes::from(updated_paymaster_data)))
	} else {
		// Not ERC20 paymaster data or insufficient length, return as-is
		debug!("PaymasterAndData is not ERC20 paymaster format, skipping processing");
		Ok(None)
	}
}

/// Calculate estimated token cost for ERC20 paymaster operations
pub async fn calculate_erc20_token_cost(
	oe_client_binance: &dyn BinancePaymasterApi,
	paymaster_and_data: &Bytes,
	gas_estimates: &GasEstimateResponse,
	user_op: &oe_client_aa::PackedUserOperation,
	chain_id: u64,
) -> Option<TokenCostEstimate> {
	// Decode paymaster data
	let (token_address, _, valid_until, valid_after) =
		decode_erc20_paymaster_data(paymaster_and_data.as_ref())?;

	// Check if timestamps are valid
	let current_time = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.ok()?
		.as_secs();

	if current_time < valid_after || current_time > valid_until {
		debug!("Exchange rate timestamps not valid for estimation");
		return None;
	}

	// Get token info from mapping
	let (token_symbol, token_decimals) =
		get_token_info_from_mapping(&token_address, chain_id).await?;

	// Get fresh exchange rate from Binance
	let exchange_rate = match calculate_exchange_rate_with_binance(
		oe_client_binance,
		&token_symbol,
		token_decimals,
	)
	.await
	{
		Ok(rate) => rate,
		Err(e) => {
			error!("Failed to get exchange rate for token cost estimation: {}", e);
			return None;
		},
	};

	// Extract gas limits from estimates
	let call_gas = gas_estimates.call_gas_limit;
	let verification_gas = gas_estimates.verification_gas_limit;
	let pre_verification_gas = gas_estimates.pre_verification_gas;
	let paymaster_verification_gas = gas_estimates.paymaster_verification_gas_limit;
	let paymaster_post_op_gas = gas_estimates.paymaster_post_op_gas_limit;

	// Extract gas price from the UserOp's gasFees field
	// gasFees format: maxFeePerGas (16 bytes) | maxPriorityFeePerGas (16 bytes)
	let gas_fees_bytes = user_op.gasFees.0;
	let mut max_fee_bytes = [0u8; 16];
	max_fee_bytes.copy_from_slice(&gas_fees_bytes[0..16]); // First 16 bytes for maxFeePerGas
	let max_fee_per_gas = u128::from_be_bytes(max_fee_bytes);

	// Calculate total gas cost in wei
	// Include all gas components including paymaster overhead
	let total_gas = call_gas
		+ verification_gas
		+ pre_verification_gas
		+ paymaster_verification_gas
		+ paymaster_post_op_gas;

	let total_cost_wei = total_gas.saturating_mul(max_fee_per_gas);

	// Calculate token amount using same formula as the contract with ceil rounding
	// tokenAmount = ceil((gasCost * exchangeRate) / 1e18)
	let numerator = total_cost_wei.saturating_mul(exchange_rate);
	let token_amount = if numerator == 0 {
		0
	} else {
		numerator.saturating_add(WEI_PER_ETH - 1).saturating_div(WEI_PER_ETH)
	};

	// Add 10% safety buffer for price fluctuations
	let token_amount_with_buffer = token_amount.saturating_mul(110).saturating_div(100);

	Some(TokenCostEstimate {
		token_address: token_address.to_string(),
		amount: token_amount_with_buffer,
		decimals: token_decimals,
		exchange_rate,
	})
}
