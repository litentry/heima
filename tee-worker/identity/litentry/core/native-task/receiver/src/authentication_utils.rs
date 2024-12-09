use crate::VerificationCode;
use alloc::{format, string::String, sync::Arc};
use codec::{Decode, Encode};
use ita_stf::{LitentryMultiSignature, TrustedCall};
use itp_types::parentchain::Index as ParentchainIndex;
use lc_data_providers::{google::GoogleOAuth2Client, DataProviderConfig};
use lc_identity_verification::web2::{email::VerificationCodeStore, google};
use lc_omni_account::InMemoryStore as OmniAccountStore;
use litentry_hex_utils::hex_encode;
use litentry_primitives::{Identity, ShardIdentifier, Web2IdentityType};
use sp_core::{blake2_256, crypto::AccountId32 as AccountId, H256};

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OAuth2Provider {
	Google,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct OAuth2Data {
	pub provider: OAuth2Provider,
	pub code: String,
	pub state: String,
	pub redirect_uri: String,
}

#[derive(Debug)]
pub enum AuthenticationError {
	Web3Error(String),
	EmailError(String),
	OAuth2Error(String),
}

pub fn verify_tca_web3_authentication(
	signature: &LitentryMultiSignature,
	call: &TrustedCall,
	nonce: ParentchainIndex,
	mrenclave: &[u8; 32],
	shard: &ShardIdentifier,
) -> Result<(), AuthenticationError> {
	let mut payload = call.encode();
	payload.append(&mut nonce.encode());
	payload.append(&mut mrenclave.encode());
	payload.append(&mut shard.encode());

	// The signature should be valid in either case:
	// 1. blake2_256(payload)
	// 2. Signature Prefix + blake2_256(payload)

	let hashed = blake2_256(&payload);

	let prettified_msg_hash = call.signature_message_prefix() + &hex_encode(&hashed);
	let prettified_msg_hash = prettified_msg_hash.as_bytes();

	// Most common signatures variants by clients are verified first (4 and 2).
	match signature.verify(prettified_msg_hash, call.sender_identity())
		|| signature.verify(&hashed, call.sender_identity())
	{
		true => Ok(()),
		false => Err(AuthenticationError::Web3Error(String::from("Invalid signature"))),
	}
}

pub fn verify_tca_email_authentication(
	sender_identity_hash: H256,
	omni_account: &AccountId,
	verification_code: VerificationCode,
) -> Result<(), AuthenticationError> {
	let Ok(Some(code)) = VerificationCodeStore::get(omni_account, sender_identity_hash) else {
		return Err(AuthenticationError::EmailError(String::from("Verification code not found")));
	};
	if code == verification_code {
		Ok(())
	} else {
		Err(AuthenticationError::EmailError(String::from("Invalid verification code")))
	}
}

pub fn verify_tca_oauth2_authentication(
	data_providers_config: Arc<DataProviderConfig>,
	sender_identity_hash: H256,
	omni_account: &AccountId,
	payload: OAuth2Data,
) -> Result<(), AuthenticationError> {
	match payload.provider {
		OAuth2Provider::Google =>
			verify_google_oauth2(data_providers_config, sender_identity_hash, omni_account, payload),
	}
}

fn verify_google_oauth2(
	data_providers_config: Arc<DataProviderConfig>,
	sender_identity_hash: H256,
	omni_account: &AccountId,
	payload: OAuth2Data,
) -> Result<(), AuthenticationError> {
	let state_verifier_result = google::OAuthStateStore::get(omni_account, sender_identity_hash);
	let Ok(Some(state_verifier)) = state_verifier_result else {
		return Err(AuthenticationError::OAuth2Error(String::from("State verifier not found")));
	};
	if state_verifier != payload.state {
		return Err(AuthenticationError::OAuth2Error(String::from("Invalid state")))
	}

	let mut google_client = GoogleOAuth2Client::new(
		data_providers_config.google_client_id.clone(),
		data_providers_config.google_client_secret.clone(),
	);
	let code = payload.code.clone();
	let redirect_uri = payload.redirect_uri.clone();
	let token = google_client.exchange_code_for_token(code, redirect_uri).map_err(|e| {
		AuthenticationError::OAuth2Error(format!("Failed to exchange code for token: {:?}", e))
	})?;
	let claims = google::decode_jwt(&token)
		.map_err(|e| AuthenticationError::OAuth2Error(format!("Failed to decode JWT: {:?}", e)))?;
	let google_identity = Identity::from_web2_account(&claims.email, Web2IdentityType::Google);
	let identity_omni_account = match OmniAccountStore::get_omni_account(google_identity.hash()) {
		Ok(Some(account_id)) => account_id,
		_ => google_identity.to_omni_account(),
	};

	match *omni_account == identity_omni_account {
		true => Ok(()),
		false => Err(AuthenticationError::OAuth2Error(String::from("Invalid identity member"))),
	}
}
