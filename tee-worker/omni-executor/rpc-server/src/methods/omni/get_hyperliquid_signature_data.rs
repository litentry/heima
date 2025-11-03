use crate::{
	detailed_error::DetailedError,
	error_code::*,
	methods::RpcResult,
	server::RpcContext,
	utils::auth::{verify_payload_timestamp, verify_wildmeta_signature},
	utils::validation::validate_ethereum_address,
	verify_auth::verify_auth,
};
use chrono::Utc;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{
	to_omni_auth, utils::hex::hex_encode, ChainId, ClientAuth, Identity, UserAuth, UserId,
};
use hyperliquid_rust_sdk::{
	ApproveAgent, ApproveBuilderFee, Eip712, SendAsset, UserDexAbstraction, Withdraw3,
};
use jsonrpsee::RpcModule;
use pumpx::pubkey_to_address;
use serde::{Deserialize, Serialize};
use signer_client::ChainType;
use std::convert::TryFrom;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct GetHyperliquidSignatureDataParams {
	pub user_id: UserId,
	pub user_auth: Option<UserAuth>,
	pub client_id: String,
	pub client_auth: Option<ClientAuth>,
	pub action_type: HyperliquidActionType,
	pub chain_id: ChainId,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HyperliquidActionType {
	ApproveAgent {
		agent_address: String,
		agent_name: Option<String>,
	},
	Withdraw3 {
		amount: String,
		destination: String,
	},
	ApproveBuilderFee {
		max_fee_rate: String,
		builder: String,
	},
	SendAsset {
		destination: String,
		source_dex: String,
		destination_dex: String,
		token: String,
		amount: String,
		from_sub_account: String,
	},
	UserDexAbstraction {
		user: String,
		enabled: bool,
	},
}

#[derive(Serialize, Clone)]
pub struct GetHyperliquidSignatureDataResponse {
	pub main_address: String,
	pub hyperliquid_signature_data: HyperliquidSignatureData,
}

#[derive(Serialize, Clone)]
pub struct HyperliquidSignatureData {
	pub action: HyperliquidAction,
	pub nonce: u64,
	pub signature: String,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HyperliquidAction {
	ApproveAgent(ApproveAgent),
	Withdraw3(Withdraw3),
	ApproveBuilderFee(ApproveBuilderFee),
	SendAsset(SendAsset),
	UserDexAbstraction(UserDexAbstraction),
}

fn is_testnet_chain(chain_id: ChainId) -> bool {
	match chain_id {
		// Testnet chain IDs
		11155111 => true, // Ethereum Sepolia
		97 => true,       // BSC Testnet
		80001 => true,    // Polygon Mumbai
		421614 => true,   // Arbitrum Sepolia
		998 => true,      // HyperEVM Testnet
		1337 => true,     // Local Anvil
		31337 => true,    // Local Anvil
		// Mainnet chain IDs
		1 => false,     // Ethereum Mainnet
		56 => false,    // BSC Mainnet
		137 => false,   // Polygon Mainnet
		42161 => false, // Arbitrum Mainnet
		999 => false,   // HyperEVM Mainnet
		_ => {
			// Default to mainnet for unknown chain IDs
			false
		},
	}
}

pub fn register_get_hyperliquid_signature_data<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_getHyperliquidSignatureData", |params, ctx, _| async move {
			let params = params.parse::<GetHyperliquidSignatureDataParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Failed to parse request parameters")
					.with_reason(format!("Invalid JSON structure: {}", e))
					.to_rpc_error()
			})?;

			debug!("Received omni_getHyperliquidSignatureData, params: {:?}", params);

			// Make sure `user_id` is non-evm type
			if matches!(params.user_id, UserId::Evm(_)) {
				error!("Invalid user_id type, expected non-Evm");
				return Err(DetailedError::new(INVALID_PARAMS_CODE, "Invalid user ID type")
					.with_field("user_id")
					.with_expected("Non-EVM user ID (Email, Twitter, Discord, etc.)")
					.with_received("EVM type")
					.with_suggestion("Use a non-EVM user ID type for this operation")
					.to_rpc_error());
			}

			// Unified authentication logic
			let main_address = if let Some(user_auth) = &params.user_auth {
				// User authentication provided
				let auth =
					to_omni_auth(user_auth, &params.user_id, &params.client_id).map_err(|e| {
						error!("Failed to convert to OmniAuth: {:?}", e);
						DetailedError::new(PARSE_ERROR_CODE, "Failed to convert authentication data")
							.with_field("user_auth")
							.with_reason(format!("OmniAuth conversion error: {:?}", e))
							.to_rpc_error()
					})?;

				verify_auth(ctx.clone(), &auth).await.map_err(|e| {
					error!("Failed to verify user authentication: {:?}", e);
					DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication verification failed")
						.with_field("user_auth")
						.with_reason(format!("Verification error: {:?}", e))
						.with_suggestion("Please check your authentication credentials")
						.to_rpc_error()
				})?;

				// Get main address from derived wallet
				let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
					error!("Failed to convert user ID to identity: {}", e);
					DetailedError::new(PARSE_ERROR_CODE, "Failed to parse user identity")
						.with_field("user_id")
						.with_reason(format!("Identity conversion error: {}", e))
						.to_rpc_error()
				})?;
				let omni_account = identity.to_omni_account(&params.client_id);

				let derived_pubkey = ctx
					.signer_client
					.request_wallet(ChainType::Evm, 0, *omni_account.as_ref())
					.await
					.map_err(|_| {
						error!("Failed to derive EVM address");
						DetailedError::new(INTERNAL_ERROR_CODE, "Failed to derive wallet address")
							.with_reason("Signer service failed to derive EVM address")
							.with_suggestion("Please try again later")
							.to_rpc_error()
					})?;
				pubkey_to_address(ChainType::Evm, &derived_pubkey).map_err(|_| {
					error!("Failed to convert derived pubkey to address");
					DetailedError::new(INTERNAL_ERROR_CODE, "Failed to convert public key")
						.with_reason("Public key to address conversion failed")
						.to_rpc_error()
				})?
			} else if let Some(client_auth) = &params.client_auth {
				// Client authentication provided (WildMeta)
				match client_auth {
					ClientAuth::WildmetaHl {
						agent_address,
						business_json,
						main_address,
						signature,
						login_type,
					} => {
						verify_wildmeta_signature(agent_address, business_json, signature)?;

						let business_data: serde_json::Value = serde_json::from_str(business_json)
							.map_err(|e| {
								error!("Failed to parse business_json: {:?}", e);
								DetailedError::new(PARSE_ERROR_CODE, "Failed to parse business JSON")
									.with_field("business_json")
									.with_reason(format!("JSON parse error: {:?}", e))
									.to_rpc_error()
							})?;

						let timestamp = business_data
							.get("timestamp")
							.and_then(|v| v.as_u64())
							.ok_or_else(|| {
								error!("Missing timestamp in business_json");
								DetailedError::new(MISSING_REQUIRED_FIELD_CODE, "Missing required field in business JSON")
									.with_field("timestamp")
									.with_expected("Unix timestamp as number")
									.with_reason("Business JSON must contain a 'timestamp' field")
									.to_rpc_error()
							})?;

						verify_payload_timestamp(
							&ctx.wildmeta_timestamp_storage,
							main_address,
							timestamp,
						)?;

						let linked = ctx
							.wildmeta_api
							.verify_hyperliquid_link(agent_address, main_address, *login_type)
							.await
							.map_err(|_| {
								error!("Failed to verify hyperliquid link");
								DetailedError::new(INTERNAL_ERROR_CODE, "Failed to verify account linkage")
									.with_reason("Hyperliquid link verification service error")
									.with_suggestion("Please try again later")
									.to_rpc_error()
							})?;

						if !linked {
							error!("Agent and main addresses are not linked");
							return Err(DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Account linkage verification failed")
								.with_field("agent_address")
								.with_expected("Linked to main address")
								.with_received("Not linked")
								.with_suggestion("Ensure the agent address is linked to your main address in Hyperliquid")
								.to_rpc_error());
						}

						main_address.clone()
					},
					_ => {
						error!("Invalid client auth type");
						return Err(DetailedError::new(INVALID_PARAMS_CODE, "Invalid client authentication type")
							.with_field("client_auth")
							.with_expected("WildmetaHl")
							.with_suggestion("Use a supported authentication method")
							.to_rpc_error());
					},
				}
			} else {
				error!("Either user_auth or client_auth must be provided");
				return Err(DetailedError::new(MISSING_REQUIRED_FIELD_CODE, "Missing authentication data")
					.with_expected("Either user_auth or client_auth")
					.with_suggestion("Provide either user_auth or client_auth for authentication")
					.to_rpc_error());
			};

			// Derive omni_account for signing (works for both auth methods)
			let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
				error!("Failed to convert user ID to identity: {}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Failed to parse user identity")
					.with_field("user_id")
					.with_reason(format!("Identity conversion error: {}", e))
					.to_rpc_error()
			})?;
			let omni_account = identity.to_omni_account(&params.client_id);

			let nonce = Utc::now().timestamp_millis() as u64;

			let hyperliquid_chain = if is_testnet_chain(params.chain_id) {
				"Testnet".to_string()
			} else {
				"Mainnet".to_string()
			};

			let (action, signature) = match params.action_type {
				HyperliquidActionType::ApproveAgent { agent_address, agent_name } => {
					let action = ApproveAgent {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						agent_address: validate_ethereum_address(&agent_address, "agent_address")
							.map_err(|e| e.to_rpc_error())?,
						agent_name,
						nonce,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::ApproveAgent(action), signature)
				},
				HyperliquidActionType::Withdraw3 { amount, destination } => {
                    let _ = validate_ethereum_address(&destination, "destination")
							.map_err(|e| e.to_rpc_error())?;
					let action = Withdraw3 {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						amount,
						time: nonce,
						destination,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::Withdraw3(action), signature)
				},
				HyperliquidActionType::ApproveBuilderFee { max_fee_rate, builder } => {
					let action = ApproveBuilderFee {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						max_fee_rate,
						builder: validate_ethereum_address(&builder, "builder")
							.map_err(|e| e.to_rpc_error())?,
						nonce,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::ApproveBuilderFee(action), signature)
				},
				HyperliquidActionType::SendAsset {
					destination,
					source_dex,
					destination_dex,
					token,
					amount,
					from_sub_account,
				} => {
					let _ = validate_ethereum_address(&destination, "destination")
						.map_err(|e| e.to_rpc_error())?;
					// Validate from_sub_account if it's not empty
					if !from_sub_account.is_empty() {
						let _ = validate_ethereum_address(&from_sub_account, "from_sub_account")
							.map_err(|e| e.to_rpc_error())?;
					}
					let action = SendAsset {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						destination,
						source_dex,
						destination_dex,
						token,
						amount,
						from_sub_account,
						nonce,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::SendAsset(action), signature)
				},
				HyperliquidActionType::UserDexAbstraction { user, enabled } => {
					let action = UserDexAbstraction {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						user: validate_ethereum_address(&user, "user")
							.map_err(|e| e.to_rpc_error())?,
						enabled,
						nonce,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::UserDexAbstraction(action), signature)
				},
			};

			Ok(GetHyperliquidSignatureDataResponse {
				main_address,
				hyperliquid_signature_data: HyperliquidSignatureData { action, nonce, signature },
			})
		})
		.expect("Failed to register omni_getHyperliquidSignatureData method");
}

async fn generate_eip712_signature<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	T: Eip712 + Send + Sync,
>(
	ctx: &RpcContext<CrossChainIntentExecutor>,
	action: &T,
	omni_account: &[u8; 32],
) -> RpcResult<String> {
	let message_hash = action.eip712_signing_hash();

	let signature_bytes = ctx
		.signer_client
		.request_signature(ChainType::Evm, 0, *omni_account, message_hash.to_vec())
		.await
		.map_err(|_| {
			error!("Failed to sign message");
			DetailedError::new(INTERNAL_ERROR_CODE, "Failed to generate signature")
				.with_reason("Signer service failed to sign EIP-712 message")
				.with_suggestion("Please try again later")
				.to_rpc_error()
		})?;

	Ok(hex_encode(&signature_bytes))
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloy::primitives::Address;
	use executor_primitives::VerificationCode;
	use std::str::FromStr;

	#[test]
	fn test_approve_agent_action_signature() {
		let action = ApproveAgent {
			signature_chain_id: 1,
			hyperliquid_chain: "Mainnet".to_string(),
			agent_address: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
			agent_name: Some("Test Agent".to_string()),
			nonce: 1234567890,
		};

		// Test domain generation
		let domain = action.domain();
		assert_eq!(domain.name, Some("HyperliquidSignTransaction".into()));
		assert_eq!(domain.version, Some("1".into()));
		assert_eq!(domain.chain_id, Some(alloy::primitives::U256::from(1)));

		// Test struct hash generation
		let struct_hash = action.struct_hash();
		assert_eq!(struct_hash.len(), 32);

		// Test EIP-712 signing hash generation
		let signing_hash = action.eip712_signing_hash();
		assert_eq!(signing_hash.len(), 32);
	}

	#[test]
	fn test_withdraw_action_signature() {
		let action = Withdraw3 {
			signature_chain_id: 1,
			hyperliquid_chain: "Mainnet".to_string(),
			amount: "100.0".to_string(),
			time: 1234567890,
			destination: "0x1234567890123456789012345678901234567890".to_string(),
		};

		// Test domain generation
		let domain = action.domain();
		assert_eq!(domain.name, Some("HyperliquidSignTransaction".into()));
		assert_eq!(domain.version, Some("1".into()));
		assert_eq!(domain.chain_id, Some(alloy::primitives::U256::from(1)));

		// Test struct hash generation
		let struct_hash = action.struct_hash();
		assert_eq!(struct_hash.len(), 32);

		// Test EIP-712 signing hash generation
		let signing_hash = action.eip712_signing_hash();
		assert_eq!(signing_hash.len(), 32);
	}

	#[test]
	fn test_approve_builder_fee_action_signature() {
		let action = ApproveBuilderFee {
			signature_chain_id: 1,
			hyperliquid_chain: "Mainnet".to_string(),
			max_fee_rate: "0.01".to_string(),
			builder: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
			nonce: 1234567890,
		};

		// Test domain generation
		let domain = action.domain();
		assert_eq!(domain.name, Some("HyperliquidSignTransaction".into()));
		assert_eq!(domain.version, Some("1".into()));
		assert_eq!(domain.chain_id, Some(alloy::primitives::U256::from(1)));

		// Test struct hash generation
		let struct_hash = action.struct_hash();
		assert_eq!(struct_hash.len(), 32);

		// Test EIP-712 signing hash generation
		let signing_hash = action.eip712_signing_hash();
		assert_eq!(signing_hash.len(), 32);
	}

	#[test]
	fn test_params_deserialization_approve_agent() {
		let json = r#"{
		"user_id": {"type": "email", "value": "test@example.com"},
		"user_auth": {"type": "email", "value": "123456"},
		"client_id": "test_client",
		"action_type": {
			"type": "approve_agent",
			"agent_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
			"agent_name": "My Trading Bot"
		},
		"chain_id": 42161
	}"#;

		let params: GetHyperliquidSignatureDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(params.user_id, UserId::Email(email) if email == "test@example.com"));
		let expected_verification_code = VerificationCode::from("123456");
		assert!(
			matches!(params.user_auth, Some(UserAuth::Email(code)) if code == expected_verification_code)
		);
		assert_eq!(params.client_id, "test_client");
		assert!(matches!(
			params.action_type,
			HyperliquidActionType::ApproveAgent { agent_address, agent_name }
			if agent_address == "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10" && agent_name == Some("My Trading Bot".to_string())
		));
		assert_eq!(params.chain_id, 42161);
	}

	#[test]
	fn test_params_deserialization_withdraw() {
		let json = r#"{
		"user_id": {"type": "email", "value": "test@example.com"},
		"user_auth": {"type": "email", "value": "123456"},
		"client_id": "test_client",
		"action_type": {
			"type": "withdraw3",
			"amount": "100.0",
			"destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
		},
		"chain_id": 42161
	}"#;

		let params: GetHyperliquidSignatureDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(
			params.action_type,
			HyperliquidActionType::Withdraw3 { amount, destination }
			if amount == "100.0" && destination == "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
		));
	}

	#[test]
	fn test_params_deserialization_approve_builder_fee() {
		let json = r#"{
		"user_id": {"type": "email", "value": "test@example.com"},
		"user_auth": {"type": "email", "value": "123456"},
		"client_id": "test_client",
		"action_type": {
			"type": "approve_builder_fee",
			"max_fee_rate": "0.01",
			"builder": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
		},
		"chain_id": 42161
	}"#;

		let params: GetHyperliquidSignatureDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(
			params.action_type,
			HyperliquidActionType::ApproveBuilderFee { max_fee_rate, builder }
			if max_fee_rate == "0.01" && builder == "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
		));
	}

	#[test]
	fn test_send_asset_action_signature() {
		let action = SendAsset {
			signature_chain_id: 998,
			hyperliquid_chain: "Testnet".to_string(),
			destination: "0x1234567890123456789012345678901234567890".to_string(),
			source_dex: "".to_string(),
			destination_dex: "".to_string(),
			token: "PURR:0xc4bf3f870c0e9465323c0b6ed28096c2".to_string(),
			amount: "100.0".to_string(),
			from_sub_account: "".to_string(),
			nonce: 1234567890,
		};

		// Test domain generation
		let domain = action.domain();
		assert_eq!(domain.name, Some("HyperliquidSignTransaction".into()));
		assert_eq!(domain.version, Some("1".into()));
		assert_eq!(domain.chain_id, Some(alloy::primitives::U256::from(998)));

		// Test struct hash generation
		let struct_hash = action.struct_hash();
		assert_eq!(struct_hash.len(), 32);

		// Test EIP-712 signing hash generation
		let signing_hash = action.eip712_signing_hash();
		assert_eq!(signing_hash.len(), 32);
	}

	#[test]
	fn test_params_deserialization_send_asset() {
		let json = r#"{
		"user_id": {"type": "email", "value": "test@example.com"},
		"user_auth": {"type": "email", "value": "123456"},
		"client_id": "test_client",
		"action_type": {
			"type": "send_asset",
			"destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
			"source_dex": "",
			"destination_dex": "spot",
			"token": "PURR:0xc4bf3f870c0e9465323c0b6ed28096c2",
			"amount": "50.5",
			"from_sub_account": ""
		},
		"chain_id": 998
	}"#;

		let params: GetHyperliquidSignatureDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(
			params.action_type,
			HyperliquidActionType::SendAsset {
				destination,
				source_dex,
				destination_dex,
				token,
				amount,
				from_sub_account
			}
			if destination == "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
				&& source_dex == ""
				&& destination_dex == "spot"
				&& token == "PURR:0xc4bf3f870c0e9465323c0b6ed28096c2"
				&& amount == "50.5"
				&& from_sub_account == ""
		));
		assert_eq!(params.chain_id, 998);
	}

	#[test]
	fn test_params_deserialization_send_asset_with_sub_account() {
		let json = r#"{
		"user_id": {"type": "email", "value": "test@example.com"},
		"user_auth": {"type": "email", "value": "123456"},
		"client_id": "test_client",
		"action_type": {
			"type": "send_asset",
			"destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
			"source_dex": "hyperliquid",
			"destination_dex": "",
			"token": "USDC:0x0",
			"amount": "1000.0",
			"from_sub_account": "0x9876543210987654321098765432109876543210"
		},
		"chain_id": 998
	}"#;

		let params: GetHyperliquidSignatureDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(
			params.action_type,
			HyperliquidActionType::SendAsset {
				destination,
				source_dex,
				destination_dex,
				token,
				amount,
				from_sub_account
			}
			if destination == "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
				&& source_dex == "hyperliquid"
				&& destination_dex == ""
				&& token == "USDC:0x0"
				&& amount == "1000.0"
				&& from_sub_account == "0x9876543210987654321098765432109876543210"
		));
	}

	#[test]
	fn test_user_dex_abstraction_action_signature() {
		let action = UserDexAbstraction {
			signature_chain_id: 1,
			hyperliquid_chain: "Mainnet".to_string(),
			user: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
			enabled: true,
			nonce: 1234567890,
		};

		// Test domain generation
		let domain = action.domain();
		assert_eq!(domain.name, Some("HyperliquidSignTransaction".into()));
		assert_eq!(domain.version, Some("1".into()));
		assert_eq!(domain.chain_id, Some(alloy::primitives::U256::from(1)));

		// Test struct hash generation
		let struct_hash = action.struct_hash();
		assert_eq!(struct_hash.len(), 32);

		// Test EIP-712 signing hash generation
		let signing_hash = action.eip712_signing_hash();
		assert_eq!(signing_hash.len(), 32);
	}

	#[test]
	fn test_params_deserialization_user_dex_abstraction() {
		let json = r#"{
		"user_id": {"type": "email", "value": "test@example.com"},
		"user_auth": {"type": "email", "value": "123456"},
		"client_id": "test_client",
		"action_type": {
			"type": "user_dex_abstraction",
			"user": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
			"enabled": true
		},
		"chain_id": 42161
	}"#;

		let params: GetHyperliquidSignatureDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(
			params.action_type,
			HyperliquidActionType::UserDexAbstraction { user, enabled }
			if user == "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10" && enabled == true
		));
		assert_eq!(params.chain_id, 42161);
	}

	#[test]
	fn test_is_testnet_chain() {
		// Test mainnet chain IDs
		assert!(!is_testnet_chain(1)); // Ethereum Mainnet
		assert!(!is_testnet_chain(56)); // BSC Mainnet
		assert!(!is_testnet_chain(137)); // Polygon Mainnet
		assert!(!is_testnet_chain(42161)); // Arbitrum Mainnet
		assert!(!is_testnet_chain(999)); // HyperEVM Mainnet

		// Test testnet chain IDs
		assert!(is_testnet_chain(11155111)); // Ethereum Sepolia
		assert!(is_testnet_chain(97)); // BSC Testnet
		assert!(is_testnet_chain(80001)); // Polygon Mumbai
		assert!(is_testnet_chain(421614)); // Arbitrum Sepolia
		assert!(is_testnet_chain(998)); // HyperEVM Testnet
		assert!(is_testnet_chain(1337)); // Local Anvil
		assert!(is_testnet_chain(31337)); // Local Anvil

		// Test unknown chain ID (defaults to mainnet)
		assert!(!is_testnet_chain(12345));
	}
}
