use executor_primitives::utils::hex::decode_hex;
use executor_primitives::AccountId;
use jsonrpsee::types::ErrorObjectOwned;

use crate::detailed_error::DetailedError;

pub fn to_omni_account(s: &str) -> Result<AccountId, ErrorObjectOwned> {
	decode_hex(s)
		.map_err(|_| ())
		.and_then(|b| AccountId::try_from(b.as_slice()))
		.map_err(|_| DetailedError::parse_error("Failed to parse omni_account").to_rpc_error())
}
