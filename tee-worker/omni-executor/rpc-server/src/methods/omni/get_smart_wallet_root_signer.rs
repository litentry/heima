use crate::{
	detailed_error::DetailedError, server::RpcContext, utils::omni::to_omni_account,
	utils::types::RpcResultExt, utils::validation::parse_rpc_params,
};
use jsonrpsee::{types::ErrorObjectOwned, RpcModule};
use oe_client_pumpx::pubkey_to_address;
use oe_client_signer::ChainType;
use oe_core::intent::executor::IntentExecutor;
use serde::Deserialize;
use tracing::debug;

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

			let address = ctx
				.signer_client
				.request_wallet(params.chain_type.into(), params.wallet_index, omni_account.into())
				.await
				.map_err(|_| DetailedError::signer_service_error().to_rpc_error())
				.and_then(|pk| {
					pubkey_to_address(params.chain_type.into(), &pk)
						.map_err_internal("Failed to convert pubkey to address")
				})?;

			Ok::<String, ErrorObjectOwned>(address)
		})
		.expect("Failed to register omni_addWallet method");
}
