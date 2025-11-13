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

use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::paymaster::{
	extract_paymaster_address, is_whitelisted_paymaster, parse_whitelisted_paymasters,
	process_erc20_paymaster_data,
};
use crate::utils::types::RpcResultExt;
use crate::RpcResult;
use alloy::primitives::{hex, Address, Bytes, FixedBytes, U256};
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::utils::hex::decode_hex;
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::types::ErrorObjectOwned;
use oe_client_aa::calculate_user_operation_hash;
use oe_client_binance::BinancePaymasterApi;
use oe_client_hyperliquid::*;
use oe_client_signer::ChainType;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Pack verification and call gas limits into a single 32-byte value
/// Format: verification_gas (16 bytes) | call_gas (16 bytes)
pub fn pack_account_gas_limits(verification_gas: u128, call_gas: u128) -> FixedBytes<32> {
	let packed: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
	FixedBytes::from(packed.to_be_bytes())
}

#[cfg(test)]
pub fn unpack_verification_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = (value >> 128) & U256::from(u128::MAX);
	result.to::<u128>()
}

#[cfg(test)]
pub fn unpack_call_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = value & U256::from(u128::MAX);
	result.to::<u128>()
}

/// Convert Substrate signature to Ethereum ECDSA format
/// Returns signature in format: [r (32 bytes), s (32 bytes), v (1 byte)]
pub fn substrate_to_ethereum_signature(substrate_sig: &[u8]) -> Result<[u8; 65], &'static str> {
	if substrate_sig.len() != 65 {
		return Err("Invalid signature length");
	}

	// Parse as (r, s, v) format - most common
	let mut r = [0u8; 32];
	let mut s = [0u8; 32];
	r.copy_from_slice(&substrate_sig[0..32]);
	s.copy_from_slice(&substrate_sig[32..64]);
	let substrate_v = substrate_sig[64];

	// Convert recovery parameter: 0/1 -> 27/28
	let ethereum_v = match substrate_v {
		0 => 27,
		1 => 28,
		27 => 27, // Already Ethereum format
		28 => 28, // Already Ethereum format
		_ => return Err("Invalid recovery parameter"),
	};

	// Build Ethereum signature: [r, s, v]
	let mut ethereum_sig = [0u8; 65];
	ethereum_sig[0..32].copy_from_slice(&r);
	ethereum_sig[32..64].copy_from_slice(&s);
	ethereum_sig[64] = ethereum_v;

	Ok(ethereum_sig)
}

/// Convert SerializablePackedUserOperation to oe_client_aa::PackedUserOperation
pub fn convert_to_packed_user_op(
	user_op: SerializablePackedUserOperation,
) -> Result<oe_client_aa::PackedUserOperation, String> {
	use std::str::FromStr;

	// Helper function to parse hex string to fixed bytes
	let parse_hex_fixed =
		|hex_str: &str, expected_len: usize, name: &str| -> Result<Vec<u8>, String> {
			let bytes = decode_hex(hex_str)
				.map_err(|e| format!("Invalid hex string '{}' '{}': {}", hex_str, name, e))?;
			if bytes.len() != expected_len {
				return Err(format!(
					"Expected {} bytes, got {} for '{}'",
					expected_len,
					bytes.len(),
					hex_str
				));
			}
			Ok(bytes)
		};

	Ok(oe_client_aa::PackedUserOperation {
		sender: Address::from_str(&user_op.sender)
			.map_err(|e| format!("Invalid sender address '{}': {}", user_op.sender, e))?,
		nonce: U256::from(user_op.nonce),
		initCode: Bytes::from(
			decode_hex(&user_op.init_code).map_err(|e| format!("Invalid init_code hex: {}", e))?,
		),
		callData: Bytes::from(
			decode_hex(&user_op.call_data).map_err(|e| format!("Invalid call_data hex: {}", e))?,
		),
		accountGasLimits: {
			let bytes = parse_hex_fixed(&user_op.account_gas_limits, 32, "account_gas_limits")?;
			FixedBytes::from_slice(&bytes)
		},
		preVerificationGas: U256::from(user_op.pre_verification_gas),
		gasFees: {
			let bytes = parse_hex_fixed(&user_op.gas_fees, 32, "gas_fees")?;
			FixedBytes::from_slice(&bytes)
		},
		paymasterAndData: Bytes::from(
			decode_hex(&user_op.paymaster_and_data)
				.map_err(|e| format!("Invalid paymaster_and_data hex: {}", e))?,
		),
		signature: match user_op.signature {
			Some(sig) => {
				Bytes::from(decode_hex(&sig).map_err(|e| format!("Invalid signature hex: {}", e))?)
			},
			None => Bytes::new(), // Empty signature for unsigned operations
		},
	})
}

/// Helper function to submit a CoreWriter userOp
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub(crate) async fn submit_corewriter_user_ops<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	omni_account: &AccountId,
	skeleton_user_op: &SerializablePackedUserOperation,
	chain_id: u64,
	wallet_index: u32,
	call_data: String,
) -> Result<Option<String>, ErrorObjectOwned> {
	let smart_wallet_address = &skeleton_user_op.sender;

	let entry_point_client = ctx.entry_point_clients.get(&chain_id).ok_or_else(|| {
		error!("No EntryPoint client configured for chain_id: {}", chain_id);
		DetailedError::invalid_chain_id(chain_id).to_rpc_error()
	})?;

	let nonce = skeleton_user_op.nonce;
	info!("Using nonce {} for smart wallet {}", nonce, smart_wallet_address);

	// Use gas settings from skeleton UserOp if provided, otherwise calculate
	let (gas_fees, account_gas_limits, pre_verification_gas) = if !skeleton_user_op
		.gas_fees
		.is_empty()
		&& skeleton_user_op.gas_fees != "0x"
		&& !skeleton_user_op.account_gas_limits.is_empty()
		&& skeleton_user_op.account_gas_limits != "0x"
	{
		info!("Using gas settings from skeleton UserOp");
		(
			skeleton_user_op.gas_fees.clone(),
			skeleton_user_op.account_gas_limits.clone(),
			skeleton_user_op.pre_verification_gas,
		)
	} else {
		info!("Calculating gas fees");
		let (max_fee_per_gas, max_priority_fee_per_gas) = entry_point_client
			.calculate_gas_fees_with_buffer(20)
			.await
			.map_err_internal("Failed to calculate gas fees")?;
		(
			pack_gas_fees(max_fee_per_gas.to::<u128>(), max_priority_fee_per_gas.to::<u128>()),
			format!("0x{}", hex::encode(pack_account_gas_limits(1_000_000, 2_000_000).as_slice())),
			100_000,
		)
	};

	let init_code = skeleton_user_op.init_code.clone();

	// Build UserOp
	let user_op = SerializablePackedUserOperation {
		sender: smart_wallet_address.to_string(),
		nonce,
		init_code,
		call_data,
		account_gas_limits,
		pre_verification_gas,
		gas_fees,
		paymaster_and_data: if !skeleton_user_op.paymaster_and_data.is_empty()
			&& skeleton_user_op.paymaster_and_data != "0x"
		{
			skeleton_user_op.paymaster_and_data.clone()
		} else {
			encode_simple_paymaster()
		},
		signature: None, // Will be signed by submit_user_ops
	};

	// Use the common submission logic
	submit_user_ops(&ctx, vec![user_op], chain_id, wallet_index, omni_account).await
}

/// Common user operation submission logic shared between test and auth endpoints
/// This function handles the complete flow of processing, signing, and submitting user operations
pub async fn submit_user_ops<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	ctx: &RpcContext<CrossChainIntentExecutor>,
	user_operations: Vec<SerializablePackedUserOperation>,
	chain_id: ChainId,
	wallet_index: u32,
	omni_account: &executor_primitives::AccountId,
) -> RpcResult<Option<String>> {
	// Inlined handler logic from handle_submit_user_op
	info!(
		"Processing SubmitUserOp for {} UserOps on chain_id: {}",
		user_operations.len(),
		chain_id
	);

	// Get EntryPoint client for this chain (needed for both signing and submission)
	let entry_point_client = ctx.entry_point_clients.get(&chain_id).ok_or_else(|| {
		error!("No EntryPoint client configured for chain_id: {}", chain_id);
		DetailedError::invalid_chain_id(chain_id).to_rpc_error()
	})?;

	// Parse whitelisted paymasters once
	let whitelisted_paymaster = parse_whitelisted_paymasters();

	let mut user_ops = Vec::new();

	for (index, serializable_user_op) in user_operations.iter().enumerate() {
		let mut packed_user_op =
			convert_to_packed_user_op(serializable_user_op.clone()).map_err(|e| {
				error!("Failed to convert UserOp[{}]: {}", index, e);
				DetailedError::invalid_user_op(&format!(
					"Failed to convert UserOp[{}]: {}",
					index, e
				))
				.to_rpc_error()
			})?;

		// Check userOp signature status and validate paymaster usage
		if packed_user_op.signature.is_empty() {
			// UNSIGNED userOp: If paymaster specified, must be whitelisted
			if !packed_user_op.paymasterAndData.is_empty() {
				if let Some(paymaster_address) =
					extract_paymaster_address(&packed_user_op.paymasterAndData)
				{
					if !is_whitelisted_paymaster(&paymaster_address, &whitelisted_paymaster) {
						error!(
							"UserOp {} uses non-whitelisted paymaster {}. Only whitelisted paymasters are allowed for unsigned userOps.",
							index, paymaster_address
						);
						return Err(DetailedError::invalid_user_op(&format!(
							"UserOp[{}] uses non-whitelisted paymaster {}",
							index, paymaster_address
						))
						.to_rpc_error());
					}
				}

				match process_erc20_paymaster_data(
					ctx.oe_client_binance_client.as_ref() as &dyn BinancePaymasterApi,
					&packed_user_op.paymasterAndData,
					chain_id,
				)
				.await
				{
					Ok(Some(updated_paymaster_data)) => {
						packed_user_op.paymasterAndData = updated_paymaster_data;
						info!("Updated ERC20 paymaster data for UserOp[{}]", index);
					},
					Ok(None) => {
						// Not an ERC20 paymaster, continue as normal
						debug!("UserOp[{}] does not use ERC20 paymaster", index);
					},
					Err(e) => {
						error!(
							"Failed to process ERC20 paymaster data for UserOp[{}]: {}",
							index, e
						);
						return Err(DetailedError::invalid_user_op(&format!(
							"ERC20 paymaster processing failed for UserOp[{}]: {}",
							index, e
						))
						.to_rpc_error());
					},
				}
			}

			info!("Requesting signature from pumpx signer for UserOp[{}]", index);

			// Log UserOp details for debugging
			info!(
				"UserOp details: sender={}, nonce={}, initCode len={}, callData len={}",
				packed_user_op.sender,
				packed_user_op.nonce,
				packed_user_op.initCode.len(),
				packed_user_op.callData.len()
			);

			let entry_point_address = entry_point_client.entry_point_address();

			let user_op_hash_bytes =
				calculate_user_operation_hash(&packed_user_op, entry_point_address, chain_id);
			let message_to_sign = user_op_hash_bytes.to_vec();

			info!(
				"Signing UserOp hash: 0x{}, EntryPoint: {}, ChainID: {}",
				hex::encode(user_op_hash_bytes),
				entry_point_address,
				chain_id
			);

			// Request signature from pumpx signer for EVM chain
			let sig = ctx
				.signer_client
				.request_signature(
					ChainType::Evm,
					wallet_index,
					omni_account.clone().into(),
					message_to_sign,
				)
				.await
				.map_err(|_| DetailedError::signer_service_error().to_rpc_error())?;

			let signature = substrate_to_ethereum_signature(&sig)
				.map_err_internal("Failed to convert signature")?
				.to_vec();

			// Prepend 0x01 byte to indicate Root signature type (according to UserOpSigner enum)
			let mut signature_with_prefix: Vec<u8> = vec![0x01];
			signature_with_prefix.extend_from_slice(&signature);
			packed_user_op.signature = Bytes::from(signature_with_prefix);
			info!("UserOp[{}] signed successfully", index);
		} else {
			// SIGNED userOp: Only allowed if no paymaster specified
			if !packed_user_op.paymasterAndData.is_empty() {
				error!(
					"UserOp[{}] is signed but has paymaster data, signed userOps are only allowed without paymaster",
					index
				);
				return Err(DetailedError::invalid_user_op(&format!(
					"UserOp[{}] is signed but specifies a paymaster",
					index
				))
				.to_rpc_error());
			}
			info!("UserOp[{}] is signed with no paymaster, processing", index);
		}

		// Convert to oe_client_aa::PackedUserOperation for EntryPoint call
		let aa_user_op = oe_client_aa::PackedUserOperation {
			sender: packed_user_op.sender,
			nonce: packed_user_op.nonce,
			initCode: packed_user_op.initCode.clone(),
			callData: packed_user_op.callData.clone(),
			accountGasLimits: packed_user_op.accountGasLimits,
			preVerificationGas: packed_user_op.preVerificationGas,
			gasFees: packed_user_op.gasFees,
			paymasterAndData: packed_user_op.paymasterAndData.clone(),
			signature: packed_user_op.signature.clone(),
		};
		user_ops.push(aa_user_op);
	}

	// Get beneficiary address from the EntryPoint client's wallet
	let beneficiary = entry_point_client
		.get_wallet_address()
		.await
		.map_err_internal("Failed to get wallet address from EntryPoint client")?;

	// Run batch simulation for all UserOperations before submission
	info!("Running batch simulation for {} UserOps", user_ops.len());
	match entry_point_client.simulate_handle_ops(&user_ops, beneficiary).await {
		Ok(simulation_results) => {
			for (index, result) in simulation_results.iter().enumerate() {
				info!(
					"UserOp[{}] simulation successful, preOpGas={}, paid={}, accountValidationData={}, paymasterValidationData={}",
					index,
					result.preOpGas,
					result.paid,
					result.accountValidationData,
					result.paymasterValidationData
				);
			}
			info!("All {} UserOps passed batch simulation checks", user_ops.len());
		},
		Err(e) => {
			let err_msg: String = format!("Batch UserOp simulation failed: {}", e);
			error!("{}", err_msg.clone());
			return Err(DetailedError::invalid_user_op(&err_msg).to_rpc_error());
		},
	}

	// Submit all UserOperations via EntryPoint.handleOps() with retry logic
	let transaction_hash =
		entry_point_client
			.handle_ops_with_retry(&user_ops, beneficiary)
			.await
			.map_err_internal("Failed to submit UserOps to EntryPoint after retries")?;

	Ok(Some(transaction_hash))
}
