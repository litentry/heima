use crate::{
	error_code::*, server::RpcContext, verify_auth::verify_email_authentication, ErrorCode,
};
use alloy::{
	dyn_abi::Eip712Domain,
	primitives::{keccak256, Address, B256},
	sol_types::{eip712_domain, SolValue},
};
use chrono::Utc;
use executor_primitives::{utils::hex::hex_encode, ChainId, Identity, UserAuth, UserId};
use jsonrpsee::{types::ErrorObject, RpcModule};
use pumpx::pubkey_to_address;
use serde::{Deserialize, Serialize, Serializer};
use signer_client::ChainType;
use std::{convert::TryFrom, str::FromStr};
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct GetHyperliquidSignatureDataParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub action_type: HyperliquidActionType,
	pub chain_id: ChainId,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HyperliquidActionType {
	ApproveAgent { agent_address: String, agent_name: Option<String> },
	Withdraw { amount: String, destination: String },
	ApproveBuilderFee { max_fee_rate: String, builder: String },
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
	ApproveAgent(ApproveAgentAction),
	Withdraw(WithdrawAction),
	ApproveBuilderFee(ApproveBuilderFeeAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAgentAction {
	#[serde(serialize_with = "serialize_hex")]
	pub signature_chain_id: u64,
	pub hyperliquid_chain: String,
	pub agent_address: Address,
	pub agent_name: Option<String>,
	pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawAction {
	#[serde(serialize_with = "serialize_hex")]
	pub signature_chain_id: u64,
	pub hyperliquid_chain: String,
	pub amount: String,
	pub time: u64,
	pub destination: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveBuilderFeeAction {
	#[serde(serialize_with = "serialize_hex")]
	pub signature_chain_id: u64,
	pub hyperliquid_chain: String,
	pub max_fee_rate: String,
	pub builder: Address,
	pub nonce: u64,
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

trait HyperliquidEip712Signature {
	fn domain(&self) -> Eip712Domain;
	fn struct_hash(&self) -> B256;
	fn eip712_signing_hash(&self) -> B256 {
		let mut digest_input = [0u8; 2 + 32 + 32];
		digest_input[0] = 0x19;
		digest_input[1] = 0x01;
		digest_input[2..34].copy_from_slice(&self.domain().hash_struct()[..]);
		digest_input[34..66].copy_from_slice(&self.struct_hash()[..]);
		keccak256(digest_input)
	}
}

impl HyperliquidEip712Signature for ApproveAgentAction {
	fn domain(&self) -> Eip712Domain {
		eip712_domain! {
			name: "HyperliquidSignTransaction",
			version: "1",
			chain_id: self.signature_chain_id,
			verifying_contract: Address::ZERO,
		}
	}

	fn struct_hash(&self) -> B256 {
		let items = (
            keccak256("HyperliquidTransaction:ApproveAgent(string hyperliquidChain,address agentAddress,string agentName,uint64 nonce)"),
            keccak256(&self.hyperliquid_chain),
            &self.agent_address,
            keccak256(self.agent_name.as_deref().unwrap_or("")),
            &self.nonce
        );
		keccak256(items.abi_encode())
	}
}

impl HyperliquidEip712Signature for WithdrawAction {
	fn domain(&self) -> Eip712Domain {
		eip712_domain! {
			name: "HyperliquidSignTransaction",
			version: "1",
			chain_id: self.signature_chain_id,
			verifying_contract: Address::ZERO,
		}
	}

	fn struct_hash(&self) -> B256 {
		let items = (
            keccak256("HyperliquidTransaction:Withdraw3(string hyperliquidChain,string amount,uint64 time,address destination)"),
            keccak256(&self.hyperliquid_chain),
            keccak256(&self.amount),
            &self.time,
            &self.destination
        );
		keccak256(items.abi_encode())
	}
}

impl HyperliquidEip712Signature for ApproveBuilderFeeAction {
	fn domain(&self) -> Eip712Domain {
		eip712_domain! {
			name: "HyperliquidSignTransaction",
			version: "1",
			chain_id: self.signature_chain_id,
			verifying_contract: Address::ZERO,
		}
	}

	fn struct_hash(&self) -> B256 {
		let items = (
            keccak256("HyperliquidTransaction:ApproveBuilderFee(string hyperliquidChain,string maxFeeRate,address builder,uint64 nonce)"),
            keccak256(&self.hyperliquid_chain),
            keccak256(&self.max_fee_rate),
            &self.builder,
            &self.nonce
        );
		keccak256(items.abi_encode())
	}
}

pub fn register_get_hyperliquid_signature_data(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getHyperliquidSignatureData", |params, ctx, _| async move {
			let params = params.parse::<GetHyperliquidSignatureDataParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				ErrorCode::ParseError
			})?;

			let verification_code = match &params.user_auth {
				UserAuth::Email(code) => code.clone(),
				_ => {
					error!("Only email authentication is supported for this method");
					return Err(ErrorObject::from(ErrorCode::InvalidParams));
				},
			};

			let email = match &params.user_id {
				UserId::Email(email) => email.clone(),
				_ => {
					error!("User ID must be an email for this method");
					return Err(ErrorObject::from(ErrorCode::ParseError));
				},
			};

			verify_email_authentication(ctx.clone(), &params.client_id, &email, &verification_code)
				.map_err(|_| {
					error!("Failed to verify email authentication");
					ErrorObject::from(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE))
				})?;

			let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
				error!("Failed to convert user ID to identity: {}", e);
				ErrorObject::from(ErrorCode::ParseError)
			})?;
			let omni_account = identity.to_omni_account(&params.client_id);

			let derived_pubkey = ctx
				.signer_client
				.request_wallet(ChainType::Evm, 0, *omni_account.as_ref())
				.await
				.map_err(|_| {
					error!("Failed to derive EVM address");
					ErrorObject::from(ErrorCode::InternalError)
				})?;
			let main_address =
				pubkey_to_address(ChainType::Evm, &derived_pubkey).map_err(|_| {
					error!("Failed to convert derived pubkey to address");
					ErrorObject::from(ErrorCode::InternalError)
				})?;

			let nonce = Utc::now().timestamp_millis() as u64;

			let hyperliquid_chain = if is_testnet_chain(params.chain_id) {
				"Testnet".to_string()
			} else {
				"Mainnet".to_string()
			};

			let (action, signature) = match params.action_type {
				HyperliquidActionType::ApproveAgent { agent_address, agent_name } => {
					let action = ApproveAgentAction {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						agent_address: Address::from_str(&agent_address).map_err(|_| {
							error!("Invalid agent address format");
							ErrorObject::from(ErrorCode::ParseError)
						})?,
						agent_name,
						nonce,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::ApproveAgent(action), signature)
				},
				HyperliquidActionType::Withdraw { amount, destination } => {
					let action = WithdrawAction {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						amount,
						time: nonce,
						destination: Address::from_str(&destination).map_err(|_| {
							error!("Invalid destination address format");
							ErrorObject::from(ErrorCode::ParseError)
						})?,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::Withdraw(action), signature)
				},
				HyperliquidActionType::ApproveBuilderFee { max_fee_rate, builder } => {
					let action = ApproveBuilderFeeAction {
						signature_chain_id: params.chain_id,
						hyperliquid_chain,
						max_fee_rate,
						builder: Address::from_str(&builder).map_err(|_| {
							error!("Invalid builder address format");
							ErrorObject::from(ErrorCode::ParseError)
						})?,
						nonce,
					};
					let signature =
						generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;
					(HyperliquidAction::ApproveBuilderFee(action), signature)
				},
			};

			Ok(GetHyperliquidSignatureDataResponse {
				main_address,
				hyperliquid_signature_data: HyperliquidSignatureData { action, nonce, signature },
			})
		})
		.expect("Failed to register omni_getHyperliquidSignatureData method");
}

fn serialize_hex<S>(val: &u64, s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&format!("0x{val:x}"))
}

async fn generate_eip712_signature<T: HyperliquidEip712Signature>(
	ctx: &RpcContext,
	action: &T,
	omni_account: &[u8; 32],
) -> Result<String, ErrorObject<'static>> {
	let message_hash = action.eip712_signing_hash();

	let signature_bytes = ctx
		.signer_client
		.request_signature(ChainType::Evm, 0, *omni_account, message_hash.to_vec())
		.await
		.map_err(|_| {
			error!("Failed to sign message");
			ErrorObject::from(ErrorCode::InternalError)
		})?;

	Ok(hex_encode(&signature_bytes))
}

#[cfg(test)]
mod tests {
	use super::*;
	use executor_primitives::VerificationCode;

	#[test]
	fn test_approve_agent_action_signature() {
		let action = ApproveAgentAction {
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
		let action = WithdrawAction {
			signature_chain_id: 1,
			hyperliquid_chain: "Mainnet".to_string(),
			amount: "100.0".to_string(),
			time: 1234567890,
			destination: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
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
		let action = ApproveBuilderFeeAction {
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
			matches!(params.user_auth, UserAuth::Email(code) if code == expected_verification_code)
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
				"type": "withdraw",
				"amount": "100.0",
				"destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10"
			},
			"chain_id": 42161
		}"#;

		let params: GetHyperliquidSignatureDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(
			params.action_type,
			HyperliquidActionType::Withdraw { amount, destination }
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
