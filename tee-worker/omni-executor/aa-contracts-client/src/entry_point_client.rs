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

use crate::error::{AaContractError, ContractError, RpcError};
use crate::types::{
	createAccountCall, depositToCall, getSenderAddressCall, getUserOpHashCall, handleOpsCall,
	simulateHandleOpsCall, simulateValidationCall, ExecutionResult, FailedOp, OwnerType,
	SenderAddressResult, ValidationResult,
};
use crate::utils::{
	build_call_transaction, build_payable_transaction, calculate_omni_account_address,
};
use crate::{OmniAccountClient, PackedUserOperation};
use alloy::hex;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::rpc::types::state::AccountOverride;
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::{SolCall, SolError, SolValue};
use ethereum_rpc::{RpcProvider, RpcProviderError};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// EntryPointSimulations deployed bytecode embedded at compile time
const SIMULATION_BYTECODE: &str =
	include_str!("bytecode/EntryPointSimulations_deployed_bytecode.hex");

/// Gas price configuration for EIP-1559 transactions
#[derive(Clone, Debug)]
pub struct GasPriceConfig {
	/// Buffer percentage to add to current gas price (e.g., 50 for 50% buffer)
	pub gas_price_buffer_percent: u64,
	/// Minimum priority fee in wei (default: 1 gwei)
	pub min_priority_fee: u128,
	/// Maximum priority fee in wei (default: 50 gwei)
	pub max_priority_fee: u128,
}

impl GasPriceConfig {
	/// Validate that the configuration values are reasonable
	pub fn validate(&self) -> Result<(), String> {
		if self.min_priority_fee > self.max_priority_fee {
			return Err("min_priority_fee cannot be greater than max_priority_fee".to_string());
		}

		if self.gas_price_buffer_percent > 500 {
			return Err(
				"gas_price_buffer_percent should not exceed 500% to prevent excessive fees"
					.to_string(),
			);
		}

		// Warn about very high max priority fees (>500 gwei)
		if self.max_priority_fee > 500_000_000_000 {
			tracing::warn!("max_priority_fee is very high ({} gwei), this may result in expensive transactions", self.max_priority_fee / 1_000_000_000);
		}

		// Ensure minimum priority fee is not zero (could cause stuck transactions)
		if self.min_priority_fee == 0 {
			return Err(
				"min_priority_fee should not be zero to prevent stuck transactions".to_string()
			);
		}

		Ok(())
	}
}

impl Default for GasPriceConfig {
	fn default() -> Self {
		Self {
			gas_price_buffer_percent: 50,     // 50% buffer
			min_priority_fee: 1_000_000_000,  // 1 gwei
			max_priority_fee: 50_000_000_000, // 50 gwei
		}
	}
}

impl GasPriceConfig {
	/// Configuration for Ethereum mainnet
	pub fn mainnet() -> Self {
		Self {
			gas_price_buffer_percent: 50,
			min_priority_fee: 2_000_000_000,   // 2 gwei
			max_priority_fee: 100_000_000_000, // 100 gwei
		}
	}

	/// Configuration for L2 networks (Arbitrum, Optimism, Base)
	pub fn l2() -> Self {
		Self {
			gas_price_buffer_percent: 30,
			min_priority_fee: 100_000_000,   // 0.1 gwei
			max_priority_fee: 2_000_000_000, // 2 gwei
		}
	}

	/// Configuration for BSC
	pub fn bsc() -> Self {
		Self {
			gas_price_buffer_percent: 40,
			min_priority_fee: 1_000_000_000,  // 1 gwei
			max_priority_fee: 10_000_000_000, // 10 gwei
		}
	}

	/// Configuration for HyperEVM
	pub fn hyperevm() -> Self {
		Self {
			gas_price_buffer_percent: 20,    // Lower buffer due to stable, low fees
			min_priority_fee: 10_000_000,    // 0.01 gwei
			max_priority_fee: 1_000_000_000, // 1 gwei
		}
	}

	/// Chain-specific gas price configuration
	pub fn for_chain(chain_id: u64) -> Self {
		match chain_id {
			1 => Self::mainnet(),    // Ethereum mainnet
			137 => Self::l2(),       // Polygon
			80001 => Self::l2(),     // Polygon Mumbai
			42161 => Self::l2(),     // Arbitrum One
			421614 => Self::l2(),    // Arbitrum Sepolia
			10 => Self::l2(),        // Optimism
			11155420 => Self::l2(),  // Optimism Sepolia
			8453 => Self::l2(),      // Base
			84532 => Self::l2(),     // Base Sepolia
			56 => Self::bsc(),       // BSC
			97 => Self::bsc(),       // BSC Testnet
			999 => Self::hyperevm(), // HyperEVM
			998 => Self::hyperevm(), // HyperEVM Testnet
			1337 => Self::l2(),      // Local Anvil
			// Ethereum testnets use mainnet config but with lower values
			11155111 => Self {
				// Sepolia
				gas_price_buffer_percent: 30,
				min_priority_fee: 1_000_000_000,  // 1 gwei
				max_priority_fee: 20_000_000_000, // 20 gwei
			},
			17000 => Self {
				// Holesky
				gas_price_buffer_percent: 30,
				min_priority_fee: 1_000_000_000,  // 1 gwei
				max_priority_fee: 20_000_000_000, // 20 gwei
			},
			_ => Self::default(),
		}
	}
}

/// Retry configuration for transaction submission
#[derive(Clone, Debug)]
pub struct RetryConfig {
	/// Maximum number of retry attempts
	pub max_attempts: u8,
	/// Initial delay between retries in milliseconds
	pub initial_delay_ms: u64,
	/// Maximum delay between retries in milliseconds
	pub max_delay_ms: u64,
	/// Gas price increase percentage for each retry
	pub gas_increase_percent: u16,
}

impl Default for RetryConfig {
	fn default() -> Self {
		Self {
			max_attempts: 3,
			initial_delay_ms: 2000,   // 2 seconds
			max_delay_ms: 30000,      // 30 seconds
			gas_increase_percent: 10, // 10% increase per retry
		}
	}
}

impl RetryConfig {
	/// Configuration for Ethereum mainnet
	pub fn mainnet() -> Self {
		Self {
			max_attempts: 5,
			initial_delay_ms: 3000,
			max_delay_ms: 60000,
			gas_increase_percent: 15,
		}
	}

	/// Configuration for L2 networks
	pub fn l2() -> Self {
		Self {
			max_attempts: 4,
			initial_delay_ms: 1000,
			max_delay_ms: 20000,
			gas_increase_percent: 20, // L2s can be more volatile
		}
	}

	/// Configuration for BSC
	pub fn bsc() -> Self {
		Self {
			max_attempts: 4,
			initial_delay_ms: 2000,
			max_delay_ms: 30000,
			gas_increase_percent: 12,
		}
	}

	/// Configuration for HyperEVM
	pub fn hyperevm() -> Self {
		Self {
			max_attempts: 2,         // Fewer retries due to fast finality (0.2s blocks)
			initial_delay_ms: 500,   // Short delay due to 1-2s transaction completion
			max_delay_ms: 5000,      // Max 5s delay given the fast network
			gas_increase_percent: 5, // Small increase due to stable, low fees
		}
	}

	/// Chain-specific configuration
	pub fn for_chain(chain_id: u64) -> Self {
		match chain_id {
			1 => Self::mainnet(),    // Ethereum mainnet
			11155111 => Self::l2(),  // Sepolia
			137 => Self::l2(),       // Polygon
			80001 => Self::l2(),     // Polygon Mumbai
			42161 => Self::l2(),     // Arbitrum
			421614 => Self::l2(),    // Arbitrum Sepolia
			10 => Self::l2(),        // Optimism
			11155420 => Self::l2(),  // Optimism Sepolia
			8453 => Self::l2(),      // Base
			84532 => Self::l2(),     // Base Sepolia
			56 => Self::bsc(),       // BSC
			999 => Self::hyperevm(), // HyperEVM
			998 => Self::hyperevm(), // HyperEVM Testnet
			1337 => Self::l2(),      // Local Anvil
			_ => Self::default(),
		}
	}
}

/// Client for interacting with on-chain EntryPoint instance
pub struct EntryPointClient<P: RpcProvider<Transaction = TransactionRequest>> {
	entry_point_address: Address,
	rpc_client: Arc<P>,
	gas_config: GasPriceConfig,
	retry_config: RetryConfig,
}

impl<P: RpcProvider<Transaction = TransactionRequest, Addr = Address>> EntryPointClient<P> {
	pub fn new(entry_point_address: Address, rpc_client: Arc<P>) -> Self {
		Self {
			entry_point_address,
			rpc_client,
			gas_config: GasPriceConfig::default(),
			retry_config: RetryConfig::default(),
		}
	}

	pub fn new_with_config(
		entry_point_address: Address,
		rpc_client: Arc<P>,
		gas_config: GasPriceConfig,
		retry_config: RetryConfig,
	) -> Self {
		// Validate gas configuration
		if let Err(e) = gas_config.validate() {
			tracing::error!("Invalid gas configuration: {}", e);
			panic!("Invalid gas configuration: {}", e);
		}

		Self { entry_point_address, rpc_client, gas_config, retry_config }
	}

	pub fn entry_point_address(&self) -> Address {
		self.entry_point_address
	}

	pub async fn get_wallet_address(&self) -> Result<Address, ()> {
		self.rpc_client.get_wallet_address().await.map_err(|_| ())
	}

	/// Calculate dynamic gas fees based on current network conditions
	async fn calculate_gas_fees(&self) -> Result<(U256, U256), ()> {
		// Try EIP-1559 estimation first
		match self.rpc_client.estimate_eip1559_fees().await {
			Ok(eip1559_estimate) => {
				// Use EIP-1559 fees with buffer
				let buffer_multiplier = 100 + self.gas_config.gas_price_buffer_percent;

				let max_fee_per_gas = U256::from(eip1559_estimate.max_fee_per_gas)
					.saturating_mul(U256::from(buffer_multiplier))
					.checked_div(U256::from(100))
					.unwrap_or(U256::from(eip1559_estimate.max_fee_per_gas));

				// Use the EIP-1559 priority fee with bounds
				let mut priority_fee = U256::from(eip1559_estimate.max_priority_fee_per_gas)
					.max(U256::from(self.gas_config.min_priority_fee))
					.min(U256::from(self.gas_config.max_priority_fee));

				// Ensure priority fee doesn't exceed max fee per gas (EIP-1559 requirement)
				if priority_fee > max_fee_per_gas {
					tracing::warn!(
						"Priority fee ({} gwei) exceeds max fee per gas ({} gwei), capping to max fee",
						priority_fee / U256::from(1_000_000_000),
						max_fee_per_gas / U256::from(1_000_000_000)
					);
					priority_fee = max_fee_per_gas;
				}

				tracing::debug!(
					"EIP-1559 gas fees calculated: max_fee={} wei ({} gwei), priority_fee={} wei ({} gwei), buffer={}%",
					max_fee_per_gas,
					max_fee_per_gas / U256::from(1_000_000_000),
					priority_fee,
					priority_fee / U256::from(1_000_000_000),
					self.gas_config.gas_price_buffer_percent
				);

				Ok((max_fee_per_gas, priority_fee))
			},
			Err(_) => {
				// Fallback to legacy gas price calculation
				// Get current gas price from network
				let current_gas_price = self
					.rpc_client
					.get_gas_price()
					.await
					.map_err(|_| error!("Failed to fetch gas price"))?;

				// Apply buffer to current gas price for max_fee_per_gas
				let buffer_multiplier = 100 + self.gas_config.gas_price_buffer_percent;
				let max_fee_per_gas = U256::from(current_gas_price)
					.saturating_mul(U256::from(buffer_multiplier))
					.checked_div(U256::from(100))
					.unwrap_or(U256::from(current_gas_price));

				// Calculate priority fee (tip) with bounds
				// Use 10% of current gas price as priority fee, bounded by min/max
				let mut priority_fee = U256::from(current_gas_price / 10)
					.max(U256::from(self.gas_config.min_priority_fee))
					.min(U256::from(self.gas_config.max_priority_fee));

				// Ensure priority fee doesn't exceed max fee per gas (EIP-1559 requirement)
				if priority_fee > max_fee_per_gas {
					tracing::warn!(
						"Priority fee ({} gwei) exceeds max fee per gas ({} gwei), capping to max fee",
						priority_fee / U256::from(1_000_000_000),
						max_fee_per_gas / U256::from(1_000_000_000)
					);
					priority_fee = max_fee_per_gas;
				}

				tracing::debug!(
					"Legacy gas fees calculated: max_fee={} wei ({} gwei), priority_fee={} wei ({} gwei), buffer={}%, base_price={} gwei",
					max_fee_per_gas,
					max_fee_per_gas / U256::from(1_000_000_000),
					priority_fee,
					priority_fee / U256::from(1_000_000_000),
					self.gas_config.gas_price_buffer_percent,
					current_gas_price / 1_000_000_000
				);

				Ok((max_fee_per_gas, priority_fee))
			},
		}
	}

	/// Simulate user operation validation using EntryPointSimulations contract
	/// This function uses state override to temporarily deploy the simulation contract
	pub async fn simulate_validation(
		&self,
		user_op: PackedUserOperation,
	) -> Result<ValidationResult, String> {
		// Create state override to deploy simulation contract at EntryPoint address
		let mut state_override = HashMap::new();
		state_override.insert(
			self.entry_point_address,
			AccountOverride {
				code: Some(
					hex::decode(SIMULATION_BYTECODE.trim())
						.map_err(|e| {
							let error_msg =
								format!("Could not decode simulation bytecode: {:?}", e);
							error!("{}", error_msg);
							error_msg
						})?
						.into(),
				),
				..Default::default()
			},
		);

		// Build call to simulateValidation
		let call_data = simulateValidationCall { userOp: user_op }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);

		// Make the call with state override
		// EntryPointSimulations.simulateValidation() returns ValidationResult on success
		match self.rpc_client.call_with_state_override(tx, state_override).await {
			Ok(result) => {
				// Decode the ValidationResult from the successful response
				ValidationResult::abi_decode(&result).map_err(|e| {
					let error_msg =
						format!("Could not decode ValidationResult from response: {:?}", e);
					error!("{}", error_msg);
					error_msg
				})
			},
			Err(error) => {
				match error {
					RpcProviderError::ExecutionReverted { reason, data } => {
						match data {
							Some(data) => {
								// Decode FailedOp from revert data
								let failed_op: FailedOp =
									FailedOp::abi_decode(&data).map_err(|e| {
										let error_msg = format!(
											"Could not decode FailedOp from revert data: {:?}",
											e
										);
										error!("{}", error_msg);
										error_msg
									})?;
								let error_msg = format!(
									"Simulation failed, opIndex: {}, reason: {}",
									failed_op.opIndex, failed_op.reason
								);
								error!("{}", error_msg);
								Err(error_msg)
							},
							None => {
								let error_msg = format!("Simulation failed, reason: {:?}", reason);
								error!("{}", error_msg);
								Err(error_msg)
							},
						}
					},
					_ => {
						let error_msg = format!("Simulation failed: {:?}", error);
						error!("{}", error_msg);
						Err(error_msg)
					},
				}
			},
		}
	}

	pub async fn simulate_handle_ops(
		&self,
		user_ops: &[PackedUserOperation],
		beneficiary: Address,
	) -> Result<Vec<ExecutionResult>, String> {
		// Create state override to deploy simulation contract at EntryPoint address
		let mut state_override = HashMap::new();
		state_override.insert(
			self.entry_point_address,
			AccountOverride {
				code: Some(
					hex::decode(SIMULATION_BYTECODE.trim())
						.map_err(|e| {
							let error_msg =
								format!("Could not decode simulation bytecode: {:?}", e);
							error!("{}", error_msg);
							error_msg
						})?
						.into(),
				),
				..Default::default()
			},
		);

		// Build call to simulateHandleOps
		let ops = user_ops.to_vec();
		let call_data = simulateHandleOpsCall { ops, beneficiary }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);

		// Make the call with state override
		// EntryPointSimulations.simulateHandleOps() returns ExecutionResult[] on success
		match self.rpc_client.call_with_state_override(tx, state_override).await {
			Ok(result) => {
				// Decode the ExecutionResult[] from the successful response
				Vec::<ExecutionResult>::abi_decode(&result).map_err(|e| {
					let error_msg =
						format!("Could not decode ExecutionResult[] from response: {:?}", e);
					error!("{}", error_msg);
					error_msg
				})
			},
			Err(error) => {
				match error {
					RpcProviderError::ExecutionReverted { reason, data } => {
						match data {
							Some(data) => {
								// Decode FailedOp from revert data
								let failed_op: FailedOp =
									FailedOp::abi_decode(&data).map_err(|e| {
										let error_msg = format!(
											"Could not decode FailedOp from revert data: {:?}",
											e
										);
										error!("{}", error_msg);
										error_msg
									})?;
								let error_msg = format!(
									"Simulation failed, opIndex: {}, reason: {}",
									failed_op.opIndex, failed_op.reason
								);
								error!("{}", error_msg);
								Err(error_msg)
							},
							None => {
								let error_msg = format!("Simulation failed, reason: {:?}", reason);
								error!("{}", error_msg);
								Err(error_msg)
							},
						}
					},
					_ => {
						let error_msg = format!("Simulation failed: {:?}", error);
						error!("{}", error_msg);
						Err(error_msg)
					},
				}
			},
		}
	}

	pub async fn handle_ops(
		&self,
		user_ops: &[PackedUserOperation],
		beneficiary: Address,
	) -> Result<String, ()> {
		let ops = user_ops.to_vec();
		let call_data = handleOpsCall { ops, beneficiary }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);
		self.rpc_client
			.send_transaction(tx)
			.await
			.map_err(|_| error!("Could not send tx"))
	}

	/// Submit UserOperations with automatic retry and gas escalation on failure
	pub async fn handle_ops_with_retry(
		&self,
		user_ops: &[PackedUserOperation],
		beneficiary: Address,
	) -> Result<String, ()> {
		let mut attempt = 0;
		let mut gas_buffer_adjustment = 0u64;

		loop {
			// Calculate gas fees with increasing buffer on retries
			let (base_max_fee, base_priority_fee) =
				self.calculate_gas_fees_with_buffer(gas_buffer_adjustment).await.map_err(|_| {
					error!("Failed to calculate gas fees");
				})?;

			// Build transaction with adjusted gas
			let ops = user_ops.to_vec();
			let call_data = handleOpsCall { ops, beneficiary }.abi_encode();
			let mut tx = build_call_transaction(self.entry_point_address, call_data);

			// Apply gas fees to transaction
			tx.max_fee_per_gas = Some(base_max_fee.to::<u128>());
			tx.max_priority_fee_per_gas = Some(base_priority_fee.to::<u128>());

			// Attempt to send transaction
			match self.rpc_client.send_transaction(tx).await {
				Ok(tx_hash) => {
					if attempt > 0 {
						info!(
							"Transaction succeeded after {} retries with gas buffer {}%",
							attempt, gas_buffer_adjustment
						);
					}
					return Ok(tx_hash);
				},
				Err(rpc_error) => {
					let error = self.classify_rpc_error(rpc_error);

					if !error.is_retryable() {
						error!("Non-retryable error encountered: {:?}", error);
						return Err(());
					}

					attempt += 1;
					if attempt >= self.retry_config.max_attempts {
						error!("Transaction failed after {} attempts", attempt);
						return Err(());
					}

					let delay = self.calculate_backoff_delay(attempt);

					gas_buffer_adjustment += self.retry_config.gas_increase_percent as u64;

					warn!(
						"Transaction failed (attempt {}/{}), retrying with {}% higher gas after {}ms",
						attempt,
						self.retry_config.max_attempts,
						gas_buffer_adjustment,
						delay
					);

					tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
				},
			}
		}
	}

	/// Calculate exponential backoff delay with jitter
	fn calculate_backoff_delay(&self, attempt: u8) -> u64 {
		let base_delay = self.retry_config.initial_delay_ms;
		let exponential_delay = base_delay.saturating_mul(2u64.pow(attempt as u32 - 1));
		let capped_delay = exponential_delay.min(self.retry_config.max_delay_ms);

		// Add jitter (±10%) to prevent thundering herd
		let jitter_range = capped_delay / 10;
		let jitter = (rand::random::<u64>() % (2 * jitter_range)).saturating_sub(jitter_range);

		capped_delay.saturating_add(jitter)
	}

	pub async fn get_sender_address(&self, init_code: Bytes) -> Result<Address, ()> {
		let call_data = getSenderAddressCall { initCode: init_code }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);
		match self.rpc_client.call(tx).await {
			Err(err) => {
				if let ethereum_rpc::RpcProviderError::ExecutionReverted { reason, .. } = &err {
					if reason.contains("0x") {
						if let Some(start) = reason.find("0x") {
							let hex_data = &reason[start..];
							if let Ok(revert_data) = hex::decode(&hex_data[2..]) {
								let result = SenderAddressResult::abi_decode(&revert_data)
									.map_err(|_| error!("Could not decode SenderAddressResult"))?;
								return Ok(result.sender);
							}
						}
					}
				}
				error!("Failed to get sender address: {:?}", err);
				Err(())
			},
			Ok(_) => Err(()),
		}
	}

	/// Calculate sender address locally using CREATE2 without calling EntryPoint
	/// This is more efficient as it doesn't require an RPC call
	pub fn calculate_sender_address(
		&self,
		factory_address: Address,
		account_implementation: Address,
		oa: FixedBytes<32>,
		oa_type: OwnerType,
		client_id: &[u8],
		root: Address,
	) -> Address {
		calculate_omni_account_address(
			factory_address,
			account_implementation,
			oa,
			oa_type,
			client_id,
			root,
		)
	}

	pub async fn deposit_to(&self, account: Address, amount: U256) -> Result<String, ()> {
		let call_data = depositToCall { account }.abi_encode();
		let tx = build_payable_transaction(self.entry_point_address, call_data, amount);
		self.rpc_client
			.send_transaction(tx)
			.await
			.map_err(|_| error!("Could not send tx"))
	}

	pub async fn get_user_op_hash(
		&self,
		user_op: PackedUserOperation,
	) -> Result<FixedBytes<32>, ()> {
		let call_data = getUserOpHashCall { userOp: user_op }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);
		let result = self.rpc_client.call(tx).await.map_err(|_| error!("Could not send tx"))?;
		let hash: FixedBytes<32> = FixedBytes::abi_decode(&result).unwrap();
		Ok(hash)
	}

	#[allow(clippy::too_many_arguments)]
	pub async fn create_packed_user_operation(
		&self,
		factory_address: Address,
		oa: [u8; 32],
		oa_type: OwnerType,
		client_id: &[u8],
		root_address: Address,
		call_data: Bytes,
		paymaster_address: Option<Address>,
	) -> Result<PackedUserOperation, ()> {
		// Create init code using existing helper
		let init_code_bytes =
			prepare_factory_init_code(factory_address, oa, oa_type, client_id, root_address);
		let init_code = Bytes::from(init_code_bytes);

		// Get sender address from EntryPoint
		let sender = self.get_sender_address(init_code).await?;

		// Use common helper for the rest
		self.build_packed_user_operation(
			sender,
			factory_address,
			oa,
			oa_type,
			client_id,
			root_address,
			call_data,
			paymaster_address,
		)
		.await
	}

	/// Create PackedUserOperation using local CREATE2 address calculation
	/// This is more efficient as it avoids the EntryPoint.getSenderAddress RPC call
	#[allow(clippy::too_many_arguments)]
	pub async fn create_packed_user_operation_with_local_address(
		&self,
		factory_address: Address,
		account_implementation: Address,
		oa: [u8; 32],
		oa_type: OwnerType,
		client_id: &[u8],
		root_address: Address,
		call_data: Bytes,
		paymaster_address: Option<Address>,
	) -> Result<PackedUserOperation, ()> {
		let oa_fixed = FixedBytes::from(oa);

		// Calculate sender address locally using CREATE2
		let sender = self.calculate_sender_address(
			factory_address,
			account_implementation,
			oa_fixed,
			oa_type,
			client_id,
			root_address,
		);

		// Use common helper for the rest
		self.build_packed_user_operation(
			sender,
			factory_address,
			oa,
			oa_type,
			client_id,
			root_address,
			call_data,
			paymaster_address,
		)
		.await
	}

	/// Common helper to build PackedUserOperation with shared logic
	#[allow(clippy::too_many_arguments)]
	async fn build_packed_user_operation(
		&self,
		sender: Address,
		factory_address: Address,
		oa: [u8; 32],
		oa_type: OwnerType,
		client_id: &[u8],
		root_address: Address,
		call_data: Bytes,
		paymaster_address: Option<Address>,
	) -> Result<PackedUserOperation, ()> {
		// Check if the account already has code deployed
		let code = self.rpc_client.get_code_at(sender).await.map_err(|_| ())?;

		// Get nonce - if smart account doesn't exist yet, use 0
		let nonce = if code.is_empty() {
			// Smart account doesn't exist yet, use 0 as nonce
			U256::from(0)
		} else {
			// Smart account exists, get nonce from contract
			let smart_account_client = OmniAccountClient::new(sender, self.rpc_client.clone());
			smart_account_client.get_nonce().await?
		};

		let init_code_to_use = if code.is_empty() {
			// No code at address, include init code
			let init_code_bytes =
				prepare_factory_init_code(factory_address, oa, oa_type, client_id, root_address);
			Bytes::from(init_code_bytes)
		} else {
			// Code already exists, no init code needed
			Bytes::new()
		};

		// Default gas limits - can be adjusted based on requirements
		let verification_gas_limit = U256::from(250000u64); // Higher for verification
		let call_gas_limit = U256::from(50000u64); // Lower for simple calls
		let account_gas_limits = create_account_gas_limits(verification_gas_limit, call_gas_limit);

		let pre_verification_gas = U256::from(21000u64);

		// Calculate dynamic gas fees
		let (max_fee_per_gas, max_priority_fee_per_gas) = self.calculate_gas_fees().await?;
		let gas_fees = create_gas_fees(max_fee_per_gas, max_priority_fee_per_gas);

		let paymaster_and_data = if let Some(paymaster_addr) = paymaster_address {
			create_paymaster_and_data(paymaster_addr, U256::from(50000u64), U256::from(50000u64))
		} else {
			Bytes::new()
		};

		Ok(PackedUserOperation {
			sender,
			nonce,
			initCode: init_code_to_use,
			callData: call_data,
			accountGasLimits: account_gas_limits,
			preVerificationGas: pre_verification_gas,
			gasFees: gas_fees,
			paymasterAndData: paymaster_and_data,
			signature: Bytes::new(),
		})
	}

	/// Calculate gas fees with additional buffer percentage
	async fn calculate_gas_fees_with_buffer(
		&self,
		additional_buffer_percent: u64,
	) -> Result<(U256, U256), AaContractError> {
		// Try EIP-1559 estimation first
		match self.rpc_client.estimate_eip1559_fees().await {
			Ok(eip1559_estimate) => {
				// Use EIP-1559 fees with buffer
				let total_buffer =
					self.gas_config.gas_price_buffer_percent + additional_buffer_percent;
				let buffer_multiplier = 100 + total_buffer;

				let max_fee_per_gas = U256::from(eip1559_estimate.max_fee_per_gas)
					.saturating_mul(U256::from(buffer_multiplier))
					.checked_div(U256::from(100))
					.unwrap_or(U256::from(eip1559_estimate.max_fee_per_gas));

				// Use the EIP-1559 priority fee with bounds
				let priority_fee = U256::from(eip1559_estimate.max_priority_fee_per_gas)
					.max(U256::from(self.gas_config.min_priority_fee))
					.min(U256::from(self.gas_config.max_priority_fee))
					// Ensure priority fee doesn't exceed max fee per gas (EIP-1559 requirement)
					.min(max_fee_per_gas);

				tracing::debug!(
					"EIP-1559 gas fees with buffer: max_fee={} wei ({} gwei), priority_fee={} wei ({} gwei), total_buffer={}%",
					max_fee_per_gas,
					max_fee_per_gas / U256::from(1_000_000_000),
					priority_fee,
					priority_fee / U256::from(1_000_000_000),
					total_buffer
				);

				Ok((max_fee_per_gas, priority_fee))
			},
			Err(_) => {
				// Fallback to legacy gas price calculation
				let current_gas_price = self.rpc_client.get_gas_price().await.map_err(|e| {
					error!("Failed to fetch gas price: {:?}", e);
					AaContractError::from(e)
				})?;

				// Apply base buffer plus additional retry buffer
				let total_buffer =
					self.gas_config.gas_price_buffer_percent + additional_buffer_percent;
				let buffer_multiplier = 100 + total_buffer;

				let max_fee_per_gas = U256::from(current_gas_price)
					.saturating_mul(U256::from(buffer_multiplier))
					.checked_div(U256::from(100))
					.unwrap_or(U256::from(current_gas_price));

				// Calculate priority fee (tip) with bounds
				let priority_fee = U256::from(current_gas_price / 10)
					.max(U256::from(self.gas_config.min_priority_fee))
					.min(U256::from(self.gas_config.max_priority_fee))
					// Ensure priority fee doesn't exceed max fee per gas (EIP-1559 requirement)
					.min(max_fee_per_gas);

				tracing::debug!(
					"Legacy gas fees with buffer: max_fee={} wei ({} gwei), priority_fee={} wei ({} gwei), total_buffer={}%, base_price={} gwei",
					max_fee_per_gas,
					max_fee_per_gas / U256::from(1_000_000_000),
					priority_fee,
					priority_fee / U256::from(1_000_000_000),
					total_buffer,
					current_gas_price / 1_000_000_000
				);

				Ok((max_fee_per_gas, priority_fee))
			},
		}
	}

	fn classify_rpc_error(&self, rpc_error: ethereum_rpc::RpcProviderError) -> AaContractError {
		match rpc_error {
			ethereum_rpc::RpcProviderError::InvalidUrl(url) => {
				AaContractError::Validation(format!("Invalid RPC URL: {}", url))
			},
			ethereum_rpc::RpcProviderError::Network(msg) => {
				AaContractError::Rpc(RpcError::ConnectionFailed { endpoint: msg })
			},
			ethereum_rpc::RpcProviderError::NoWallet => {
				AaContractError::Validation("No wallet configured for signing".to_string())
			},
			ethereum_rpc::RpcProviderError::ExecutionReverted { reason, .. } => {
				AaContractError::Contract(ContractError::ExecutionReverted { reason })
			},
			ethereum_rpc::RpcProviderError::JsonRpc { code, message, data } => {
				// Parse JSON-RPC error codes for specific error types
				match code {
					-32099..=-32000 => {
						// Server errors - parse message for specific conditions
						let message_lower = message.to_lowercase();
						if message_lower.contains("nonce too low")
							|| message_lower.contains("nonce already used")
						{
							AaContractError::Rpc(RpcError::NonceTooLow)
						} else if message_lower.contains("replacement transaction underpriced")
							|| message_lower.contains("transaction underpriced")
							|| message_lower.contains("max fee per gas less than block base fee")
							|| message_lower.contains("gasfeecap less than block base fee")
						{
							AaContractError::Rpc(RpcError::TransactionUnderpriced)
						} else if message_lower.contains("insufficient funds")
							|| message_lower.contains("insufficient balance")
						{
							AaContractError::Contract(ContractError::InsufficientFunds)
						} else if message_lower.contains("gas required exceeds")
							|| message_lower.contains("out of gas")
							|| message_lower.contains("gas too low")
							|| message_lower.contains("intrinsic gas too low")
						{
							AaContractError::Contract(ContractError::GasEstimationFailed {
								reason: message.clone(),
							})
						} else if message_lower.contains("invalid signature")
							|| message_lower.contains("ecrecover")
						{
							AaContractError::Contract(ContractError::InvalidSignature)
						} else {
							AaContractError::Rpc(RpcError::Generic { code, message })
						}
					},
					429 => {
						// HTTP 429 Too Many Requests
						AaContractError::Rpc(RpcError::RateLimited)
					},
					-32603 => {
						// Internal error - usually retryable
						AaContractError::Rpc(RpcError::Generic { code, message })
					},
					_ => {
						// Other error codes
						if let Some(ref data_str) = data {
							// Check if this is contract revert data
							if data_str.starts_with("0x08c379a0") {
								AaContractError::Contract(ContractError::ExecutionReverted {
									reason: message,
								})
							} else {
								AaContractError::Rpc(RpcError::Generic { code, message })
							}
						} else {
							AaContractError::Rpc(RpcError::Generic { code, message })
						}
					},
				}
			},
			ethereum_rpc::RpcProviderError::Transaction(msg) => {
				let msg_lower = msg.to_lowercase();
				if msg_lower.contains("nonce too low") || msg_lower.contains("nonce already used") {
					AaContractError::Rpc(RpcError::NonceTooLow)
				} else if msg_lower.contains("replacement transaction underpriced")
					|| msg_lower.contains("transaction underpriced")
					|| msg_lower.contains("max fee per gas less than block base fee")
					|| msg_lower.contains("gasfeecap less than block base fee")
					|| msg_lower.contains("gas price too low")
				{
					AaContractError::Rpc(RpcError::TransactionUnderpriced)
				} else if msg_lower.contains("insufficient funds")
					|| msg_lower.contains("insufficient balance")
				{
					AaContractError::Contract(ContractError::InsufficientFunds)
				} else if msg_lower.contains("gas required exceeds")
					|| msg_lower.contains("out of gas")
					|| msg_lower.contains("gas too low")
					|| msg_lower.contains("intrinsic gas too low")
				{
					AaContractError::Contract(ContractError::GasEstimationFailed {
						reason: msg.clone(),
					})
				} else if msg_lower.contains("execution reverted") || msg_lower.contains("revert") {
					AaContractError::Contract(ContractError::ExecutionReverted { reason: msg })
				} else {
					AaContractError::Transaction(msg)
				}
			},
			ethereum_rpc::RpcProviderError::Generic(msg) => AaContractError::Generic(msg),
		}
	}
}

#[allow(dead_code)]
pub fn prepare_factory_init_code(
	factory_address: Address,
	oa: [u8; 32],
	oa_type: OwnerType,
	client_id: &[u8],
	root: Address,
) -> Vec<u8> {
	let mut init_code = vec![];

	let mut call = createAccountCall {
		//safe to unwrap
		oa: oa.into(),
		oaType: oa_type,
		//safe to unwrap
		clientId: client_id.to_owned().into(),
		root,
	}
	.abi_encode();
	init_code.append(&mut factory_address.to_vec());
	init_code.append(&mut call);

	init_code
}

pub fn create_paymaster_and_data(
	paymaster_address: Address,
	verification_gas_limit: U256,
	post_op_gas_limit: U256,
) -> Bytes {
	let mut paymaster_and_data = Vec::new();
	paymaster_and_data.extend_from_slice(paymaster_address.as_slice()); // 20 bytes
	paymaster_and_data.extend_from_slice(&verification_gas_limit.to_be_bytes::<32>()[16..]); // 16 bytes
	paymaster_and_data.extend_from_slice(&post_op_gas_limit.to_be_bytes::<32>()[16..]); // 16 bytes
	paymaster_and_data.into()
}

pub fn create_account_gas_limits(
	verification_gas_limit: U256,
	call_gas_limit: U256,
) -> FixedBytes<32> {
	let mut gas_limits = [0u8; 32];
	// First 16 bytes: verification gas limit
	gas_limits[0..16].copy_from_slice(&verification_gas_limit.to_be_bytes::<32>()[16..]);
	// Last 16 bytes: call gas limit
	gas_limits[16..32].copy_from_slice(&call_gas_limit.to_be_bytes::<32>()[16..]);
	FixedBytes::from(gas_limits)
}

pub fn create_gas_fees(max_fee_per_gas: U256, max_priority_fee_per_gas: U256) -> FixedBytes<32> {
	let mut gas_fees = [0u8; 32];
	// First 16 bytes: max fee per gas (EIP-4337 specification)
	gas_fees[0..16].copy_from_slice(&max_fee_per_gas.to_be_bytes::<32>()[16..]);
	// Last 16 bytes: max priority fee per gas (EIP-4337 specification)
	gas_fees[16..32].copy_from_slice(&max_priority_fee_per_gas.to_be_bytes::<32>()[16..]);
	FixedBytes::from(gas_fees)
}

#[cfg(test)]
pub mod test {
	use crate::types::{depositCall, OwnerType};
	use crate::utils::build_payable_transaction;
	use crate::{prepare_factory_init_code, EntryPointClient, GasPriceConfig};
	use alloy::hex;
	use alloy::network::EthereumWallet;
	use alloy::primitives::{address, Bytes, FixedBytes, U256};
	use alloy::signers::local::PrivateKeySigner;
	use alloy::signers::Signer;
	use alloy::sol_types::SolCall;
	use ethereum_rpc::mocks::MockRpcProvider;
	use ethereum_rpc::{AlloyRpcProvider, RpcProvider};
	use heima_primitives::{AccountId, Identity, Web2IdentityType};
	use std::str::FromStr;
	use std::sync::Arc;
	use test_log::test;

	#[test(tokio::test)]
	pub async fn test_get_sender_address() {
		let expected_sender = address!("0x5dfec187c82986cf670f4e2ed6de1cd001cee5be");
		let client_id = "test_client";
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let root_address = address!("0x0000000000000000000000000000000000000001");
		let mut rpc_client = MockRpcProvider::new();

		rpc_client
			.expect_call()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| {
				let revert_data = hex::decode(
					"0x6ca7b8060000000000000000000000005dfec187c82986cf670f4e2ed6de1cd001cee5be",
				)
				.unwrap();
				let reason = format!("execution reverted: 0x{}", hex::encode(&revert_data));
				Err(ethereum_rpc::RpcProviderError::ExecutionReverted { reason, data: None })
			});

		let entrypoint_client = EntryPointClient::new(entrypoint_address, Arc::new(rpc_client));

		let oa: AccountId =
			Identity::Evm(user_address.0.as_slice().try_into().unwrap()).to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let client_id_fixed_bytes = Bytes::from(client_id_bytes);
		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());
		let init_code_bytes = prepare_factory_init_code(
			factory_address,
			oa_bytes.0,
			OwnerType::Evm,
			client_id_fixed_bytes.as_ref(),
			root_address,
		);

		let init_code = Bytes::from(init_code_bytes.to_vec());
		let sender = entrypoint_client.get_sender_address(init_code.clone()).await.unwrap();

		assert_eq!(expected_sender, sender);
	}

	#[test(test)]
	pub fn test_prepare_init_code() {
		let expected = "e7f1725e7734ce288f8367e1bb143e90bb3f0512158b8ca56d659c2361df3754aa7d46d9f0c993ec0335e60508128c6e3843f7dd9621139100000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";

		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let oa: [u8; 32] =
			hex::decode("0x6d659c2361df3754aa7d46d9f0c993ec0335e60508128c6e3843f7dd96211391")
				.unwrap()
				.try_into()
				.unwrap();
		let client_id: Vec<u8> =
			hex::decode("0x0000000000000000000000000000000000000000000000000000000000000000")
				.unwrap();
		let root_address = address!("0x0000000000000000000000000000000000000001");

		let init_code = prepare_factory_init_code(
			factory_address,
			oa,
			OwnerType::Evm,
			&client_id,
			root_address,
		);

		assert_eq!(expected, hex::encode(init_code));
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_get_sender_address() {
		let expected_sender = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let client_id = "test_client";
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let root_address = address!("0x0000000000000000000000000000000000000001");
		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let entrypoint_client = EntryPointClient::new(entrypoint_address, rpc_client);
		// first 20 bytes factory address, then call data which should be oa + client_id + root_address encode packed
		let oa: AccountId =
			Identity::Evm(user_address.0.as_slice().try_into().unwrap()).to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let client_id_fixed_bytes = Bytes::from(client_id_bytes);
		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());
		let init_code_bytes = prepare_factory_init_code(
			factory_address,
			oa_bytes.0,
			OwnerType::Evm,
			&client_id_fixed_bytes,
			root_address,
		);

		let init_code = Bytes::from(init_code_bytes.to_vec());
		let sender = entrypoint_client.get_sender_address(init_code.clone()).await.unwrap();

		assert_eq!(expected_sender, sender);
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn test_simulate_handle_ops() {
		// This test demonstrates how to use the new simulate_handle_ops function
		// to simulate a batch of user operations before actual execution

		let client_id = "test_client";
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let paymaster_address = address!("0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0");
		let beneficiary_address = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
		let root_address = address!("0x0000000000000000000000000000000000000001");

		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let entrypoint_client = EntryPointClient::new(entrypoint_address, rpc_client);

		// Create two user operations for batch simulation
		let oa: AccountId =
			Identity::Evm(user_address.0.as_slice().try_into().unwrap()).to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());

		// Create first user operation
		let call_data1 = Bytes::from(vec![0x12, 0x34]); // dummy call data
		let user_op1 = entrypoint_client
			.create_packed_user_operation(
				factory_address,
				oa_bytes.0,
				OwnerType::Evm,
				client_id_bytes,
				root_address,
				call_data1,
				Some(paymaster_address),
			)
			.await
			.expect("Should create first user operation");

		// Create second user operation with different call data
		let call_data2 = Bytes::from(vec![0x56, 0x78]); // different dummy call data
		let user_op2 = entrypoint_client
			.create_packed_user_operation(
				factory_address,
				oa_bytes.0,
				OwnerType::Evm,
				client_id_bytes,
				root_address,
				call_data2,
				Some(paymaster_address),
			)
			.await
			.expect("Should create second user operation");

		// Simulate batch execution of both user operations
		let user_ops = vec![user_op1, user_op2];
		let simulation_results = entrypoint_client
			.simulate_handle_ops(&user_ops, beneficiary_address)
			.await
			.expect("simulate_handle_ops should succeed");

		// Verify we got results for both operations
		assert_eq!(simulation_results.len(), 2, "Should get results for both operations");

		// Log simulation results
		for (i, result) in simulation_results.iter().enumerate() {
			println!("UserOp {} simulation result:", i);
			println!("  Pre-op gas: {}", result.preOpGas);
			println!("  Paid: {}", result.paid);
			println!("  Account validation data: {}", result.accountValidationData);
			println!("  Paymaster validation data: {}", result.paymasterValidationData);
			println!("  Target success: {}", result.targetSuccess);
		}

		// After successful simulation, execute the batch
		let tx_hash = entrypoint_client
			.handle_ops(&user_ops, beneficiary_address)
			.await
			.expect("Batch execution should succeed");
		println!("Batch transaction hash: {}", tx_hash);
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_full_flow() {
		let oa_type = OwnerType::Evm;
		let user_signer = PrivateKeySigner::from_str(
			"0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
		)
		.unwrap();
		let client_id = "heima";
		// calculate from wallet
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0xe7f1725e7734ce288f8367e1bb143e90bb3f0512");
		let factory_address = address!("0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0");
		let root_address = user_address;
		// This should be the address where SimplePaymaster contract is deployed
		let paymaster_address = address!("0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9");
		let signer = PrivateKeySigner::from_str(
			"0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
		)
		.unwrap();
		let wallet = EthereumWallet::new(signer);
		let rpc_client =
			Arc::new(AlloyRpcProvider::new_with_wallet("http://localhost:8545", wallet));
		let entrypoint_client = EntryPointClient::new(entrypoint_address, rpc_client);
		// first 20 bytes factory address, then call data which should be oa + client_id + root_address encode packed
		let oa: AccountId =
			Identity::from_web2_account("user@example.com", Web2IdentityType::Email)
				.to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let client_id_fixed_bytes = Bytes::from(client_id_bytes);

		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());
		let init_code_bytes = prepare_factory_init_code(
			factory_address,
			oa_bytes.0,
			oa_type,
			&client_id_fixed_bytes,
			root_address,
		);

		// Create PackedUserOperation using the utility function
		let call_data = Bytes::from(init_code_bytes.to_vec()); // Use init_code as call_data for account creation

		let mut user_op = entrypoint_client
			.create_packed_user_operation(
				factory_address,
				oa_bytes.0,
				oa_type,
				&client_id_fixed_bytes,
				root_address,
				call_data.clone(),
				None,
			)
			.await
			.unwrap();

		println!("Sender address: {:?}", user_op.sender);
		println!("Init code: {:?}", hex::encode(init_code_bytes));
		println!("Call data: {:?}", hex::encode(call_data));

		let user_op_hash = entrypoint_client.get_user_op_hash(user_op.clone()).await.unwrap();
		let signature = user_signer.sign_hash(&user_op_hash).await.unwrap();

		// Prepend 0x00 byte to indicate Owner signature type (according to UserOpSigner enum)
		let mut signature_with_prefix: Vec<u8> = vec![0x01];
		signature_with_prefix.extend_from_slice(&signature.as_bytes());
		user_op.signature = signature_with_prefix.into();

		// Fund the paymaster with ETH deposits to EntryPoint
		let paymaster_deposit_call = depositCall {}.abi_encode();
		let paymaster_deposit_tx = build_payable_transaction(
			paymaster_address,
			paymaster_deposit_call,
			U256::from_str("1000000000000000000").unwrap(), // 1 ETH
		);
		entrypoint_client
			.rpc_client
			.send_transaction(paymaster_deposit_tx)
			.await
			.unwrap();

		// Test simulate_validation before executing the user operation
		let _validation_result = entrypoint_client
			.simulate_validation(user_op.clone())
			.await
			.expect("simulate_validation should succeed");

		// Execute user operation with paymaster sponsorship
		let tx_hash =
			entrypoint_client.handle_ops(&vec![user_op], entrypoint_address).await.unwrap();
		println!("Transaction hash: {}", tx_hash);
	}

	/// Integration test to verify that local CREATE2 calculation matches EntryPoint.getSenderAddress
	///
	/// To run this test:
	/// 1. Deploy contracts using: `cd aa-contracts && ./deploy-local.sh`
	/// 2. Run: `cargo test test_local_vs_entrypoint_address_calculation -- --ignored`
	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn test_local_vs_entrypoint_address_calculation() {
		let client_id = "test_client";
		let oa_type = OwnerType::Evm;
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let account_implementation = address!("0xCafaC3dD18aC6c6e92c921884f9E4176737C052C");
		let root_address = address!("0x0000000000000000000000000000000000000001");

		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let entrypoint_client = EntryPointClient::new(entrypoint_address, rpc_client);

		// Create test parameters
		let oa: AccountId =
			Identity::Evm(user_address.0.as_slice().try_into().unwrap()).to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());

		// Calculate address using EntryPoint.getSenderAddress
		let init_code_bytes = prepare_factory_init_code(
			factory_address,
			oa_bytes.0,
			oa_type,
			client_id_bytes,
			root_address,
		);
		let init_code = Bytes::from(init_code_bytes);
		let entrypoint_address_result = entrypoint_client
			.get_sender_address(init_code)
			.await
			.expect("EntryPoint address calculation should succeed");

		// Calculate address using local CREATE2 calculation
		let local_address_result = entrypoint_client.calculate_sender_address(
			factory_address,
			account_implementation,
			oa_bytes,
			oa_type,
			client_id_bytes,
			root_address,
		);

		// Assert that all three methods return the same address
		assert_eq!(entrypoint_address_result, local_address_result,);
	}

	#[test(tokio::test)]
	pub async fn test_dynamic_gas_pricing() {
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let mut rpc_client = MockRpcProvider::new();

		// Mock EIP-1559 to fail, forcing legacy fallback
		rpc_client.expect_estimate_eip1559_fees().times(1).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});
		// Mock gas price at 30 gwei
		let mock_gas_price = 30_000_000_000u128;
		rpc_client.expect_get_gas_price().times(1).returning(move || Ok(mock_gas_price));

		// Test with default config (50% buffer)
		let default_client = EntryPointClient::new(entrypoint_address, Arc::new(rpc_client));
		let (max_fee, priority_fee) = default_client.calculate_gas_fees().await.unwrap();

		// max_fee should be 30 gwei * 1.5 = 45 gwei
		assert_eq!(max_fee, U256::from(45_000_000_000u128));
		// priority fee should be max(3 gwei, min_fee) = 3 gwei
		assert_eq!(priority_fee, U256::from(3_000_000_000u128));

		// Test with mainnet config
		let mut mainnet_rpc_client = MockRpcProvider::new();
		mainnet_rpc_client.expect_estimate_eip1559_fees().times(1).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});
		mainnet_rpc_client
			.expect_get_gas_price()
			.times(1)
			.returning(move || Ok(mock_gas_price));

		let mainnet_client = EntryPointClient::new_with_config(
			entrypoint_address,
			Arc::new(mainnet_rpc_client),
			GasPriceConfig::mainnet(),
			crate::RetryConfig::default(),
		);
		let (max_fee_mainnet, priority_fee_mainnet) =
			mainnet_client.calculate_gas_fees().await.unwrap();

		// Mainnet has same buffer but different priority fee bounds
		assert_eq!(max_fee_mainnet, U256::from(45_000_000_000u128));
		// priority fee should be max(3 gwei, 2 gwei min) = 3 gwei
		assert_eq!(priority_fee_mainnet, U256::from(3_000_000_000u128));

		// Test with L2 config and lower gas price
		let mut l2_rpc_client = MockRpcProvider::new();
		let l2_gas_price = 1_000_000_000u128; // 1 gwei
		l2_rpc_client.expect_estimate_eip1559_fees().times(1).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});
		l2_rpc_client
			.expect_get_gas_price()
			.times(1)
			.returning(move || Ok(l2_gas_price));

		let l2_client = EntryPointClient::new_with_config(
			entrypoint_address,
			Arc::new(l2_rpc_client),
			GasPriceConfig::l2(),
			crate::RetryConfig::default(),
		);
		let (max_fee_l2, priority_fee_l2) = l2_client.calculate_gas_fees().await.unwrap();

		// L2 has 30% buffer: 1 gwei * 1.3 = 1.3 gwei
		assert_eq!(max_fee_l2, U256::from(1_300_000_000u128));
		// priority fee should be max(0.1 gwei, 0.1 gwei min) = 0.1 gwei
		assert_eq!(priority_fee_l2, U256::from(100_000_000u128));
	}

	#[test]
	fn test_retry_config_defaults() {
		let config = crate::RetryConfig::default();
		assert_eq!(config.max_attempts, 3);
		assert_eq!(config.initial_delay_ms, 2000);
		assert_eq!(config.max_delay_ms, 30000);
		assert_eq!(config.gas_increase_percent, 10);
	}

	#[test]
	fn test_retry_config_for_chain() {
		// Test mainnet configuration
		let mainnet_config = crate::RetryConfig::for_chain(1);
		assert_eq!(mainnet_config.max_attempts, 5);
		assert_eq!(mainnet_config.gas_increase_percent, 15);

		// Test L2 configuration
		let arbitrum_config = crate::RetryConfig::for_chain(42161);
		assert_eq!(arbitrum_config.max_attempts, 4);
		assert_eq!(arbitrum_config.gas_increase_percent, 20);

		// Test HyperEVM configuration
		let hyperevm_config = crate::RetryConfig::for_chain(999);
		assert_eq!(hyperevm_config.max_attempts, 2);
		assert_eq!(hyperevm_config.initial_delay_ms, 500);
		assert_eq!(hyperevm_config.max_delay_ms, 5000);
		assert_eq!(hyperevm_config.gas_increase_percent, 5);

		// Test default for unknown chain
		let unknown_config = crate::RetryConfig::for_chain(99999);
		assert_eq!(unknown_config.max_attempts, 3);
		assert_eq!(unknown_config.gas_increase_percent, 10);
	}

	#[test]
	fn test_backoff_calculation() {
		let retry_config = crate::RetryConfig::default();
		let client = EntryPointClient::new_with_config(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(MockRpcProvider::new()),
			GasPriceConfig::default(),
			retry_config,
		);

		// Test exponential backoff
		let delay1 = client.calculate_backoff_delay(1);
		let delay2 = client.calculate_backoff_delay(2);
		let delay3 = client.calculate_backoff_delay(3);

		// Allow for jitter (±10%)
		assert!((1800..=2200).contains(&delay1)); // ~2 seconds
		assert!((3600..=4400).contains(&delay2)); // ~4 seconds
		assert!((7200..=8800).contains(&delay3)); // ~8 seconds

		// Test max cap
		let delay_max = client.calculate_backoff_delay(10);
		assert!(delay_max <= 33000); // Max 30 seconds + 10% jitter
	}

	#[test(tokio::test)]
	async fn test_handle_ops_with_retry_success() {
		use std::sync::atomic::{AtomicU8, Ordering};
		use std::sync::Arc;

		let mut mock_client = MockRpcProvider::new();
		let counter = Arc::new(AtomicU8::new(0));
		let counter_clone = counter.clone();

		// Set up EIP-1559 to fail, forcing legacy fallback
		mock_client.expect_estimate_eip1559_fees().times(3).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});
		// Set up gas price expectation
		mock_client.expect_get_gas_price().times(3).returning(|| Ok(30_000_000_000u128)); // 30 gwei

		// Set up send_transaction to fail twice then succeed
		mock_client.expect_send_transaction().times(3).returning(move |_| {
			let count = counter_clone.fetch_add(1, Ordering::SeqCst);
			if count < 2 {
				Err(ethereum_rpc::RpcProviderError::Network("Temporary network error".to_string()))
			// Fail first two attempts
			} else {
				Ok("0x1234567890abcdef".to_string()) // Succeed on third
			}
		});

		let client = EntryPointClient::new_with_config(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(mock_client),
			GasPriceConfig::default(),
			crate::RetryConfig {
				max_attempts: 3,
				initial_delay_ms: 100, // Short delay for tests
				max_delay_ms: 1000,
				gas_increase_percent: 10,
			},
		);

		let result = client
			.handle_ops_with_retry(&[], address!("0x0000000000000000000000000000000000000000"))
			.await;
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), "0x1234567890abcdef");
		assert_eq!(counter.load(Ordering::SeqCst), 3);
	}

	#[test(tokio::test)]
	async fn test_handle_ops_with_retry_max_attempts() {
		let mut mock_client = MockRpcProvider::new();

		// Set up EIP-1559 to fail, forcing legacy fallback
		mock_client.expect_estimate_eip1559_fees().times(3).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});
		// Set up gas price expectation
		mock_client.expect_get_gas_price().times(3).returning(|| Ok(30_000_000_000u128)); // 30 gwei

		// Set up send_transaction to always fail
		mock_client.expect_send_transaction().times(3).returning(|_| {
			Err(ethereum_rpc::RpcProviderError::Network("Connection failed".to_string()))
		});

		let client = EntryPointClient::new_with_config(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(mock_client),
			GasPriceConfig::default(),
			crate::RetryConfig {
				max_attempts: 3,
				initial_delay_ms: 100, // Short delay for tests
				max_delay_ms: 1000,
				gas_increase_percent: 10,
			},
		);

		let result = client
			.handle_ops_with_retry(&[], address!("0x0000000000000000000000000000000000000000"))
			.await;
		assert!(result.is_err());
	}

	#[test(tokio::test)]
	async fn test_calculate_gas_fees_with_buffer() {
		let mut mock_client = MockRpcProvider::new();

		// Set up EIP-1559 to fail, forcing legacy fallback
		mock_client.expect_estimate_eip1559_fees().times(1).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});

		// Set up gas price expectation
		mock_client.expect_get_gas_price().times(1).returning(|| Ok(20_000_000_000u128)); // 20 gwei

		let client = EntryPointClient::new_with_config(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(mock_client),
			GasPriceConfig {
				gas_price_buffer_percent: 50,
				min_priority_fee: 1_000_000_000,
				max_priority_fee: 50_000_000_000,
			},
			crate::RetryConfig::default(),
		);

		// Test with 10% additional buffer
		let (max_fee, priority_fee) = client.calculate_gas_fees_with_buffer(10).await.unwrap();

		// Expected: 20 gwei * 1.6 (50% + 10%) = 32 gwei
		assert_eq!(max_fee, U256::from(32_000_000_000u128));
		// Priority fee: 20 gwei / 10 = 2 gwei (within bounds)
		assert_eq!(priority_fee, U256::from(2_000_000_000u128));
	}

	#[test(tokio::test)]
	async fn test_calculate_gas_fees_with_eip1559_support() {
		use ethereum_rpc::Eip1559FeeEstimate;
		let mut mock_client = MockRpcProvider::new();

		// Set up EIP-1559 fee estimation to succeed
		mock_client
			.expect_estimate_eip1559_fees()
			.times(1)
			.returning(|| Ok(Eip1559FeeEstimate::new(40_000_000_000, 2_000_000_000))); // 40 gwei max, 2 gwei priority

		let client = EntryPointClient::new_with_config(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(mock_client),
			GasPriceConfig {
				gas_price_buffer_percent: 50,
				min_priority_fee: 1_000_000_000,
				max_priority_fee: 50_000_000_000,
			},
			crate::RetryConfig::default(),
		);

		// Test calculate_gas_fees
		let (max_fee, priority_fee) = client.calculate_gas_fees().await.unwrap();

		// Expected: 40 gwei * 1.5 = 60 gwei
		assert_eq!(max_fee, U256::from(60_000_000_000u128));
		// Priority fee: 2 gwei (within bounds)
		assert_eq!(priority_fee, U256::from(2_000_000_000u128));
	}

	#[test(tokio::test)]
	async fn test_calculate_gas_fees_eip1559_fallback() {
		let mut mock_client = MockRpcProvider::new();

		// Set up EIP-1559 to fail, fallback to legacy
		mock_client.expect_estimate_eip1559_fees().times(1).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});

		mock_client.expect_get_gas_price().times(1).returning(|| Ok(30_000_000_000u128)); // 30 gwei

		let client = EntryPointClient::new_with_config(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(mock_client),
			GasPriceConfig {
				gas_price_buffer_percent: 50,
				min_priority_fee: 1_000_000_000,
				max_priority_fee: 50_000_000_000,
			},
			crate::RetryConfig::default(),
		);

		// Test calculate_gas_fees with fallback
		let (max_fee, priority_fee) = client.calculate_gas_fees().await.unwrap();

		// Expected: 30 gwei * 1.5 = 45 gwei
		assert_eq!(max_fee, U256::from(45_000_000_000u128));
		// Priority fee: 30 gwei / 10 = 3 gwei (within bounds)
		assert_eq!(priority_fee, U256::from(3_000_000_000u128));
	}

	#[test]
	fn test_error_classification() {
		use crate::error::{AaContractError, ContractError, RpcError};
		use ethereum_rpc::RpcProviderError;

		let client = EntryPointClient::new(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(MockRpcProvider::new()),
		);

		// Test network error classification
		let network_error = RpcProviderError::Network("Connection refused".to_string());
		let classified = client.classify_rpc_error(network_error);
		match classified {
			AaContractError::Rpc(RpcError::ConnectionFailed { endpoint }) => {
				assert_eq!(endpoint, "Connection refused");
			},
			_ => panic!("Expected ConnectionFailed error"),
		}

		// Test nonce too low error
		let nonce_error = RpcProviderError::Transaction(
			"nonce too low: address 0x123 current nonce 5".to_string(),
		);
		let classified = client.classify_rpc_error(nonce_error);
		match classified {
			AaContractError::Rpc(RpcError::NonceTooLow) => {},
			_ => panic!("Expected NonceTooLow error"),
		}

		// Test transaction underpriced error
		let gas_error =
			RpcProviderError::Transaction("replacement transaction underpriced".to_string());
		let classified = client.classify_rpc_error(gas_error);
		match classified {
			AaContractError::Rpc(RpcError::TransactionUnderpriced) => {},
			_ => panic!("Expected TransactionUnderpriced error"),
		}

		// Test insufficient funds error
		let funds_error =
			RpcProviderError::Transaction("insufficient funds for gas * price + value".to_string());
		let classified = client.classify_rpc_error(funds_error);
		match classified {
			AaContractError::Contract(ContractError::InsufficientFunds) => {},
			_ => panic!("Expected InsufficientFunds error"),
		}

		// Test execution reverted error
		let revert_error = RpcProviderError::Transaction(
			"execution reverted: ERC20: transfer amount exceeds balance".to_string(),
		);
		let classified = client.classify_rpc_error(revert_error);
		match classified {
			AaContractError::Contract(ContractError::ExecutionReverted { reason }) => {
				assert!(reason.contains("execution reverted"));
			},
			_ => panic!("Expected ExecutionReverted error"),
		}

		// Test rate limiting error
		let rate_limit_error = RpcProviderError::JsonRpc {
			code: 429,
			message: "too many requests, please retry later".to_string(),
			data: None,
		};
		let classified = client.classify_rpc_error(rate_limit_error);
		match classified {
			AaContractError::Rpc(RpcError::RateLimited) => {},
			_ => panic!("Expected RateLimited error"),
		}

		// Test new error patterns
		// Test "max fee per gas less than block base fee"
		let max_fee_error =
			RpcProviderError::Transaction("max fee per gas less than block base fee".to_string());
		let classified = client.classify_rpc_error(max_fee_error);
		match classified {
			AaContractError::Rpc(RpcError::TransactionUnderpriced) => {},
			_ => panic!("Expected TransactionUnderpriced for max fee per gas error"),
		}

		// Test "gas too low"
		let gas_low_error = RpcProviderError::Transaction("gas too low".to_string());
		let classified = client.classify_rpc_error(gas_low_error);
		match classified {
			AaContractError::Contract(ContractError::GasEstimationFailed { reason }) => {
				assert!(reason.contains("gas too low"));
			},
			_ => panic!("Expected GasEstimationFailed for gas too low error"),
		}

		// Test "intrinsic gas too low"
		let intrinsic_gas_error = RpcProviderError::JsonRpc {
			code: -32000,
			message: "intrinsic gas too low".to_string(),
			data: None,
		};
		let classified = client.classify_rpc_error(intrinsic_gas_error);
		match classified {
			AaContractError::Contract(ContractError::GasEstimationFailed { reason }) => {
				assert!(reason.contains("intrinsic gas too low"));
			},
			_ => panic!("Expected GasEstimationFailed for intrinsic gas error"),
		}

		// Test "gas price too low"
		let gas_price_error = RpcProviderError::Transaction("gas price too low".to_string());
		let classified = client.classify_rpc_error(gas_price_error);
		match classified {
			AaContractError::Rpc(RpcError::TransactionUnderpriced) => {},
			_ => panic!("Expected TransactionUnderpriced for gas price too low error"),
		}
	}

	#[test(tokio::test)]
	async fn test_handle_ops_with_retry_non_retryable() {
		// Create a mock that implements RpcProvider
		let mut mock_client = MockRpcProvider::new();

		// Mock estimate_eip1559_fees_ext to fail, forcing legacy fallback
		mock_client.expect_estimate_eip1559_fees().times(1).returning(|| {
			Err(ethereum_rpc::RpcProviderError::Generic("EIP-1559 not supported".to_string()))
		});
		mock_client.expect_get_gas_price().times(1).returning(|| Ok(30_000_000_000u128));

		// Mock send_transaction to return a non-retryable error
		mock_client.expect_send_transaction().times(1).returning(|_| {
			Err(ethereum_rpc::RpcProviderError::ExecutionReverted {
				reason: "Contract error".to_string(),
				data: None,
			})
		});

		let client = EntryPointClient::new_with_config(
			address!("0x0000000000000000000000000000000000000000"),
			Arc::new(mock_client),
			GasPriceConfig::default(),
			crate::RetryConfig {
				max_attempts: 3,
				initial_delay_ms: 100,
				max_delay_ms: 1000,
				gas_increase_percent: 10,
			},
		);

		// Test that non-retryable errors fail immediately
		let result = client
			.handle_ops_with_retry(&[], address!("0x0000000000000000000000000000000000000000"))
			.await;

		assert!(result.is_err());
	}
}
