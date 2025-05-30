use crate::{
	error_code::*, methods::omni::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode, Serialize,
};
use executor_primitives::{OmniAuth, UserAuth, UserId};
use heima_primitives::Identity;
use jsonrpsee::RpcModule;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct UserLoginParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
}

#[derive(Serialize, Clone)]
pub struct UserLoginResponse {
	pub access_token: String,
	pub id_token: String,
}

impl TryFrom<UserLoginParams> for OmniAuth {
	type Error = PumpxRpcError;

	fn try_from(p: UserLoginParams) -> Result<Self, Self::Error> {
		let omni_auth = match p.user_auth {
			UserAuth::Email(code) => {
				let UserId::Email(email) = p.user_id else {
					error!("User ID must be an email for Email authentication");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ParseError));
				};
				OmniAuth::Email(email, code)
			},
			UserAuth::Web3(signature) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					PumpxRpcError::from_error_code(ErrorCode::ParseError)
				})?;
				if !identity.is_web3() {
					error!("User ID must be a Web3 identity for Web3 authentication");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ParseError));
				}
				OmniAuth::Web3(identity, signature)
			},
			UserAuth::AuthToken(token) => OmniAuth::AuthToken(token),
			UserAuth::OAuth2(data) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					PumpxRpcError::from_error_code(ErrorCode::ParseError)
				})?;
				OmniAuth::OAuth2(identity, data)
			},
			UserAuth::Pumpx { email_code, .. } => {
				let UserId::Pumpx(handle) = p.user_id else {
					error!("User ID must be a Pumpx handle for Pumpx authentication");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ParseError));
				};
				OmniAuth::Email(handle, email_code)
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
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;
			let auth = OmniAuth::try_from(params).map_err(|e| {
				error!("Failed to convert params to OmniAuth: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;
			verify_auth(ctx.clone(), &auth).await.map_err(|_| {
				error!("Failed to verify auth: {:?}", auth);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;

			// TOOD:
			// - Generate access token and ID token
			// - Call client specific api ?

			Ok::<UserLoginResponse, PumpxRpcError>(UserLoginResponse {
				access_token: "mock_access".to_string(),
				id_token: "mock_id_token".to_string(),
			})
		})
		.expect("Failed to register omni_requestJwt method");
}
