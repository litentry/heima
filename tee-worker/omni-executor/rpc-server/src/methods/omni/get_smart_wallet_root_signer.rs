use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE, *},
	methods::omni::PumpxRpcError,
	server::RpcContext,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::utils::hex::FromHexPrefixed;
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
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
		.register_async_method("omni_getSmartWalletRootSigner", |params, ctx, _| async move {
			let params = params.parse::<GetSmartWalletRootSignerParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_getSmartWalletRootSigner, params: {:?}", params);

			let Ok(address) = Address32::from_hex(&params.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to parse omni account from authentication token"),
				));
			};

			let pubkey = ctx
				.signer_client
				.request_wallet(
					params.chain_type.into(),
					params.wallet_index,
					address.as_ref().to_owned(),
				)
				.await
				.map_err(|_| {
					error!("Failed to request wallet from signer client");
					PumpxRpcError::from(
						DetailedError::new(
							PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE,
							"Failed to request wallet from signer service",
						)
						.with_reason("Signer service is temporarily unavailable")
						.with_suggestion("Please try again later"),
					)
				})?;

			let address = pubkey_to_address(params.chain_type.into(), &pubkey).map_err(|_| {
				error!("Failed to convert pubkey to address");
				PumpxRpcError::from(
					DetailedError::new(
						PUMPX_SIGNER_PUBKEY_TO_ADDRESS_FAILED_CODE,
						"Failed to convert public key to address",
					)
					.with_reason("Public key conversion error")
					.with_suggestion("Please check your chain type and try again"),
				)
			})?;

			Ok::<String, _>(address)
		})
		.expect("Failed to register omni_addWallet method");
}
