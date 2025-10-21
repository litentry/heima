use super::common::check_omni_api_response;
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
use executor_core::native_task::PumxWalletIndex;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::{handle_pumpx_transfer_withdraw, NativeTaskError, NativeTaskOk};
use pumpx::methods::create_transfer_tx::CreateTransferTxResponse;
use serde::Serialize;
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

pub fn register_transfer_withdraw<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_transferWithdraw", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from(DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication failed"
				)
				.with_suggestion("Please provide valid authentication credentials"))
			})?;

			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(DetailedError::new(
					PARSE_ERROR_CODE,
					"Failed to parse request parameters"
				)
				.with_reason(format!("Invalid JSON structure: {}", e)))
			})?;

			debug!("Received omni_transferWithdraw, chain_id: {}, wallet_index: {}, recipient_address: {}, token_ca: {}, amount: {}",
		params.chain_id, params.wallet_index, params.recipient_address, params.token_ca, params.amount);

			validate_chain_id(params.chain_id, Some("evm")).map_err(PumpxRpcError::from)?;

			validate_wallet_index(params.wallet_index).map_err(PumpxRpcError::from)?;

			validate_ethereum_address(&params.recipient_address, "recipient_address")
				.map_err(PumpxRpcError::from)?;

			validate_token_address(&params.token_ca, "token_ca")
				.map_err(PumpxRpcError::from)?;

			validate_amount(&params.amount, "amount")
				.map_err(PumpxRpcError::from)?;

			let address_bytes = validate_omni_account_hex(&user.omni_account, "omni_account")
				.map_err(PumpxRpcError::from)?;

			validate_omni_account_length(&address_bytes, "omni_account")
				.map_err(PumpxRpcError::from)?;

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account after validation");
				return Err(DetailedError::account_parse_error(
					&user.omni_account,
					"Address32 conversion failed"
				).into());
			};
			let omni_account = AccountId::from(address);

			let result = handle_pumpx_transfer_withdraw(
				ctx.to_task_handler_context(),
				omni_account,
				params.request_id,
				params.chain_id,
				params.wallet_index,
				params.recipient_address,
				params.token_ca,
				params.amount,
				params.google_code,
				params.lang,
				user.client_id,
			)
			.await;

			match result {
				Ok(NativeTaskOk::PumpxTransferWithdraw(response)) => {
					check_omni_api_response(response.clone(), "Transfer withdraw".into())?;
					Ok(TransferWithdrawResponse { backend_response: response })
				},
				Ok(_) => {
					error!("Unexpected response type from native task handler");
					Err(DetailedError::unexpected_response_type(
						"PumpxTransferWithdraw",
						"Unknown"
					).into())
				},
				Err(NativeTaskError::PumpxApiError(api_error)) => {
					error!("Pumpx API error: {:?}", api_error);
					match api_error {
						native_task_handler::PumpxApiError::GoogleCodeVerificationFailed => {
							Err(PumpxRpcError::from(DetailedError::new(
								PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE,
								"Google code verification failed"
							).with_suggestion("Please check your Google verification code and try again")))
						},
						native_task_handler::PumpxApiError::CreateTransferTxFailed => {
							Err(PumpxRpcError::from(DetailedError::new(
								INTERNAL_ERROR_CODE,
								"Failed to create transfer transaction"
							).with_suggestion("Please check your transfer parameters and try again")))
						},
						_ => {
							Err(PumpxRpcError::from(DetailedError::new(
								INTERNAL_ERROR_CODE,
								"Pumpx API error"
							).with_reason(format!("{:?}", api_error))))
						}
					}
				},
				Err(NativeTaskError::ChainNotSupported(chain_id)) => {
					error!("Chain not supported: {}", chain_id);
					Err(PumpxRpcError::from(DetailedError::new(
						INVALID_CHAIN_ID_CODE,
						"Chain not supported"
					).with_reason(format!("Chain ID {} is not supported", chain_id))))
				},
				Err(NativeTaskError::InternalError(msg)) => {
					error!("Internal error: {:?}", msg);
					Err(PumpxRpcError::from(DetailedError::new(
						INTERNAL_ERROR_CODE,
						"Internal error"
					).with_reason(msg.unwrap_or_else(|| "Unknown internal error".to_string()))))
				},
				Err(e) => {
					error!("Failed to create transfer withdraw: {:?}", e);
					Err(PumpxRpcError::from(DetailedError::new(
						INTERNAL_ERROR_CODE,
						"Failed to create transfer withdraw"
					).with_reason(format!("{:?}", e))))
				},
			}
		})
		.expect("Failed to register omni_transferWithdraw method");
}
