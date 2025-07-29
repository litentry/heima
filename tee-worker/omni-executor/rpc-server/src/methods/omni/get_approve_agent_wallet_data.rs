use crate::{
	error_code::*, server::RpcContext, verify_auth::verify_email_authentication, ErrorCode,
};
use alloy::{
	dyn_abi::Eip712Domain,
	primitives::{keccak256, Address, B256},
	sol_types::{eip712_domain, SolValue},
};
use chrono::Utc;
use executor_primitives::{
	utils::hex::hex_encode, ChainId, Identity, UserAuth, UserId, Web2IdentityType,
};
use jsonrpsee::{types::ErrorObject, RpcModule};
use pumpx::pubkey_to_address;
use serde::{Deserialize, Serialize, Serializer};
use signer_client::ChainType;
use std::str::FromStr;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct GetApproveAgentWalletDataParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub agent_address: String,
	pub agent_name: Option<String>,
	pub chain_id: ChainId,
	pub is_testnet: bool,
}

#[derive(Serialize, Clone)]
pub struct GetApproveAgentWalletDataResponse {
	pub main_address: String,
	pub approve_agent_data: ApproveAgentData,
}

#[derive(Serialize, Clone)]
pub struct ApproveAgentData {
	pub action: ApproveAgentAction,
	pub nonce: u64,
	pub signature: String,
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

impl ApproveAgentAction {
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

	fn eip712_signing_hash(&self) -> B256 {
		let mut digest_input = [0u8; 2 + 32 + 32];
		digest_input[0] = 0x19;
		digest_input[1] = 0x01;
		digest_input[2..34].copy_from_slice(&self.domain().hash_struct()[..]);
		digest_input[34..66].copy_from_slice(&self.struct_hash()[..]);
		keccak256(digest_input)
	}
}

pub fn register_get_approve_agent_wallet_data(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getApproveAgentWalletData", |params, ctx, _| async move {
			let params = params.parse::<GetApproveAgentWalletDataParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				ErrorCode::ParseError
			})?;

			let (email, verification_code) = match &params.user_auth {
				UserAuth::Email(code) => {
					let UserId::Email(email) = &params.user_id else {
						error!("User ID must be an email for Email authentication");
						return Err(ErrorObject::from(ErrorCode::ParseError));
					};
					(email.clone(), code.clone())
				},
				_ => {
					error!("Only email authentication is supported for this method");
					return Err(ErrorObject::from(ErrorCode::InvalidParams));
				},
			};

			verify_email_authentication(ctx.clone(), &params.client_id, &email, &verification_code)
				.map_err(|_| {
					error!("Failed to verify email authentication");
					ErrorObject::from(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE))
				})?;

			let identity = Identity::from_web2_account(&email, Web2IdentityType::Email);
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

			let hyperliquid_chain =
				if params.is_testnet { "Testnet".to_string() } else { "Mainnet".to_string() };

			let action = ApproveAgentAction {
				signature_chain_id: params.chain_id,
				hyperliquid_chain,
				agent_address: Address::from_str(&params.agent_address).map_err(|_| {
					error!("Invalid agent address format");
					ErrorObject::from(ErrorCode::ParseError)
				})?,
				agent_name: params.agent_name.clone(),
				nonce,
			};

			let signature = generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;

			Ok(GetApproveAgentWalletDataResponse {
				main_address,
				approve_agent_data: ApproveAgentData { action, nonce, signature },
			})
		})
		.expect("Failed to register omni_getApproveAgentWalletData method");
}

fn serialize_hex<S>(val: &u64, s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&format!("0x{val:x}"))
}

async fn generate_eip712_signature(
	ctx: &RpcContext,
	action: &ApproveAgentAction,
	omni_account: &[u8; 32],
) -> Result<String, ErrorObject<'static>> {
	let message_hash = action.eip712_signing_hash();

	// Request signature from signer client
	let signature_bytes = ctx
		.signer_client
		.request_signature(ChainType::Evm, 0, *omni_account, message_hash.to_vec())
		.await
		.map_err(|_| {
			error!("Failed to sign message");
			ErrorObject::from(ErrorCode::InternalError)
		})?;

	// Convert signature to hex string with 0x prefix
	Ok(hex_encode(&signature_bytes))
}

#[cfg(test)]
mod tests {
	use super::*;
	use executor_primitives::VerificationCode;

	#[test]
	fn test_approve_agent_action() {
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
	fn test_serialize_hex() {
		// Test mainnet chain ID
		let mut buffer = Vec::new();
		let mut serializer = serde_json::Serializer::new(&mut buffer);
		serialize_hex(&1u64, &mut serializer).unwrap();
		assert_eq!(String::from_utf8(buffer).unwrap(), "\"0x1\"");

		// Test Arbitrum chain ID
		let mut buffer = Vec::new();
		let mut serializer = serde_json::Serializer::new(&mut buffer);
		serialize_hex(&42161u64, &mut serializer).unwrap();
		assert_eq!(String::from_utf8(buffer).unwrap(), "\"0xa4b1\"");
	}

	#[test]
	fn test_approve_agent_action_serialization() {
		let action = ApproveAgentAction {
			signature_chain_id: 42161, // Arbitrum
			hyperliquid_chain: "Testnet".to_string(),
			agent_address: Address::from_str("0x742d35Cc6634C0532925a3b844Bc9e7595f02A10").unwrap(),
			agent_name: None,
			nonce: 1700000000000,
		};

		let json = serde_json::to_string(&action).unwrap();
		let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

		assert_eq!(parsed["signatureChainId"], "0xa4b1");
		assert_eq!(parsed["hyperliquidChain"], "Testnet");
		assert_eq!(parsed["agentAddress"], "0x742d35cc6634c0532925a3b844bc9e7595f02a10");
		assert_eq!(parsed["agentName"], serde_json::Value::Null);
		assert_eq!(parsed["nonce"], 1700000000000u64);
	}

	#[test]
	fn test_get_approve_agent_wallet_data_params_deserialization() {
		let json = r#"{
			"user_id": {"type": "email", "value": "test@example.com"},
			"user_auth": {"type": "email", "value": "123456"},
			"client_id": "test_client",
			"agent_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
			"agent_name": "My Trading Bot",
			"chain_id": 42161,
			"is_testnet": true
		}"#;

		let params: GetApproveAgentWalletDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(params.user_id, UserId::Email(email) if email == "test@example.com"));
		let expected_verification_code = VerificationCode::from("123456");
		assert!(
			matches!(params.user_auth, UserAuth::Email(code) if code == expected_verification_code)
		);
		assert_eq!(params.client_id, "test_client");
		assert_eq!(params.agent_address, "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10");
		assert_eq!(params.agent_name, Some("My Trading Bot".to_string()));
		assert_eq!(params.chain_id, 42161);
		assert!(params.is_testnet);
	}

	#[test]
	fn test_response_serialization() {
		let response = GetApproveAgentWalletDataResponse {
			main_address: "0x1234567890123456789012345678901234567890".to_string(),
			approve_agent_data: ApproveAgentData {
				action: ApproveAgentAction {
					signature_chain_id: 1,
					hyperliquid_chain: "Mainnet".to_string(),
					agent_address: Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
						.unwrap(),
					agent_name: Some("Test Bot".to_string()),
					nonce: 1700000000000,
				},
				nonce: 1700000000000,
				signature: "0x1234567890abcdef".to_string(),
			},
		};

		let json = serde_json::to_string(&response).unwrap();
		let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

		assert_eq!(parsed["main_address"], "0x1234567890123456789012345678901234567890");
		assert_eq!(parsed["approve_agent_data"]["action"]["signatureChainId"], "0x1");
		assert_eq!(parsed["approve_agent_data"]["action"]["hyperliquidChain"], "Mainnet");
		assert_eq!(
			parsed["approve_agent_data"]["action"]["agentAddress"],
			"0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
		);
		assert_eq!(parsed["approve_agent_data"]["action"]["agentName"], "Test Bot");
		assert_eq!(parsed["approve_agent_data"]["nonce"], 1700000000000u64);
		assert_eq!(parsed["approve_agent_data"]["signature"], "0x1234567890abcdef");
	}

	#[test]
	fn test_testnet_vs_mainnet_chain_params() {
		// Test mainnet parameters
		let params_mainnet = GetApproveAgentWalletDataParams {
			user_id: UserId::Email("test@example.com".to_string()),
			user_auth: UserAuth::Email("123456".into()),
			client_id: "test".to_string(),
			agent_address: "0x0000000000000000000000000000000000000000".to_string(),
			agent_name: None,
			chain_id: 1,
			is_testnet: false,
		};

		let hyperliquid_chain = if params_mainnet.is_testnet { "Testnet" } else { "Mainnet" };
		assert_eq!(hyperliquid_chain, "Mainnet");

		// Test testnet parameters
		let params_testnet = GetApproveAgentWalletDataParams {
			user_id: UserId::Email("test@example.com".to_string()),
			user_auth: UserAuth::Email("123456".into()),
			client_id: "test".to_string(),
			agent_address: "0x0000000000000000000000000000000000000000".to_string(),
			agent_name: None,
			chain_id: 42161,
			is_testnet: true,
		};

		let hyperliquid_chain = if params_testnet.is_testnet { "Testnet" } else { "Mainnet" };
		assert_eq!(hyperliquid_chain, "Testnet");
	}
}
