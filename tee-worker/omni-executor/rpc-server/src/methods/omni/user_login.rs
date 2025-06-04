use crate::{
	error_code::*, server::RpcContext, verify_auth::verify_auth, Deserialize, ErrorCode, Serialize,
};
use chrono::{Days, Utc};
use executor_crypto::jwt;
use executor_primitives::{utils::hex::ToHexPrefixed, ClientAuth, OmniAuth, UserAuth, UserId};
use executor_storage::{HeimaJwtStorage, Storage};
use heima_authentication::auth_token::{
	AuthOptions, AuthTokenClaims, AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_EXPIRATION_DAYS,
	AUTH_TOKEN_ID_TYPE,
};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::{debug, error};

#[derive(Debug, Deserialize, Clone)]
pub struct UserLoginParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub client_auth: Option<ClientAuth>,
}

#[derive(Serialize, Clone)]
pub struct UserLoginResponse {
	pub access_token: String,
	pub id_token: String,
}

#[derive(Serialize, Clone)]
pub struct HeimaPostLoginParams {
	pub user_id: UserId,
	pub client_id: String,
	pub user_auth: UserAuth,
	pub heima_login_success: bool,
	pub access_token: String,
}

impl TryFrom<UserLoginParams> for OmniAuth {
	type Error = ErrorCode;

	fn try_from(p: UserLoginParams) -> Result<Self, Self::Error> {
		let omni_auth = match p.user_auth {
			UserAuth::Email(code) => {
				let UserId::Email(email) = p.user_id else {
					error!("User ID must be an email for Email authentication");
					return Err(ErrorCode::ParseError);
				};
				OmniAuth::Email(email, code)
			},
			UserAuth::Substrate(signature) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					ErrorCode::ParseError
				})?;
				if !identity.is_substrate() {
					error!("User ID must be a Substrate identity for Substrate authentication");
					return Err(ErrorCode::ParseError);
				}
				OmniAuth::Web3(identity, signature.into())
			},
			UserAuth::Evm(signature) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					ErrorCode::ParseError
				})?;
				if !identity.is_evm() {
					error!("User ID must be an EVM identity for EVM authentication");
					return Err(ErrorCode::ParseError);
				}
				OmniAuth::Web3(identity, signature.into())
			},
			UserAuth::Solana(signature) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					ErrorCode::ParseError
				})?;
				if !identity.is_solana() {
					error!("User ID must be a Solana identity for Solana authentication");
					return Err(ErrorCode::ParseError);
				}
				OmniAuth::Web3(identity, signature.into())
			},
			UserAuth::Bitcoin(signature) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					ErrorCode::ParseError
				})?;
				if !identity.is_bitcoin() {
					error!("User ID must be a Bitcoin identity for Bitcoin authentication");
					return Err(ErrorCode::ParseError);
				}
				OmniAuth::Web3(identity, signature.into())
			},
			UserAuth::AuthToken(token) => OmniAuth::AuthToken(token),
			UserAuth::OAuth2(data) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					ErrorCode::ParseError
				})?;
				OmniAuth::OAuth2(identity, data)
			},
		};

		Ok(omni_auth)
	}
}

pub fn register_user_login(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_userLogin", |params, ctx, _| async move {
			let params = params.parse::<UserLoginParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				ErrorCode::ParseError
			})?;
			let auth = OmniAuth::try_from(params.clone()).map_err(|e| {
				error!("Failed to convert params to OmniAuth: {:?}", e);
				ErrorCode::ParseError
			})?;
			verify_auth(ctx.clone(), &auth).await.map_err(|_| {
				error!("Failed to verify auth: {:?}", auth);
				ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
			})?;
			let identity = Identity::try_from(params.user_id.clone()).map_err(|_| {
				error!("Invalid user ID format");
				ErrorCode::ParseError
			})?;
			let id_token = create_jwt_for_user(
				identity.clone(),
				AUTH_TOKEN_ID_TYPE,
				&params.client_id,
				&ctx.jwt_rsa_private_key,
			)
			.map_err(|_| {
				error!("Failed to create access token for user");
				ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
			})?;
			let access_token = create_jwt_for_user(
				identity.clone(),
				AUTH_TOKEN_ACCESS_TYPE,
				&params.client_id,
				&ctx.jwt_rsa_private_key,
			)
			.map_err(|_| {
				error!("Failed to create access token for user");
				ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
			})?;

			// // TODO: create constants for client IDs
			if params.client_id == "wildmeta" {
				let Some(ClientAuth::Wildmeta { google_code, invite_code }) = params.client_auth
				else {
					error!("Client authentication data is missing for Pumpx client");
					return Err(ErrorCode::ParseError.into());
				};

				debug!("Wildmeta client login with Google code: {}", google_code);

				debug!("Invite code: {:?}", invite_code);

				let _heima_post_login_params = HeimaPostLoginParams {
					user_id: params.user_id,
					client_id: params.client_id,
					user_auth: params.user_auth,
					heima_login_success: true,
					access_token: access_token.clone(),
				};

				// TODO: call wildmeta post login endpoint
			}

			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			if storage
				.insert(
					&(identity.to_omni_account().clone(), AUTH_TOKEN_ACCESS_TYPE),
					access_token.clone(),
				)
				.is_err()
			{
				error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ACCESS_TYPE);
			};

			// TODO: include clients post login response in the response

			Ok::<UserLoginResponse, ErrorObject>(UserLoginResponse { access_token, id_token })
		})
		.expect("Failed to register omni_requestJwt method");
}

fn create_jwt_for_user(
	identity: Identity,
	token_type: &str,
	client_id: &str,
	jwt_rsa_private_key: &[u8],
) -> Result<String, ()> {
	let expires_at = Utc::now()
		.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
		.expect("Failed to calculate expiration")
		.timestamp();
	let auth_options = AuthOptions { expires_at };
	let omni_account = identity.to_omni_account();
	let token_claims = AuthTokenClaims::new(
		omni_account.to_hex(),
		token_type.to_string(),
		client_id.to_string(),
		auth_options.clone(),
	);
	jwt::create(&token_claims, jwt_rsa_private_key).map_err(|e| {
		error!("Failed to create JWT token: {:?}", e);
	})
}
