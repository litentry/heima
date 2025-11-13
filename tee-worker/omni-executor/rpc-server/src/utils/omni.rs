use crate::error_code::INVALID_RPC_EXTENSION;
use crate::middlewares::RpcExtensions;
use jsonrpsee::core::RpcResult;
use jsonrpsee::Extensions;
use oe_primitives::utils::hex::decode_hex;
use oe_primitives::AccountId;

use crate::detailed_error::DetailedError;

pub fn to_omni_account(s: &str) -> RpcResult<AccountId> {
	decode_hex(s)
		.map_err(|_| ())
		.and_then(|b| AccountId::try_from(b.as_slice()))
		.map_err(|_| DetailedError::parse_error("Failed to parse omni_account").to_rpc_error())
}

/// This is used to extract the sender from the extension (see rpc_middleware.rs),
/// and then convert it to a valid omni_account
pub fn extract_omni_account(ext: &Extensions) -> RpcResult<AccountId> {
	ext.get::<RpcExtensions>()
		.map(|e| e.sender.clone())
		.ok_or(DetailedError::new(INVALID_RPC_EXTENSION, "Invalid rpc extension").to_rpc_error())
		.and_then(|s| to_omni_account(&s))
}
