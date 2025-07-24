use crate::{error_code::*, methods::omni::PumpxRpcError, server::RpcContext, ErrorCode};
use executor_primitives::utils::hex::FromHexPrefixed;
use heima_primitives::Address32;
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

pub fn register_get_smart_wallet_root_signer(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getSmartWalletRootSigner", |params, ctx, _| async move {
			let params = params.parse::<GetSmartWalletRootSignerParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_getSmartWalletRootSigner, params: {:?}", params);

			let Ok(address) = Address32::from_hex(&params.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
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
					PumpxRpcError::from_error_code(ErrorCode::ServerError(
						PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE,
					))
				})?;

			let address = pubkey_to_address(params.chain_type.into(), &pubkey).map_err(|_| {
				error!("Failed to convert pubkey to address");
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					PUMPX_SIGNER_PUBKEY_TO_ADDRESS_FAILED_CODE,
				))
			})?;

			Ok::<String, _>(address)
		})
		.expect("Failed to register omni_addWallet method");
}
