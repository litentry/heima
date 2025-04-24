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
use pumpx::types::CreateTransferTxResponse;
use serde::Serialize;

use super::common::{check_pumpx_api_response, handle_pumpx_native_task};

#[derive(Debug, Deserialize)]
pub struct TransferWithdrawParams {
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

impl From<TransferWithdrawParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: TransferWithdrawParams) -> Self {
		Self {
			task: NativeTask::PumpxTransferWidthdraw(
				Identity::from_web2_account(p.user_id.as_str(), Web2IdentityType::Pumpx),
				p.request_id,
				p.chain_id,
				p.wallet_index,
				p.recipient_address,
				p.token_ca,
				p.amount,
				p.google_code,
				p.lang,
			),
			nonce: None,
			auth: Some(OmniAuth::Email(p.user_email, p.email_code)),
		}
	}
}

pub fn register_transfer_withdraw(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_transferWithdraw", |params, ctx, _| async move {
			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				log::error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			log::debug!("Received pumpx_transferWithdraw, user_id: {}, chain_id: {}, wallet_index: {}, recipient_address: {}, token_ca: {}, amount: {}", 
		params.user_id, params.chain_id, params.wallet_index, params.recipient_address, params.token_ca, params.amount);

			// verify user_id and user_email matches
			log::debug!("Calling pumpx get_account_user_id, email: {}", params.user_email);
			let Ok(res) = ctx.pumpx_api.get_account_user_id(params.user_email.clone()).await else {
				log::error!("Failed to call get_account_user_id");
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE,
				)));
			};
			log::debug!("Response pumpx get_account_user_id: {:?}", res);

			let res_data = check_and_get_option_response_data(res.data, PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE, "Response data of call get_account_user_id is none")?;
			let user_id = check_and_get_option_response_data(res_data.user_id, PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE, "Response data.user_id of call get_account_user_id is none")?;

			if user_id != params.user_id {
				log::error!(
					"Parameter mismatch: user_id {} and user_email {}, expected user_id {}",
					params.user_id,
					params.user_email,
					user_id
				);
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					USER_EMAIL_ID_MISMATCH_CODE,
				)));
			}

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				log::error!("Failed to verify auth token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			}

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxTransferWithdraw(response) => {
					check_pumpx_api_response(response.clone(), "Transfer withdraw".into())?;
					Ok(TransferWithdrawResponse { backend_response: response })
				},
				_ => {
					log::error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_transferWithdraw method");
}
