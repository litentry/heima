use super::common::{check_omni_api_response, handle_omni_native_task};
use crate::{
	detailed_error::DetailedError,
	error_code::*,
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
	validation_helpers::{
		validate_amount, validate_chain_id, validate_ethereum_address, validate_omni_account_hex,
		validate_omni_account_length, validate_token_address, validate_wallet_index,
	},
	Deserialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use pumpx::methods::create_transfer_tx::CreateTransferTxResponse;
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct TransferWithdrawParams {
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
	pub fn into_native_task_wrapper(
		self,
		client_id: String,
		omni_account: AccountId,
	) -> NativeTaskWrapper<NativeTask> {
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
			None,
			client_id,
		)
	}
}

#[tracing::instrument(skip(params, user, ctx), fields(
	client_id = %user.client_id,
	omni_account = %user.omni_account,
	request_id = ?params.request_id,
	chain_id = %params.chain_id,
	wallet_index = %params.wallet_index,
	recipient_address = %params.recipient_address,
	token_ca = %params.token_ca,
	amount = %params.amount,
	lang = ?params.lang
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
	user: crate::methods::omni::common::User,
	ctx: Arc<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) -> Result<TransferWithdrawResponse, PumpxRpcError> {
	debug!("Processing omni_transferWithdraw request");

	validate_chain_id(params.chain_id, Some("evm")).map_err(PumpxRpcError::from)?;

	validate_wallet_index(params.wallet_index).map_err(PumpxRpcError::from)?;

	validate_ethereum_address(&params.recipient_address, "recipient_address")
		.map_err(PumpxRpcError::from)?;

	validate_token_address(&params.token_ca, "token_ca").map_err(PumpxRpcError::from)?;

	validate_amount(&params.amount, "amount").map_err(PumpxRpcError::from)?;

	let address_bytes = validate_omni_account_hex(&user.omni_account, "omni_account")
		.map_err(PumpxRpcError::from)?;

	validate_omni_account_length(&address_bytes, "omni_account").map_err(PumpxRpcError::from)?;

	let Ok(address) = Address32::from_hex(&user.omni_account) else {
		error!("Failed to parse from omni account after validation");
		return Err(DetailedError::account_parse_error(
			&user.omni_account,
			"Address32 conversion failed",
		)
		.into());
	};
	let omni_account = AccountId::from(address);

	let wrapper = params.into_native_task_wrapper(user.client_id, omni_account);

	handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
		NativeTaskOk::PumpxTransferWithdraw(response) => {
			check_omni_api_response(response.clone(), "Transfer withdraw".into())?;
			Ok(TransferWithdrawResponse { backend_response: response })
		},
		_ => {
			error!("Unexpected response type from native task handler");
			Err(DetailedError::unexpected_response_type("PumpxTransferWithdraw", "Unknown").into())
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
	module
		.register_async_method("omni_transferWithdraw", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication failed")
						.with_suggestion("Please provide valid authentication credentials"),
				)
			})?;

			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Failed to parse request parameters")
						.with_reason(format!("Invalid JSON structure: {}", e)),
				)
			})?;

			handle_transfer_withdraw_request(params, user, ctx).await
		})
		.expect("Failed to register omni_transferWithdraw method");
}
