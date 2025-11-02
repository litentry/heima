use crate::{
	detailed_error::DetailedError,
	error_code::{PARSE_ERROR_CODE, *},
	server::RpcContext,
	utils::omni::to_omni_account,
};
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::RpcModule;
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
			let params = params.parse::<GetSmartWalletRootSignerParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
					.into()
			})?;

			debug!("Received omni_getSmartWalletRootSigner, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account).map_err(|_| {
				DetailedError::new(PARSE_ERROR_CODE, "Failed to parse omni account").into()
			})?;

			let pubkey = ctx
				.signer_client
				.request_wallet(params.chain_type.into(), params.wallet_index, omni_account.into())
				.await
				.map_err(|_| {
					error!("Failed to request wallet from signer client");
					DetailedError::new(
						PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE,
						"Failed to request wallet from signer service",
					)
					.with_reason("Signer service is temporarily unavailable")
					.with_suggestion("Please try again later")
					.into()
				})?;

			let address = pubkey_to_address(params.chain_type.into(), &pubkey).map_err(|_| {
				error!("Failed to convert pubkey to address");
				DetailedError::new(
					PUMPX_SIGNER_PUBKEY_TO_ADDRESS_FAILED_CODE,
					"Failed to convert public key to address",
				)
				.with_reason("Public key conversion error")
				.with_suggestion("Please check your chain type and try again")
				.into()
			})?;

			Ok::<String>(address)
		})
		.expect("Failed to register omni_addWallet method");
}
