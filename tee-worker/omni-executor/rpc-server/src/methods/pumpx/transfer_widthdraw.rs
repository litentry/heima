use crate::{
	error_code::*, oneshot, server::RpcContext, verify_auth::verify_auth, Decode, Deserialize,
	ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};
use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct TransferWithdrawParams {
	pub user_email: String,
	pub chain_id: u32,
	pub wallet_index: PumxWalletIndex,
	pub recipient_address: String,
	pub token_ca: String,
	pub amount: String,
	pub email_code: String,
	pub google_code: MaybeGoogleCode,
	pub lang: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TransferWithdrawResponse {
	pub backend_response: Option<Vec<String>>,
}

impl From<TransferWithdrawParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: TransferWithdrawParams) -> Self {
		Self {
			task: NativeTask::PumpxTransferWidthdraw(
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email),
				p.chain_id,
				p.wallet_index,
				p.recipient_address,
				p.token_ca,
				p.amount,
				p.google_code,
				p.lang,
			),
			nonce: None,
			auth: Some(OmniAuth::Email(p.email_code)),
		}
	}
}

pub fn register_transfer_withdraw(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_transferWithdraw", |params, ctx, _| async move {
			let internal_error: ErrorObject = ErrorCode::InternalError.into();
			let params = params.parse::<TransferWithdrawParams>()?;

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(internal_error);
			}

			match response_receiver.await {
				Ok(response) => {
					let native_task_response: NativeTaskResponse =
						Decode::decode(&mut response.as_slice())
							.map_err(|_| internal_error.clone())?;
					match native_task_response {
						Ok(NativeTaskOk::PumpxTransferWithdraw(send_tx_response)) => {
							Ok(TransferWithdrawResponse {
								backend_response: send_tx_response.data.tx_hash,
							})
						},
						Err(NativeTaskError::InternalError) => {
							log::error!("Internal error in native task");
							Err(internal_error)
						},
						Err(native_task_error) => {
							log::error!("Native task error: {:?}", native_task_error);
							Err(ErrorCode::ServerError(get_native_task_error_code(
								&native_task_error,
							))
							.into())
						},
						_ => {
							log::error!("Unexpected response type");
							Err(internal_error)
						},
					}
				},
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(internal_error)
				},
			}
		})
		.expect("Failed to register pumpx_transferWithdraw method");
}
