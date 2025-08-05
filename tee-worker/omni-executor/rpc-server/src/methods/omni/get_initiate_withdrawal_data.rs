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
pub struct GetInitiateWithdrawalDataParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub amount: String,
	pub destination: String,
	pub chain_id: ChainId,
}

#[derive(Serialize, Clone)]
pub struct GetInitiateWithdrawalDataResponse {
	pub main_address: String,
	pub initiate_withdrawal_data: InitiateWithdrawalData,
}

#[derive(Serialize, Clone)]
pub struct InitiateWithdrawalData {
	pub action: InitiateWithdrawalAction,
	pub nonce: u64,
	pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitiateWithdrawalAction {
	#[serde(serialize_with = "serialize_hex")]
	pub signature_chain_id: u64,
	pub hyperliquid_chain: String,
	pub amount: String,
	pub time: u64,
	pub destination: Address,
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

impl InitiateWithdrawalAction {
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

	fn eip712_signing_hash(&self) -> B256 {
		let mut digest_input = [0u8; 2 + 32 + 32];
		digest_input[0] = 0x19;
		digest_input[1] = 0x01;
		digest_input[2..34].copy_from_slice(&self.domain().hash_struct()[..]);
		digest_input[34..66].copy_from_slice(&self.struct_hash()[..]);
		keccak256(digest_input)
	}
}

pub fn register_get_initiate_withdrawal_data(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getInitiateWithdrawalData", |params, ctx, _| async move {
			let params = params.parse::<GetInitiateWithdrawalDataParams>().map_err(|e| {
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

			let action = InitiateWithdrawalAction {
				signature_chain_id: params.chain_id,
				hyperliquid_chain,
				amount: params.amount,
				time: nonce,
				destination: Address::from_str(&params.destination).map_err(|_| {
					error!("Invalid destination address format");
					ErrorObject::from(ErrorCode::ParseError)
				})?,
			};

			let signature = generate_eip712_signature(&ctx, &action, omni_account.as_ref()).await?;

			Ok(GetInitiateWithdrawalDataResponse {
				main_address,
				initiate_withdrawal_data: InitiateWithdrawalData { action, nonce, signature },
			})
		})
		.expect("Failed to register omni_getInitiateWithdrawalData method");
}

fn serialize_hex<S>(val: &u64, s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&format!("0x{val:x}"))
}

async fn generate_eip712_signature(
	ctx: &RpcContext,
	action: &InitiateWithdrawalAction,
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
	fn test_initiate_withdrawal_action() {
		let action = InitiateWithdrawalAction {
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
	fn test_get_initiate_withdrawal_data_params_deserialization() {
		let json = r#"{
			"user_id": {"type": "email", "value": "test@example.com"},
			"user_auth": {"type": "email", "value": "123456"},
			"client_id": "test_client",
			"amount": "100.0",
			"destination": "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10",
			"chain_id": 42161
		}"#;

		let params: GetInitiateWithdrawalDataParams = serde_json::from_str(json).unwrap();

		assert!(matches!(params.user_id, UserId::Email(email) if email == "test@example.com"));
		let expected_verification_code = VerificationCode::from("123456");
		assert!(
			matches!(params.user_auth, UserAuth::Email(code) if code == expected_verification_code)
		);
		assert_eq!(params.client_id, "test_client");
		assert_eq!(params.amount, "100.0");
		assert_eq!(params.destination, "0x742d35Cc6634C0532925a3b844Bc9e7595f02A10");
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
