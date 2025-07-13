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

use super::common::handle_omni_native_task;
use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::ErrorCode;
use alloy::primitives::{Address, FixedBytes};
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{
	AccountId, ChainId, ClientAuth, Identity, UserId, UserAuth,
	signature::{HeimaMultiSignature, EthereumSignature},
};
use executor_storage::{Storage, WildmetaTimestampStorage};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpWithAuthParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub user_id: UserId,
	pub user_auth: Option<UserAuth>,
	pub client_id: String,
	pub client_auth: ClientAuth,
}

#[derive(Serialize, Clone)]
pub struct SubmitUserOpWithAuthResponse {
	pub transaction_hash: Option<String>,
}

pub fn register_submit_user_op_with_auth(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_submitUserOpWithAuth", |params, ctx, _ext| async move {
			let params = params.parse::<SubmitUserOpWithAuthParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_submitUserOpWithAuth, params: {:?}", params);

			// Verify client auth
			let (_agent_address, main_address, _timestamp) = match &params.client_auth {
				ClientAuth::WildmetaHl {
					agent_address,
					business_json,
					main_address,
					signature,
					login_type,
				} => {
					// Verify signature
					verify_wildmeta_signature(
						agent_address,
						business_json,
						signature,
					).await?;

					// Extract timestamp from business_json
					let timestamp = business_json.get("timestamp")
						.and_then(|v| v.as_u64())
						.ok_or_else(|| {
							error!("Missing timestamp in business_json");
							PumpxRpcError::from_error_code(ErrorCode::ParseError)
						})?;

					// Check timestamp is monotonically increasing
					verify_timestamp_monotonic(
						&ctx.wildmeta_timestamp_storage,
						main_address,
						timestamp,
					).await?;

					// Verify hyperliquid link
					let linked = ctx.wildmeta_api
						.verify_hyperliquid_link(agent_address, main_address, *login_type)
						.await
						.map_err(|_| {
							error!("Failed to verify hyperliquid link");
							PumpxRpcError::from_error_code(ErrorCode::InternalError)
						})?;

					if !linked {
						error!("Agent and main addresses are not linked");
						return Err(PumpxRpcError::from_error_code(
							ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
						));
					}

					(agent_address.clone(), main_address.clone(), timestamp)
				},
				_ => {
					error!("Invalid client auth type");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ParseError));
				}
			};

			// Convert user_id to Identity and then to AccountId
			let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
				error!("Failed to convert UserId to Identity: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			// For non-EVM identities, verify that main_address matches derived EVM address
			match &identity {
				Identity::Evm(_) => {
					// For EVM identity, main_address should match user_id
					if let UserId::Evm(user_address) = &params.user_id {
						if user_address.to_lowercase() != main_address.to_lowercase() {
							error!("Main address does not match user_id for EVM identity");
							return Err(PumpxRpcError::from_error_code(
								ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
							));
						}
					}
				},
				_ => {
					// For non-EVM identities, derive EVM address and verify
					let omni_account = identity.to_omni_account(&params.client_id);
					// Convert AccountId to bytes
					let omni_account_bytes: [u8; 32] = omni_account.encode().try_into().map_err(|_| {
						error!("Failed to convert omni account to bytes");
						PumpxRpcError::from_error_code(ErrorCode::InternalError)
					})?;
					let derived_address = ctx.signer_client
						.request_wallet(
							signer_client::ChainType::Evm,
							params.wallet_index,
							omni_account_bytes,
						)
						.await
						.map_err(|_| {
							error!("Failed to derive EVM address");
							PumpxRpcError::from_error_code(ErrorCode::InternalError)
						})?;

					// Compare derived address with main_address
					let derived_hex = format!("0x{}", hex::encode(&derived_address));
					if derived_hex.to_lowercase() != main_address.to_lowercase() {
						error!("Main address does not match derived EVM address");
						return Err(PumpxRpcError::from_error_code(
							ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
						));
					}
				}
			}

			// Get AccountId from identity
			let account_id = identity.to_omni_account(&params.client_id);

			// Validate UserOperation ownership (reuse existing logic)
			let unique_addresses: HashSet<Address> = params
				.user_operations
				.iter()
				.map(|op| {
					op.sender.parse::<Address>().map_err(|e| {
						error!("Invalid sender address '{}': {}", op.sender, e);
						PumpxRpcError::from_error_code(ErrorCode::ParseError)
					})
				})
				.collect::<Result<HashSet<_>, _>>()?;

			for sender_address in unique_addresses {
				validate_user_operation_ownership(
					&account_id,
					&sender_address,
					&params.chain_id,
					&ctx,
				)
				.await?;
			}

			// Create task wrapper and submit
			let wrapper = NativeTaskWrapper::new(
				NativeTask::SubmitUserOp(
					account_id,
					params.user_operations.clone(),
					params.chain_id,
					params.wallet_index,
				),
				None,
				None,
				params.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::SubmitUserOp(transaction_hash) => {
					Ok(SubmitUserOpWithAuthResponse { transaction_hash })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_submitUserOpWithAuth method");
}

async fn verify_wildmeta_signature(
	agent_address: &str,
	business_json: &serde_json::Value,
	signature: &str,
) -> Result<(), PumpxRpcError> {
	// Serialize business_json to bytes
	let message = serde_json::to_vec(business_json).map_err(|e| {
		error!("Failed to serialize business_json: {:?}", e);
		PumpxRpcError::from_error_code(ErrorCode::ParseError)
	})?;

	// Create Ethereum signed message
	let eth_message = format!("\x19Ethereum Signed Message:\n{}{}", 
		message.len(), 
		String::from_utf8_lossy(&message)
	);
	
	// Parse signature
	let sig_bytes = hex::decode(signature.strip_prefix("0x").unwrap_or(signature))
		.map_err(|e| {
			error!("Failed to decode signature: {:?}", e);
			PumpxRpcError::from_error_code(ErrorCode::ParseError)
		})?;

	// Convert to array and create EthereumSignature
	let sig_array: [u8; 65] = sig_bytes.try_into().map_err(|_| {
		error!("Invalid signature length, expected 65 bytes");
		PumpxRpcError::from_error_code(ErrorCode::ParseError)
	})?;

	// Create HeimaMultiSignature
	let heima_sig = HeimaMultiSignature::Ethereum(EthereumSignature(sig_array));

	// Parse agent address
	let agent_addr_bytes = hex::decode(agent_address.strip_prefix("0x").unwrap_or(agent_address))
		.map_err(|e| {
			error!("Failed to decode agent address: {:?}", e);
			PumpxRpcError::from_error_code(ErrorCode::ParseError)
		})?;

	// Convert to array
	let agent_addr_array: [u8; 20] = agent_addr_bytes.try_into().map_err(|_| {
		error!("Invalid agent address length, expected 20 bytes");
		PumpxRpcError::from_error_code(ErrorCode::ParseError)
	})?;

	let agent_identity = Identity::Evm(agent_addr_array.into());

	// Verify signature
	if !heima_sig.verify(eth_message.as_bytes(), &agent_identity) {
		error!("Signature verification failed");
		return Err(PumpxRpcError::from_error_code(
			ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
		));
	}

	Ok(())
}

async fn verify_timestamp_monotonic(
	storage: &Arc<WildmetaTimestampStorage>,
	main_address: &str,
	new_timestamp: u64,
) -> Result<(), PumpxRpcError> {
	// Get last timestamp
	let last_timestamp = storage.get(&main_address.to_string())
		.map_err(|_| {
			error!("Failed to get last timestamp");
			PumpxRpcError::from_error_code(ErrorCode::InternalError)
		})?
		.unwrap_or(0);

	// Check monotonic increase
	if new_timestamp <= last_timestamp {
		error!("Timestamp not monotonically increasing: {} <= {}", new_timestamp, last_timestamp);
		return Err(PumpxRpcError::from_error_code(
			ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
		));
	}

	// Store new timestamp
	storage.insert(&main_address.to_string(), new_timestamp)
		.map_err(|_| {
			error!("Failed to store timestamp");
			PumpxRpcError::from_error_code(ErrorCode::InternalError)
		})?;

	Ok(())
}

async fn validate_user_operation_ownership(
	account_id: &AccountId,
	sender_address: &Address,
	chain_id: &ChainId,
	ctx: &RpcContext,
) -> Result<(), PumpxRpcError> {
	// Similar to existing implementation in submit_user_op.rs
	// Get expected OA from account_id - AccountId is already 32 bytes
	let account_bytes = account_id.encode();
	let expected_oa = FixedBytes::<32>::from_slice(&account_bytes);

	// Get RPC client for chain
	let Some(rpc_client) = ctx.rpc_clients.get(chain_id) else {
		error!("No RPC client found for chain ID: {}", chain_id);
		return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
	};

	// Query OmniWallet contract
	let omni_client = aa_contracts_client::OmniAccountClient::new(*sender_address, rpc_client.clone());
	
	let stored_oa = match omni_client.get_owner().await {
		Ok(oa) => oa,
		Err(e) => {
			error!("Failed to query OmniWallet owner: {:?}", e);
			return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
		},
	};

	// Compare OA values
	if stored_oa != expected_oa {
		error!(
			"OA mismatch: contract has 0x{}, expected 0x{}",
			hex::encode(stored_oa.as_slice()),
			hex::encode(expected_oa.as_slice())
		);
		return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(-32010)));
	}

	Ok(())
}