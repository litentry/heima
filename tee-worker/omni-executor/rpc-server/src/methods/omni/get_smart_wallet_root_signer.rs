use crate::{
	error_code::*,
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
	ErrorCode,
};
use executor_primitives::utils::hex::{hex_encode, FromHexPrefixed};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use signer_client::ChainType;
use tracing::{debug, error};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetSmartWalletRootSignerParams {
	pub chain_type: ChainType,
	pub index: u32,
}

pub fn register_get_smart_wallet_root_signer(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getSmartWalletRootSigner", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;

			let params = params.parse::<GetSmartWalletRootSignerParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_getSmartWalletRootSigner");

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			let wallet = ctx
				.signer_client
				.request_wallet(params.chain_type, params.index, address.as_ref().to_owned())
				.await
				.map_err(|_| {
					error!("Failed to request wallet from signer client");
					PumpxRpcError::from_error_code(ErrorCode::ServerError(
						PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE,
					))
				})?;

			Ok::<String, _>(hex_encode(&wallet))
		})
		.expect("Failed to register omni_addWallet method");
}
