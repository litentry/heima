use super::common::handle_omni_native_task;
use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::ErrorCode;
use alloy::primitives::{Address, FixedBytes};
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::utils::hex::decode_hex;
use executor_primitives::{
	signature::{EthereumSignature, HeimaMultiSignature},
	utils::hex::FromHexPrefixed,
	AccountId, ChainId, ClientAuth, Identity, UserAuth, UserId,
};
use executor_storage::{Storage, WildmetaTimestampStorage};
use heima_primitives::Address20;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Encode;
use pumpx::pubkey_to_address;
use serde::{Deserialize, Serialize};
use signer_client::ChainType;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpWithAuthParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub user_id: UserId,
	pub _user_auth: Option<UserAuth>,
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

			let (_agent_address, main_address, _timestamp) = match &params.client_auth {
				ClientAuth::WildmetaHl {
					agent_address,
					business_json,
					main_address,
					signature,
					login_type,
				} => {
					verify_wildmeta_signature(agent_address, business_json, signature).await?;

					let timestamp = business_json
						.get("timestamp")
						.and_then(|v| v.as_u64())
						.ok_or_else(|| {
							error!("Missing timestamp in business_json");
							PumpxRpcError::from_error_code(ErrorCode::ParseError)
						})?;

					verify_payload_timestamp(
						&ctx.wildmeta_timestamp_storage,
						main_address,
						timestamp,
					)
					.await?;

					let linked = ctx
						.wildmeta_api
						.verify_hyperliquid_link(agent_address, main_address, *login_type)
						.await
						.map_err(|_| {
							error!("Failed to verify hyperliquid link");
							PumpxRpcError::from_error_code(ErrorCode::InternalError)
						})?;

					if !linked {
						error!("Agent and main addresses are not linked");
						return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
							AUTH_VERIFICATION_FAILED_CODE,
						)));
					}

					(agent_address.clone(), main_address.clone(), timestamp)
				},
				_ => {
					error!("Invalid client auth type");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ParseError));
				},
			};

			let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
				error!("Failed to convert UserId to Identity: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			match &identity {
				Identity::Evm(_) => {
					if let UserId::Evm(user_address) = &params.user_id {
						if user_address.to_lowercase() != main_address.to_lowercase() {
							error!("Main address does not match user_id for EVM identity");
							return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
								AUTH_VERIFICATION_FAILED_CODE,
							)));
						}
					}
				},
				_ => {
					let omni_account = identity.to_omni_account(&params.client_id);
					let derived_pubkey = ctx
						.signer_client
						.request_wallet(ChainType::Evm, params.wallet_index, *omni_account.as_ref())
						.await
						.map_err(|_| {
							error!("Failed to derive EVM address");
							PumpxRpcError::from_error_code(ErrorCode::InternalError)
						})?;
					let derived_address = pubkey_to_address(ChainType::Evm, &derived_pubkey)
						.map_err(|_| {
							error!("Failed to convert derived pubkey to address");
							PumpxRpcError::from_error_code(ErrorCode::ServerError(
								AUTH_VERIFICATION_FAILED_CODE,
							))
						})?;
					if derived_address.to_lowercase() != main_address.to_lowercase() {
						error!("Main address does not match derived EVM address");
						return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
							AUTH_VERIFICATION_FAILED_CODE,
						)));
					}
				},
			}

			let account_id = identity.to_omni_account(&params.client_id);

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
	let message = serde_json::to_vec(business_json).map_err(|e| {
		error!("Failed to serialize business_json: {:?}", e);
		PumpxRpcError::from_error_code(ErrorCode::ParseError)
	})?;
	let signature_bytes = decode_hex(signature).map_err(|e| {
		error!("Failed to decode signature: {:?}", e);
		PumpxRpcError::from_error_code(ErrorCode::ParseError)
	})?;
	let ethereum_signature =
		EthereumSignature::try_from(signature_bytes.as_slice()).map_err(|e| {
			error!("Failed to convert signature to EthereumSignature: {:?}", e);
			PumpxRpcError::from_error_code(ErrorCode::ParseError)
		})?;
	let heima_sig = HeimaMultiSignature::Ethereum(ethereum_signature);

	let agent_address = Address20::from_hex(agent_address).map_err(|e| {
		error!("Failed to parse agent address: {:?}", e);
		PumpxRpcError::from_error_code(ErrorCode::ParseError)
	})?;

	let agent_identity = Identity::Evm(agent_address);

	if !heima_sig.verify(&message, &agent_identity) {
		error!("Signature verification failed");
		return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
			AUTH_VERIFICATION_FAILED_CODE,
		)));
	}

	Ok(())
}

async fn verify_payload_timestamp(
	storage: &Arc<WildmetaTimestampStorage>,
	main_address: &str,
	new_timestamp: u64,
) -> Result<(), PumpxRpcError> {
	let last_timestamp = storage
		.get(&main_address.to_string())
		.map_err(|_| {
			error!("Failed to get last timestamp");
			PumpxRpcError::from_error_code(ErrorCode::InternalError)
		})?
		.unwrap_or(0);

	if new_timestamp <= last_timestamp {
		error!("Invalid payload timestamp: {} <= {}", new_timestamp, last_timestamp);
		return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
			AUTH_VERIFICATION_FAILED_CODE,
		)));
	}

	storage.insert(&main_address.to_string(), new_timestamp).map_err(|_| {
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
	let omni_client =
		aa_contracts_client::OmniAccountClient::new(*sender_address, rpc_client.clone());

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
