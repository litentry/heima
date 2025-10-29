use executor_primitives::utils::hex::decode_hex;
use executor_primitives::AccountId;

pub fn to_omni_account(s: &str) -> Result<AccountId, ()> {
	decode_hex(s).map_err(|_| ()).and_then(|b| AccountId::try_from(b.as_slice()))
}
