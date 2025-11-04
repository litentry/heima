use crate::{
	detailed_error::DetailedError, server::RpcContext, utils::omni::to_omni_account,
	utils::validation::parse_rpc_params,
};
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::{types::ErrorObjectOwned, RpcModule};
use pumpx::pubkey_to_address;
use serde::Deserialize;
use signer_client::ChainType;
use tracing::{debug, error};

// used in rpc with backend only
#[derive(Debug, Copy, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SerdeChainType {
	Evm,
	Solana,
	Tron,
}

impl From<SerdeChainType> for ChainType {
	fn from(c: SerdeChainType) -> Self {
		match c {
			SerdeChainType::Evm => Self::Evm,
			SerdeChainType::Solana => Self::Solana,
			SerdeChainType::Tron => Self::Tron,
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct GetSmartWalletRootSignerParams {
	pub omni_account: String,
	pub chain_type: SerdeChainType,
	pub wallet_index: u32,
}

pub fn register_get_smart_wallet_root_signer<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_getSmartWalletRootSigner", |params, ctx, _| async move {
			let params = parse_rpc_params::<GetSmartWalletRootSignerParams>(params)?;

			debug!("Received omni_getSmartWalletRootSigner, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account)?;

			let pubkey = ctx
				.signer_client
				.request_wallet(params.chain_type.into(), params.wallet_index, omni_account.into())
				.await
				.map_err(|_| DetailedError::signer_service_error().to_rpc_error())?;

			let address = pubkey_to_address(params.chain_type.into(), &pubkey).map_err(|_| {
				let msg = "Failed to convert pubkey to address";
				error!(msg);
				DetailedError::internal_error(msg).to_rpc_error()
			})?;

			Ok::<String, ErrorObjectOwned>(address)
		})
		.expect("Failed to register omni_addWallet method");
}
