use std::sync::Arc;
use super::common::{check_pumpx_api_response, handle_pumpx_native_task};
use crate::{
	error_code::*,
	methods::pumpx::{common::check_and_get_option_response_data, PumpxRpcError},
	server::RpcContext,
	verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use pumpx::methods::create_transfer_tx::CreateTransferTxResponse;
use serde::Serialize;
use tracing::{debug, error};

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
		let identity = Identity::from_web2_account(self.user_id.as_str(), Web2IdentityType::Pumpx);
		let omni_account = identity.to_omni_account(&self.client_id);

		NativeTaskWrapper::new(
			NativeTask::PumpxTransferWidthdraw(
				omni_account,
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
			Some(OmniAuth::Email(self.client_id.clone(), self.user_email, self.email_code)),
			self.client_id,
		)
	}
}

#[tracing::instrument(skip(ctx, params), fields(
	client_id = %params.client_id,
	user_id = %params.user_id,
	user_email = %params.user_email,
	request_id = %params.request_id.unwrap_or(0),
	chain_id = %params.chain_id,
	wallet_index = %params.wallet_index,
	recipient_address = %params.recipient_address,
	token_ca = %params.token_ca,
	amount = %params.amount,
	lang = %params.lang.as_deref().unwrap_or("None")
))]
async fn handle_transfer_withdraw_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: TransferWithdrawParams,
	ctx: Arc<RpcContext<
		Header,
		RpcClient,
		RpcClientFactory,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		CrossChainIntentExecutor,
	>>,
) -> Result<TransferWithdrawResponse, PumpxRpcError> {
	debug!("Processing pumpx_transferWithdraw request");

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
}

pub fn register_transfer_withdraw<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
	module        .register_async_method("pumpx_transferWithdraw", |params, ctx, _ext| async move {
			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			handle_transfer_withdraw_request(params, ctx).await
		})
		.expect("Failed to register pumpx_transferWithdraw method");
}
