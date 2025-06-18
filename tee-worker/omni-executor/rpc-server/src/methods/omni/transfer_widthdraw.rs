use crate::{
	error_code::*,
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::methods::create_transfer_tx::CreateTransferTxResponse;
use serde::Serialize;
use tracing::{debug, error};

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
	pub google_code: String,
	pub lang: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TransferWithdrawResponse {
	pub backend_response: CreateTransferTxResponse,
}

impl TransferWithdrawParams {
	pub fn into_native_task_wrapper(self, client_id: String) -> NativeTaskWrapper<NativeTask> {
		NativeTaskWrapper::new(
			NativeTask::PumpxTransferWidthdraw(
				Identity::from_web2_account(self.user_email.as_str(), Web2IdentityType::Email),
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
			None,
			client_id,
		)
	}
}

pub fn register_transfer_withdraw(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_transferWithdraw", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;

			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_transferWithdraw, user_email: {}, chain_id: {}, wallet_index: {}, recipient_address: {}, token_ca: {}, amount: {}",
		params.user_email, params.chain_id, params.wallet_index, params.recipient_address, params.token_ca, params.amount);

			let wrapper = params.into_native_task_wrapper(user.client_id);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxTransferWithdraw(response) => {
					check_omni_api_response(response.clone(), "Transfer withdraw".into())?;
					Ok(TransferWithdrawResponse { backend_response: response })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_transferWithdraw method");
}
