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

use crate::utils::paymaster::calculate_erc20_token_cost;
use crate::utils::types::GasEstimateResponse;
use crate::utils::user_op::pack_account_gas_limits;
use alloy::primitives::{Address, Bytes, U256};
use executor_primitives::ChainId;
use oe_client_aa::EntryPointClient;
use oe_client_binance::BinancePaymasterApi;
use oe_client_ethereum::AlloyRpcProvider;
use std::sync::Arc;
use tracing::{debug, info};

// Gas estimation constants
/// Maximum verification gas limit to prevent DoS attacks
pub const MAX_VERIFICATION_GAS: u128 = 3_000_000;
/// Default verification gas for testing during binary search
pub const DEFAULT_VERIFICATION_GAS_FOR_TESTING: u64 = 1_000_000;
/// Minimum transaction gas as per EIP-155
pub const MIN_TRANSACTION_GAS: u64 = 21_000;
/// Maximum reasonable gas for normal operations
pub const MAX_NORMAL_GAS: u64 = 10_000_000;
/// Minimum gas for deployment operations
pub const MIN_DEPLOYMENT_GAS: u64 = 100_000;
/// Maximum gas for deployment operations
pub const MAX_DEPLOYMENT_GAS: u64 = 20_000_000;
/// Safety buffer percentage for verification gas
pub const VERIFICATION_GAS_BUFFER_PERCENT: u64 = 20;
/// Default paymaster verification gas limit
pub const DEFAULT_PAYMASTER_VERIFICATION_GAS: u128 = 100_000;
/// Default paymaster post-operation gas limit
pub const DEFAULT_PAYMASTER_POST_OP_GAS: u128 = 50_000;
/// Maximum allowed paymaster gas to prevent abuse
pub const MAX_PAYMASTER_GAS: u128 = 5_000_000;

/// Main gas estimation function
pub async fn estimate_user_op_gas(
	entry_point_client: Arc<EntryPointClient<AlloyRpcProvider>>,
	user_op: oe_client_aa::PackedUserOperation,
	chain_id: ChainId,
	oe_client_binance: &dyn BinancePaymasterApi,
) -> Result<GasEstimateResponse, String> {
	// Step 1: Simulate validation to get base gas requirements
	let validation_result = entry_point_client
		.simulate_validation(user_op.clone())
		.await
		.map_err(|e| format!("Validation simulation failed: {:?}", e))?;

	// Extract preOpGas from validation result
	let pre_op_gas = validation_result.returnInfo.preOpGas;
	debug!("Validation simulation preOpGas: {}", pre_op_gas);

	// Step 2: Calculate verification gas limit based on validation result
	// Add buffer for safety
	let buffer_multiplier = U256::from(100 + VERIFICATION_GAS_BUFFER_PERCENT);
	let verification_gas_base = pre_op_gas.saturating_mul(buffer_multiplier) / U256::from(100);
	let verification_gas_limit = verification_gas_base.min(U256::from(MAX_VERIFICATION_GAS));

	// Step 3: Binary search for optimal call gas limit
	let call_gas_limit =
		estimate_call_gas_limit(entry_point_client.clone(), user_op.clone(), chain_id).await?;

	// Step 4: Calculate preVerificationGas (static + dynamic components)
	let (static_pvg, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);
	let pre_verification_gas = static_pvg + dynamic_pvg;

	// Step 5: Extract paymaster gas limits if paymaster is present
	let (paymaster_verification_gas_limit, paymaster_post_op_gas_limit) =
		extract_paymaster_gas_limits(&user_op.paymasterAndData);

	// Step 6: Calculate gas fees with buffer
	let (max_fee_per_gas, max_priority_fee_per_gas) = entry_point_client
		.calculate_gas_fees_with_buffer(10) // Use 10% additional buffer for estimation
		.await
		.map_err(|e| format!("Failed to calculate gas fees: {:?}", e))?;

	// Convert to u128 for response, ensuring values are within bounds
	let call_gas_limit = call_gas_limit.try_into().map_err(|_| {
		format!("Call gas limit {} exceeds maximum supported value", call_gas_limit)
	})?;

	let verification_gas_limit = verification_gas_limit.try_into().map_err(|_| {
		format!("Verification gas limit {} exceeds maximum supported value", verification_gas_limit)
	})?;

	let pre_verification_gas = pre_verification_gas.try_into().map_err(|_| {
		format!("Pre-verification gas {} exceeds maximum supported value", pre_verification_gas)
	})?;

	let max_fee_per_gas = max_fee_per_gas.try_into().map_err(|_| {
		format!("Max fee per gas {} exceeds maximum supported value", max_fee_per_gas)
	})?;

	let max_priority_fee_per_gas = max_priority_fee_per_gas.try_into().map_err(|_| {
		format!(
			"Max priority fee per gas {} exceeds maximum supported value",
			max_priority_fee_per_gas
		)
	})?;

	// Build initial response with gas estimates
	let gas_response = GasEstimateResponse {
		call_gas_limit,
		verification_gas_limit,
		pre_verification_gas,
		paymaster_verification_gas_limit,
		paymaster_post_op_gas_limit,
		max_fee_per_gas,
		max_priority_fee_per_gas,
		estimated_token_cost: None, // Will be updated if ERC20 paymaster is detected
	};

	let estimated_token_cost = if !user_op.paymasterAndData.is_empty() {
		// Step 7: Calculate token cost if ERC20 paymaster is present
		calculate_erc20_token_cost(
			oe_client_binance,
			&user_op.paymasterAndData,
			&gas_response,
			&user_op,
			chain_id,
		)
		.await
	} else {
		None
	};

	// Update response with token cost estimate
	let final_response = GasEstimateResponse {
		call_gas_limit,
		verification_gas_limit,
		pre_verification_gas,
		paymaster_verification_gas_limit,
		paymaster_post_op_gas_limit,
		max_fee_per_gas,
		max_priority_fee_per_gas,
		estimated_token_cost,
	};

	info!("Gas estimation complete: {:?}", final_response);
	Ok(final_response)
}

/// Binary search for optimal call gas limit following Rundler's approach
async fn estimate_call_gas_limit(
	entry_point_client: Arc<EntryPointClient<AlloyRpcProvider>>,
	user_op: oe_client_aa::PackedUserOperation,
	chain_id: ChainId,
) -> Result<U256, String> {
	// Determine gas limits based on operation type
	let (min_gas, max_gas) = if !user_op.initCode.is_empty() {
		// Deployment operation requires higher gas limits
		(U256::from(MIN_DEPLOYMENT_GAS), U256::from(MAX_DEPLOYMENT_GAS))
	} else {
		// Normal operation
		(U256::from(MIN_TRANSACTION_GAS), U256::from(MAX_NORMAL_GAS))
	};

	// Step 1: Initial simulation at maximum to get baseline gas usage
	let mut test_user_op = user_op.clone();
	let verification_gas = U256::from(DEFAULT_VERIFICATION_GAS_FOR_TESTING);
	test_user_op.accountGasLimits =
		pack_account_gas_limits(verification_gas.to::<u128>(), max_gas.to::<u128>());

	let initial_result = entry_point_client
		.simulate_handle_ops(&vec![test_user_op], Address::ZERO)
		.await
		.map_err(|e| format!("Initial simulation failed: {:?}", e))?;

	if initial_result.is_empty() || !initial_result[0].targetSuccess {
		return Err("UserOperation validation failed at maximum gas limit".to_string());
	}

	// Extract actual gas used from the successful simulation
	let initial_gas_used = initial_result[0].paid;

	// Step 2: Set initial guess as 2x the gas used (accounts for 63/64ths rule)
	let initial_guess = initial_gas_used.saturating_mul(U256::from(2)).min(max_gas);

	info!("Initial gas simulation: used={}, initial_guess={}", initial_gas_used, initial_guess);

	// Step 3: Binary search with 10% tolerance (following Rundler's approach)
	let mut lower_bound = min_gas;
	let mut upper_bound = initial_guess;
	let mut iterations = 0;
	const MAX_ITERATIONS: u32 = 20;
	const TOLERANCE_PERCENT: u64 = 10; // Stop when bounds are within 10%

	while iterations < MAX_ITERATIONS {
		// Check if we've converged within tolerance
		if lower_bound > U256::ZERO {
			let range = upper_bound - lower_bound;
			let tolerance_threshold = lower_bound / U256::from(TOLERANCE_PERCENT);

			if range <= tolerance_threshold {
				debug!(
					"Binary search converged after {} iterations: range={}, threshold={}",
					iterations, range, tolerance_threshold
				);
				break;
			}
		}

		let mid_gas = (lower_bound + upper_bound) / U256::from(2);

		// Test if this gas limit works
		let mut test_user_op = user_op.clone();
		test_user_op.accountGasLimits =
			pack_account_gas_limits(verification_gas.to::<u128>(), mid_gas.to::<u128>());

		let test_result =
			entry_point_client.simulate_handle_ops(&vec![test_user_op], Address::ZERO).await;

		match test_result {
			Ok(results) if !results.is_empty() && results[0].targetSuccess => {
				// Simulation succeeded, try lower gas
				upper_bound = mid_gas;
				debug!("Binary search iteration {}: gas {} succeeded", iterations, mid_gas);
			},
			_ => {
				// Simulation failed, need more gas
				lower_bound = mid_gas + U256::from(1);
				debug!("Binary search iteration {}: gas {} failed", iterations, mid_gas);
			},
		}

		iterations += 1;
	}

	// Use the upper bound as our estimate (ensures success)
	let optimal_gas = upper_bound;

	// Add safety buffer based on chain
	let buffer_percent = match chain_id {
		1 => 50,     // Mainnet: 50% buffer
		42161 => 30, // Arbitrum: 30% buffer
		8453 => 30,  // Base: 30% buffer
		56 => 40,    // BSC: 40% buffer
		80084 => 20, // HyperEVM: 20% buffer
		_ => 40,     // Default: 40% buffer
	};

	let final_gas = optimal_gas.saturating_mul(U256::from(100 + buffer_percent)) / U256::from(100);

	info!("Call gas limit estimation: optimal={}, with_buffer={}", optimal_gas, final_gas);
	Ok(final_gas)
}

/// Calculate preVerificationGas split into static and dynamic components
fn calculate_pre_verification_gas(
	user_op: &oe_client_aa::PackedUserOperation,
	chain_id: ChainId,
) -> (U256, U256) {
	// EIP-2028 gas costs
	const GAS_PER_ZERO_BYTE: u64 = 4;
	const GAS_PER_NON_ZERO_BYTE: u64 = 16;
	const BASE_TRANSACTION_GAS: u64 = 21_000;
	const CREATE2_OVERHEAD_GAS: u64 = 32_000;
	const BUNDLE_OVERHEAD_GAS: u64 = 5_000; // Per-UserOp share of bundle transaction overhead

	// Helper function to calculate gas for bytes
	let calculate_bytes_gas = |data: &[u8]| -> U256 {
		let zero_bytes = data.iter().filter(|&&b| b == 0).count() as u64;
		let non_zero_bytes = (data.len() as u64) - zero_bytes;
		U256::from(zero_bytes * GAS_PER_ZERO_BYTE + non_zero_bytes * GAS_PER_NON_ZERO_BYTE)
	};

	// === STATIC PVG ===
	// These costs don't change based on network conditions
	let mut static_gas = U256::from(BASE_TRANSACTION_GAS + BUNDLE_OVERHEAD_GAS);

	// Calculate gas for UserOp calldata that will be included in the bundle
	static_gas += calculate_bytes_gas(&user_op.callData);
	static_gas += calculate_bytes_gas(&user_op.initCode);
	static_gas += calculate_bytes_gas(&user_op.paymasterAndData);
	static_gas += calculate_bytes_gas(&user_op.signature);

	// Add gas for fixed-size fields
	// sender (address as bytes20 padded to bytes32)
	static_gas += U256::from(20 * GAS_PER_NON_ZERO_BYTE + 12 * GAS_PER_ZERO_BYTE);
	// nonce (usually has many zero bytes)
	static_gas += U256::from(32 * GAS_PER_ZERO_BYTE);
	// accountGasLimits (bytes32)
	static_gas += U256::from(16 * GAS_PER_NON_ZERO_BYTE + 16 * GAS_PER_ZERO_BYTE);
	// preVerificationGas (uint256)
	static_gas += U256::from(8 * GAS_PER_NON_ZERO_BYTE + 24 * GAS_PER_ZERO_BYTE);
	// gasFees (bytes32)
	static_gas += U256::from(16 * GAS_PER_NON_ZERO_BYTE + 16 * GAS_PER_ZERO_BYTE);

	// Add deployment overhead if initCode is present
	if !user_op.initCode.is_empty() {
		static_gas += U256::from(CREATE2_OVERHEAD_GAS);
	}

	// === DYNAMIC PVG ===
	// L2-specific costs that can change based on L1 gas prices
	let dynamic_gas = calculate_l2_data_cost(user_op, chain_id);

	// Apply buffers
	let static_with_buffer = static_gas.saturating_mul(U256::from(110)) / U256::from(100); // 10% buffer
	let dynamic_with_buffer = if dynamic_gas > U256::ZERO {
		// L2s need higher buffer due to L1 gas price volatility
		dynamic_gas.saturating_mul(U256::from(125)) / U256::from(100) // 25% buffer for L2
	} else {
		U256::ZERO
	};

	debug!(
		"PreVerificationGas: static={}, dynamic={} (chain_id={}, total_bytes={})",
		static_with_buffer,
		dynamic_with_buffer,
		chain_id,
		user_op.callData.len() + user_op.initCode.len() + user_op.paymasterAndData.len()
	);

	(static_with_buffer, dynamic_with_buffer)
}

/// Calculate L2-specific data availability costs
fn calculate_l2_data_cost(user_op: &oe_client_aa::PackedUserOperation, chain_id: ChainId) -> U256 {
	// Check if this is an L2 network
	let is_l2 = matches!(
		chain_id,
		42161 | 421614 | // Arbitrum One, Arbitrum Sepolia
		10 | 11155420 |  // Optimism, Optimism Sepolia
		8453 | 84532 |   // Base, Base Sepolia
		137 | 80001 // Polygon, Mumbai
	);

	if !is_l2 {
		return U256::ZERO;
	}

	// Calculate total calldata size that needs to be posted to L1
	let total_bytes = user_op.callData.len()
		+ user_op.initCode.len()
		+ user_op.paymasterAndData.len()
		+ user_op.signature.len()
		+ 32 * 5; // Fixed fields

	// L2-specific multipliers (these would ideally come from an oracle)
	// These are rough estimates - production should use actual L1 gas price oracles
	let l1_data_cost_per_byte = match chain_id {
		42161 | 421614 => U256::from(140), // Arbitrum (uses Nitro compression)
		10 | 11155420 => U256::from(160),  // Optimism (uses bedrock compression)
		8453 | 84532 => U256::from(160),   // Base (same as Optimism)
		137 | 80001 => U256::from(50),     // Polygon (cheaper as sidechain)
		_ => U256::from(100),              // Default for unknown L2s
	};

	let dynamic_cost = U256::from(total_bytes) * l1_data_cost_per_byte;

	debug!(
		"L2 data cost calculation: chain_id={}, bytes={}, cost_per_byte={}, total={}",
		chain_id, total_bytes, l1_data_cost_per_byte, dynamic_cost
	);

	dynamic_cost
}

/// Extract and validate paymaster gas limits from paymasterAndData field
fn extract_paymaster_gas_limits(paymaster_and_data: &Bytes) -> (u128, u128) {
	// If no paymaster, return zeros
	if paymaster_and_data.len() < 20 {
		return (0, 0);
	}

	// paymasterAndData format:
	// [0:20] - paymaster address
	// [20:36] - paymaster verification gas limit (uint128)
	// [36:52] - paymaster post-op gas limit (uint128)
	// [52:] - paymaster data
	// check UserOperationLib.sol

	if paymaster_and_data.len() >= 52 {
		// Extract verification gas limit (bytes 20-36)
		let mut verification_bytes = [0u8; 16];
		verification_bytes.copy_from_slice(&paymaster_and_data[20..36]);
		let verification_gas_limit = u128::from_be_bytes(verification_bytes);

		// Extract post-op gas limit (bytes 36-52)
		let mut post_op_bytes = [0u8; 16];
		post_op_bytes.copy_from_slice(&paymaster_and_data[36..52]);
		let post_op_gas_limit = u128::from_be_bytes(post_op_bytes);

		// Validate gas limits to prevent abuse
		let validated_verification = if verification_gas_limit > MAX_PAYMASTER_GAS {
			debug!(
				"Paymaster verification gas {} exceeds maximum, using default",
				verification_gas_limit
			);
			DEFAULT_PAYMASTER_VERIFICATION_GAS
		} else if verification_gas_limit == 0 {
			debug!("Paymaster verification gas is zero, using default");
			DEFAULT_PAYMASTER_VERIFICATION_GAS
		} else {
			verification_gas_limit
		};

		let validated_post_op = if post_op_gas_limit > MAX_PAYMASTER_GAS {
			debug!("Paymaster post-op gas {} exceeds maximum, using default", post_op_gas_limit);
			DEFAULT_PAYMASTER_POST_OP_GAS
		} else if post_op_gas_limit == 0 {
			debug!("Paymaster post-op gas is zero, using default");
			DEFAULT_PAYMASTER_POST_OP_GAS
		} else {
			post_op_gas_limit
		};

		debug!(
			"Validated paymaster gas limits: verification={}, post_op={}",
			validated_verification, validated_post_op
		);

		(validated_verification, validated_post_op)
	} else {
		// Paymaster present but no gas limits specified, use defaults
		debug!("Paymaster present but gas limits not specified, using defaults");
		(DEFAULT_PAYMASTER_VERIFICATION_GAS, DEFAULT_PAYMASTER_POST_OP_GAS)
	}
}
