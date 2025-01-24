mod types;

use crypto::jwt;
use executor_core::native_call::NativeCall;
use heima_authentication::auth_token::AuthTokenClaims;
use parentchain_rpc_client::{CustomConfig, SubxtClient};
use parity_scale_codec::Encode;
use primitives::{utils::hex::ToHexPrefixed, OmniAccountAuthType};
use std::sync::Arc;
use storage::{MemberOmniAccountStorage, Storage, StorageDB};
use tokio::sync::{mpsc, oneshot};
use types::{NativeCallError, NativeCallOk};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeTaskSender = mpsc::Sender<NativeTask>;
type NativeCallResponse = Result<NativeCallOk, NativeCallError>;

pub struct NativeTask {
	pub call: NativeCall,
	pub auth_type: OmniAccountAuthType,
	pub response_sender: ResponseSender,
}
pub struct TaskHandlerContext {
	pub parentchain_rpc_client: Arc<SubxtClient<CustomConfig>>,
	pub storage_db: Arc<StorageDB>,
	pub jwt_secret: String,
}

pub async fn run_native_task_handler(
	buffer: usize,
	ctx: Arc<TaskHandlerContext>,
) -> NativeTaskSender {
	let (sender, mut receiver) = mpsc::channel::<NativeTask>(buffer);

	tokio::spawn(async move {
		while let Some(task) = receiver.recv().await {
			handle_native_call(ctx.clone(), task.call, task.response_sender).await;
		}
	});

	sender
}

async fn handle_native_call(
	ctx: Arc<TaskHandlerContext>,
	call: NativeCall,
	response_sender: ResponseSender,
) {
	match call {
		NativeCall::request_auth_token(sender_identity, auth_options) => {
			let member_omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = member_omni_account_storage.get(&sender_identity.hash())
			else {
				let response = NativeCallResponse::Err(NativeCallError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let claims = AuthTokenClaims::new(omni_account.to_hex(), auth_options);
			let Ok(token) = jwt::create(&claims, ctx.jwt_secret.as_bytes()) else {
				let response = NativeCallResponse::Err(NativeCallError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			// let auth_token_requested_call = parentchain_api_interface::tx()
			// 	.omni_account()
			// 	.auth_token_requested(omni_account.into(), claims.exp);
			// TODO: send transaction to parentchain
			let response = NativeCallResponse::Ok(NativeCallOk::AuthToken(token));
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
		_ => {
			let response = NativeCallResponse::Err(NativeCallError::UnexpectedCall(format!(
				"Unexpected call: {:?}",
				call
			)));
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
	}
}
