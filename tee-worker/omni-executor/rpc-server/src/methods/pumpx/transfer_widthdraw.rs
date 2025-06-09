use crate::{
	error_code::*,
	methods::pumpx::{common::check_and_get_option_response_data, PumpxRpcError},
	server::RpcContext,
	verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::methods::create_transfer_tx::CreateTransferTxResponse;
use serde::Serialize;
use tracing::{debug, error};

use super::common::{check_pumpx_api_response, handle_pumpx_native_task};

#[derive(Debug, Deserialize)]
pub struct TransferWithdrawParams {
	pub client_id: String,
	pub user_id: String,
	pub user_email: String,
	pub request_id: Option<u32>,
	pub chain_id: u32,
	pub wallet_index: PumxWalletIndex,
	pub recipient_address: String,
	pub token_ca: String,
	pub amount: String,
	pub email_code: String,
	pub google_code: String,
	pub lang: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TransferWithdrawResponse {
	pub backend_response: CreateTransferTxResponse,
}

impl TransferWithdrawParams {
	pub fn into_native_task_wrapper(self) -> NativeTaskWrapper<NativeTask> {
		NativeTaskWrapper::new(
			NativeTask::PumpxTransferWidthdraw(
				Identity::from_web2_account(self.user_id.as_str(), Web2IdentityType::Pumpx),
				self.request_id,
				self.chain_id,
				self.wallet_index,
				self.recipient_address,
				self.token_ca,
				self.amount,
				self.google_code,
				self.lang,
			),
			None,
			Some(OmniAuth::Email(self.user_email, self.email_code)),
			self.client_id,
		)
	}
}

pub fn register_transfer_withdraw(module: &mut RpcModule<RpcContext>) {
	module        .register_async_method("pumpx_transferWithdraw", |params, ctx, _ext| async move {
			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received pumpx_transferWithdraw, user_id: {}, chain_id: {}, wallet_index: {}, recipient_address: {}, token_ca: {}, amount: {}",
		params.user_id, params.chain_id, params.wallet_index, params.recipient_address, params.token_ca, params.amount);

			// verify user_id and user_email matches
			debug!("Calling pumpx get_account_user_id, email: {}", params.user_email);
			let Ok(res) = ctx.pumpx_api.get_account_user_id(params.user_email.clone()).await else {
				error!("Failed to call get_account_user_id");
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE,
				)));
			};
			debug!("Response pumpx get_account_user_id: {:?}", res);

			let user_id = check_and_get_option_response_data(res.data.user_id, PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE, "Response data.user_id of call get_account_user_id is none")?;

			if user_id != params.user_id {
				error!(
					"Parameter mismatch: user_id {} and user_email {}, expected user_id {}",
					params.user_id,
					params.user_email,
					user_id
				);
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					USER_EMAIL_ID_MISMATCH_CODE,
				)));
			}

			let wrapper: NativeTaskWrapper<NativeTask> = params.into_native_task_wrapper();

            if wrapper.task.require_auth() {
				let Some(ref auth) = wrapper.auth else {
					error!("Missing auth token");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
						REQUIRE_AUTHENTICATION_CODE,
					)));
				};
		        verify_auth(ctx.clone(), auth).await.map_err(|_| {
		        	error!("Failed to verify auth: {:?}", wrapper.auth);
                    PumpxRpcError::from_error_code(ErrorCode::ServerError(
                        AUTH_VERIFICATION_FAILED_CODE,
                    ))
		        })?;
	        }

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxTransferWithdraw(response) => {
					check_pumpx_api_response(response.clone(), "Transfer withdraw".into())?;
					Ok(TransferWithdrawResponse { backend_response: response })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_transferWithdraw method");
}
