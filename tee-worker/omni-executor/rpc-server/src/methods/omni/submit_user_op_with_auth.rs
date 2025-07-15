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

					let business_data: serde_json::Value = serde_json::from_str(business_json)
						.map_err(|e| {
							error!("Failed to parse business_json: {:?}", e);
							PumpxRpcError::from_error_code(ErrorCode::ParseError)
						})?;

					let timestamp = business_data
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
	business_json: &str,
	signature: &str,
) -> Result<(), PumpxRpcError> {
	let message = business_json.as_bytes();

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

	if !heima_sig.verify(message, &agent_identity) {
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

// TODO: abstract this once https://github.com/litentry/heima/pull/3590 is merged
// Reuse the logic from submit_user_op.rs
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

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_verify_wildmeta_signature_with_real_data() {
		// Test data provided by the user
		let business_json = r#"{"action":"trade","amount":1.5,"customField1":"buy","customField2":"market","leverage":10,"metadata":{"features":{"darkMode":true,"notifications":false},"userAgent":"mobile-app","version":"1.0.0"},"positions":[{"entryPrice":50000,"metadata":{"openTime":1640995200,"strategy":"momentum"},"side":"long","size":1.5,"symbol":"BTC/USD"},{"entryPrice":3000,"metadata":{"openTime":1640995300,"strategy":"reversal"},"side":"short","size":2,"symbol":"ETH/USD"}],"price":50000,"riskManagement":{"maxLeverage":20,"stopLoss":{"enabled":true,"percentage":0.05},"takeProfit":{"enabled":true,"percentage":0.1}},"slippage":0.01,"symbol":"BTC/USD","timestamp":1752573555}"#;

		let signature = "0x46c737250d61b60cbf0f46a6755e59815844a2f7cdb9dc16bf867b57bfed3526424343a237c15eef9089d571d1f60fd0bd7f91d5888c649216a7df147b386a681c";
		let agent_address = "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd";

		// This should verify successfully
		let result = verify_wildmeta_signature(agent_address, business_json, signature).await;
		assert!(result.is_ok(), "Signature verification should succeed");
	}

	#[tokio::test]
	async fn test_verify_wildmeta_signature_invalid_signature() {
		let business_json = r#"{"action":"trade","timestamp":1752573555}"#;
		let signature = "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
		let agent_address = "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd";

		let result = verify_wildmeta_signature(agent_address, business_json, signature).await;
		assert!(result.is_err(), "Should fail with invalid signature");
	}

	#[tokio::test]
	async fn test_verify_wildmeta_signature_wrong_signer() {
		let business_json = r#"{"action":"trade","amount":1.5,"customField1":"buy","customField2":"market","leverage":10,"metadata":{"features":{"darkMode":true,"notifications":false},"userAgent":"mobile-app","version":"1.0.0"},"positions":[{"entryPrice":50000,"metadata":{"openTime":1640995200,"strategy":"momentum"},"side":"long","size":1.5,"symbol":"BTC/USD"},{"entryPrice":3000,"metadata":{"openTime":1640995300,"strategy":"reversal"},"side":"short","size":2,"symbol":"ETH/USD"}],"price":50000,"riskManagement":{"maxLeverage":20,"stopLoss":{"enabled":true,"percentage":0.05},"takeProfit":{"enabled":true,"percentage":0.1}},"slippage":0.01,"symbol":"BTC/USD","timestamp":1752573555}"#;

		let signature = "0x46c737250d61b60cbf0f46a6755e59815844a2f7cdb9dc16bf867b57bfed3526424343a237c15eef9089d571d1f60fd0bd7f91d5888c649216a7df147b386a681c";
		// Use a different address than the actual signer
		let wrong_agent_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		let result = verify_wildmeta_signature(wrong_agent_address, business_json, signature).await;
		assert!(result.is_err(), "Should fail with wrong signer address");
	}

	#[tokio::test]
	async fn test_verify_wildmeta_signature_invalid_hex() {
		let business_json = r#"{"timestamp":1752573555}"#;
		let signature = "invalid_hex";
		let agent_address = "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd";

		let result = verify_wildmeta_signature(agent_address, business_json, signature).await;
		assert!(result.is_err(), "Should fail with invalid hex signature");
	}

	#[tokio::test]
	async fn test_parse_business_json_timestamp() {
		let business_json = r#"{"action":"trade","timestamp":1752573555}"#;

		let parsed: serde_json::Value = serde_json::from_str(business_json).unwrap();
		let timestamp = parsed.get("timestamp").and_then(|v| v.as_u64());

		assert_eq!(timestamp, Some(1752573555), "Should correctly parse timestamp");
	}

	#[tokio::test]
	async fn test_parse_business_json_missing_timestamp() {
		let business_json = r#"{"action":"trade","amount":1.5}"#;

		let parsed: serde_json::Value = serde_json::from_str(business_json).unwrap();
		let timestamp = parsed.get("timestamp").and_then(|v| v.as_u64());

		assert_eq!(timestamp, None, "Should return None for missing timestamp");
	}
}
