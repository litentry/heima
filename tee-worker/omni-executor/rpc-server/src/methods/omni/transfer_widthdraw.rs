use crate::{
	error_code::*, methods::omni::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::types::CreateTransferTxResponse;
use serde::Serialize;

use super::common::{check_omni_api_response, handle_omni_native_task};

#[derive(Debug, Deserialize)]
pub struct TransferWithdrawParams {
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
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email),
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
		.register_async_method("omni_transferWithdraw", |params, ctx, _| async move {
			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				log::error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			log::debug!("Received omni_transferWithdraw, user_email: {}, chain_id: {}, wallet_index: {}, recipient_address: {}, token_ca: {}, amount: {}",
		params.user_email, params.chain_id, params.wallet_index, params.recipient_address, params.token_ca, params.amount);


			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

          	if wrapper.task.require_auth() {
				let Some(ref auth) = wrapper.auth else {
					log::error!("Missing auth token");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
						REQUIRE_AUTHENTICATION_CODE,
					)));
				};
				verify_auth(ctx.clone(), auth).await.map_err(|_| {
					log::error!("Failed to verify auth: {:?}", wrapper.auth);
					PumpxRpcError::from_error_code(ErrorCode::ServerError(
						AUTH_VERIFICATION_FAILED_CODE,
					))
				})?;
			}

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxTransferWithdraw(response) => {
					check_omni_api_response(response.clone(), "Transfer withdraw".into())?;
					Ok(TransferWithdrawResponse { backend_response: response })
				},
				_ => {
					log::error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_transferWithdraw method");
}
