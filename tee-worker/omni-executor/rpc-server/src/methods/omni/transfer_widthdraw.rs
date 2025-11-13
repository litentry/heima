use crate::{
	detailed_error::DetailedError,
	methods::omni::check_backend_response,
	server::RpcContext,
	utils::omni::extract_omni_account,
	utils::pumpx::verify_google_code,
	utils::validation::{
		parse_rpc_params, validate_amount, validate_chain_id, validate_evm_address,
		validate_wallet_index,
	},
	Deserialize,
};
use jsonrpsee::RpcModule;
use oe_client_pumpx::methods::create_transfer_tx::{
	CreateTransferTxBody, CreateTransferTxResponse,
};
use oe_core::auth::constants::AUTH_TOKEN_ACCESS_TYPE;
use oe_core::intent::executor::IntentExecutor;
use oe_core::native_task::PumxWalletIndex;
use oe_storage::{HeimaJwtStorage, Storage};
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
			debug!("Received omni_transferWithdraw, params: {:?}", params);

			let params = parse_rpc_params::<TransferWithdrawParams>(params)?;
			let omni_account = extract_omni_account(&ext)?;

			validate_chain_id(params.chain_id)?;
			validate_wallet_index(params.wallet_index)?;
			validate_evm_address(&params.recipient_address, "recipient_address")?;
			validate_evm_address(&params.token_ca, "token_ca")?;
			validate_amount(&params.amount, "amount")?;

			// Inline handle_pumpx_transfer_withdraw logic
			// 1. Verify we have a valid Pumpx "access" token for the user
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get access_token within TransferWidthdraw");
				return Err(DetailedError::storage_service_error("get access token").to_rpc_error());
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
				let msg = "Failed to verify google code within TransferWidthdraw";
				error!(msg);
				return Err(DetailedError::internal_error(msg).to_rpc_error());
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
			let response = ctx
				.pumpx_api
				.create_transfer_tx(&access_token, body, params.lang.clone())
				.await
				.map_err(|e| {
					error!("Failed to create transfer tx: {:?}", e);
					DetailedError::pumpx_service_error("create_transfer_tx", format!("{:?}", e))
						.to_rpc_error()
				})?;

			check_backend_response(&response, "transfer_withdraw")?;
			Ok(TransferWithdrawResponse { backend_response: response })
		})
		.expect("Failed to register omni_transferWithdraw method");
}
