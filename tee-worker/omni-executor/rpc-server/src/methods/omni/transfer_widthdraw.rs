use crate::{
	detailed_error::DetailedError,
	error_code::*,
	methods::omni::{check_auth, check_omni_api_response},
	server::RpcContext,
	utils::omni::to_omni_account,
	utils::pumpx::verify_google_code,
	utils::validation::{
		validate_amount, validate_chain_id, validate_ethereum_address, validate_token_address,
		validate_wallet_index,
	},
	Deserialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::PumxWalletIndex;
use executor_storage::{HeimaJwtStorage, Storage};
use heima_authentication::constants::AUTH_TOKEN_ACCESS_TYPE;
use jsonrpsee::RpcModule;
use pumpx::methods::create_transfer_tx::{CreateTransferTxBody, CreateTransferTxResponse};
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
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_transferWithdraw", |params, ctx, ext| async move {
			let oa_str = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication failed"
				)
				.with_suggestion("Please provide valid authentication credentials").to_rpc_error()
			})?;

			let params = params.parse::<TransferWithdrawParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(
					PARSE_ERROR_CODE,
					"Failed to parse request parameters"
				)
				.with_reason(format!("Invalid JSON structure: {}", e)).to_rpc_error()
			})?;

			debug!("Received omni_transferWithdraw, chain_id: {}, wallet_index: {}, recipient_address: {}, token_ca: {}, amount: {}",
		params.chain_id, params.wallet_index, params.recipient_address, params.token_ca, params.amount);

			validate_chain_id(params.chain_id, Some("evm")).map_err(|e| e.to_rpc_error())?;

			validate_wallet_index(params.wallet_index).map_err(|e| e.to_rpc_error())?;

			validate_ethereum_address(&params.recipient_address, "recipient_address")
				.map_err(|e| e.to_rpc_error())?;

			validate_token_address(&params.token_ca, "token_ca")
				.map_err(|e| e.to_rpc_error())?;

			validate_amount(&params.amount, "amount")
				.map_err(|e| e.to_rpc_error())?;

			let omni_account = to_omni_account(&oa_str).map_err(|_| {
				DetailedError::new(
					PARSE_ERROR_CODE,
					"Failed to parse omni account",
				).to_rpc_error()
			})?;

			// Inline handle_pumpx_transfer_withdraw logic
			// 1. Verify we have a valid Pumpx "access" token for the user
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) = storage.get(&(omni_account, AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get access_token within TransferWidthdraw");
				return Err(DetailedError::new(
					INTERNAL_ERROR_CODE,
					"Internal error"
				).with_reason("Failed to get access token").to_rpc_error());
			};

			// 2. Verify google code
			let verify_success = verify_google_code(
				ctx.pumpx_api.as_ref().as_ref(),
				&access_token,
				params.google_code,
				params.lang.clone(),
			)
			.await;

			if !verify_success {
				error!("Failed to verify google code within TransferWidthdraw");
				return Err(DetailedError::new(
					PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE,
					"Google code verification failed"
				).with_suggestion("Please check your Google verification code and try again").to_rpc_error());
			}

			// 3. Create a transfer tx and send to backend
			let body = CreateTransferTxBody {
				request_id: params.request_id,
				chain_id: params.chain_id,
				wallet_index: params.wallet_index,
				recipient_address: params.recipient_address,
				token_ca: params.token_ca,
				amount: params.amount,
			};

			debug!("Calling pumpx create_transfer_tx, body {:?}", body);
			let response = ctx.pumpx_api.create_transfer_tx(&access_token, body, params.lang.clone()).await
				.map_err(|e| {
					error!("Failed to create transfer tx: {}", e);
					DetailedError::new(
						PUMPX_API_CREATE_TRANSFER_TX_FAILED_CODE,
						"Failed to create transfer transaction"
					).with_suggestion("Please check your transfer parameters and try again").to_rpc_error()
				})?;

			check_omni_api_response(response.clone(), "Transfer withdraw".into())?;
			Ok(TransferWithdrawResponse { backend_response: response })
		})
		.expect("Failed to register omni_transferWithdraw method");
}
