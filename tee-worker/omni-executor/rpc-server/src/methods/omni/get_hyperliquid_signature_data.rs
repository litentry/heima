use crate::{
	detailed_error::DetailedError,
	server::RpcContext,
	utils::auth::{verify_payload_timestamp, verify_wildmeta_signature},
	utils::types::RpcResultExt,
	utils::validation::{parse_rpc_params, validate_evm_address},
	verify_auth::verify_auth,
	RpcResult,
};
use chrono::Utc;
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::passkey::{AttestationResult, PasskeyVerifier};
use executor_primitives::{
	to_omni_auth, utils::hex::hex_encode, ChainId, ClientAuth, UserAuth, UserId,
};
use executor_storage::{PasskeyChallengeError, PasskeyChallengeStorage, PasskeyStorage};
use hyperliquid_rust_sdk::{
	ApproveAgent, ApproveBuilderFee, Eip712, SendAsset, UserDexAbstraction, Withdraw3,
};
use jsonrpsee::RpcModule;
use pumpx::pubkey_to_address;
use serde::{Deserialize, Serialize};
use signer_client::ChainType;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct GetHyperliquidSignatureDataParams {
	pub user_id: UserId,
	pub user_auth: Option<UserAuth>,
	pub client_id: String,
	pub client_auth: Option<ClientAuth>,
	pub action_type: HyperliquidActionType,
	pub chain_id: ChainId,
	pub attach_passkey: Option<AttachPasskeyData>,
}

#[derive(Debug, Deserialize)]
pub struct AttachPasskeyData {
	/// The attestation object from the WebAuthn registration ceremony (base64url encoded)
	/// This contains the credential public key, credential ID, and attestation statement
	pub attestation_object: String,
	/// The client data JSON from the WebAuthn registration ceremony (base64url encoded)
	/// This contains the challenge, origin, and other client-side data
	pub client_data_json: String,
	pub alias_name: Option<String>, // Optional alias name for the passkey
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
			let params = parse_rpc_params::<GetHyperliquidSignatureDataParams>(params)?;

			debug!("Received omni_getHyperliquidSignatureData, params: {:?}", params);

			// Make sure `user_id` is non-evm type
			if matches!(params.user_id, UserId::Evm(_)) {
				error!("Invalid user_id type, expected non-evm");
				return Err(
					DetailedError::invalid_params("user_id", "expect non-evm").to_rpc_error()
				);
			}

			let omni_account = params
				.user_id
				.to_omni_account(&params.client_id)
				.map_err_parse("Failed to convert to omni_account")?;

			if let Some(attach_passkey_data) = &params.attach_passkey {
				// Reject UserId::Passkey type - passkeys cannot be attached to passkey identities
				if matches!(params.user_id, UserId::Passkey(_)) {
					error!("Cannot attach passkey to a Passkey user_id type");
					return Err(DetailedError::invalid_params(
						"user_id",
						"UserId::Passkey type is not allowed for passkey attachment",
					)
					.with_reason("Passkeys can only be attached to non-passkey identity types")
					.with_suggestion("Use a different identity type (e.g. Email) as user_id")
					.to_rpc_error());
				}

				let user_auth = params.user_auth.as_ref().ok_or_else(|| {
					error!("user_auth is required when attach_passkey is provided");
					DetailedError::invalid_params("user_auth", "required when attaching a passkey")
						.to_rpc_error()
				})?;
				let auth = to_omni_auth(user_auth, &params.user_id, &params.client_id)
					.map_err_parse("Failed to convert to OmniAuth")?;

				verify_auth(ctx.clone(), &auth).await.map_err(|e| {
					error!("Failed to verify user authentication: {:?}", e);
					e.to_detailed_error().to_rpc_error()
				})?;

				// Determine expected origin based on client_id
				let expected_origin = super::get_origin_for_client(&params.client_id);

				// Verify client data JSON and consume challenge
				let challenge_storage = PasskeyChallengeStorage::new(ctx.storage_db.clone());
				PasskeyVerifier::verify_client_data_json(
					&attach_passkey_data.client_data_json,
					omni_account.as_ref(),
					expected_origin,
					"webauthn.create", // For passkey registration/attachment
					|challenge, omni_account| {
						challenge_storage
							.verify_and_consume_challenge(challenge, &(*omni_account).into())
							.map_err(|e| {
								match e {
									PasskeyChallengeError::ChallengeNotFound => {
										error!("Challenge not found for passkey attachment");
									},
									PasskeyChallengeError::ChallengeExpired => {
										error!("Challenge expired for passkey attachment");
									},
									PasskeyChallengeError::InvalidChallenge => {
										error!("Invalid challenge for passkey attachment");
									},
									_ => {
										error!("Challenge verification failed during passkey attachment: {:?}", e);
									},
								}
								executor_crypto::passkey::PasskeyError::ChallengeVerificationFailed
							})
					},
				)
				.map_err_internal("Client data verification failed")?;

				// Verify attestation and extract credential_id and public_key
				let AttestationResult { credential_id, public_key } =
					PasskeyVerifier::verify_attestation(&attach_passkey_data.attestation_object)
						.map_err_parse("Attestation verification failed")?;

				// Store public key as direct SEC1 bytes for direct usage without parsing
				let public_key_sec1_bytes = public_key.verifying_key.to_sec1_bytes();

				// Store the new passkey to the authenticated user's account
				let passkey_storage = PasskeyStorage::new(ctx.storage_db.clone());
				passkey_storage
					.add_passkey(
						&omni_account,
						&credential_id,
						&public_key_sec1_bytes,
						attach_passkey_data.alias_name.clone(),
					)
					.map_err_internal("Failed to attach passkey")?;
			}

			// Unified authentication logic
			let main_address = if let Some(user_auth) = &params.user_auth {
				// User authentication provided
				let auth = to_omni_auth(user_auth, &params.user_id, &params.client_id)
					.map_err_parse("Failed to convert to OmniAuth")?;

				verify_auth(ctx.clone(), &auth).await.map_err(|e| {
					error!("Failed to verify user authentication: {:?}", e);
					e.to_detailed_error().to_rpc_error()
				})?;

				// Get main address from derived wallet
				ctx.signer_client
					.request_wallet(ChainType::Evm, 0, *omni_account.as_ref())
					.await
					.map_err(|_| {
						error!("Failed to derive EVM address");
						DetailedError::signer_service_error().to_rpc_error()
					})
					.and_then(|pk| {
						pubkey_to_address(ChainType::Evm, &pk)
							.map_err_internal("Failed to convert pubkey to address")
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
							.map_err_parse("Failed to parse business_json")?;

						let timestamp = business_data
							.get("timestamp")
							.and_then(|v| v.as_u64())
							.ok_or_else(|| {
								error!("Missing timestamp in business_json");
								DetailedError::invalid_params("business_json", "missing timestamp")
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
								let msg = "Failed to verify hyperliquid link";
								error!(msg);
								DetailedError::wildmeta_service_error("verify_hyperliquid_link")
									.to_rpc_error()
							})?;

						if !linked {
							let msg = "Agent and main addresses are not linked";
							error!(msg);
							return Err(DetailedError::internal_error(msg).to_rpc_error());
						}

						main_address.clone()
					},
					_ => {
						error!("Invalid client auth type");
						return Err(DetailedError::invalid_params(
							"client_auth",
							"expect wildmeta_hl",
						)
						.to_rpc_error());
					},
				}
			} else {
				error!("Either user_auth or client_auth must be provided");
				return Err(DetailedError::invalid_params(
					"auth",
					"expect either user_auth or client_auth",
				)
				.to_rpc_error());
			};

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
						agent_address: validate_evm_address(&agent_address, "agent_address")?,
						agent_name,
						nonce,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::ApproveAgent(action), signature)
				},
				HyperliquidActionType::Withdraw3 { amount, destination } => {
					let _ = validate_evm_address(&destination, "destination")?;
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
						builder: validate_evm_address(&builder, "builder")?,
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
					let _ = validate_evm_address(&destination, "destination")?;
					// Validate from_sub_account if it's not empty
					if !from_sub_account.is_empty() {
						let _ = validate_evm_address(&from_sub_account, "from_sub_account")?;
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
						user: validate_evm_address(&user, "user")?,
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
			let msg = "Failed to sign message";
			error!(msg);
			DetailedError::signer_service_error().to_rpc_error()
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
				&& source_dex.is_empty()
				&& destination_dex == "spot"
				&& token == "PURR:0xc4bf3f870c0e9465323c0b6ed28096c2"
				&& amount == "50.5"
				&& from_sub_account.is_empty()
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
				&& destination_dex.is_empty()
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
			if user == "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10" && enabled
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
