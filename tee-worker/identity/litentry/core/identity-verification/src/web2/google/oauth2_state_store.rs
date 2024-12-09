use crate::alloc::{fmt, format, string::String};
use codec::Encode;
use core::result::Result;
use lazy_static::lazy_static;
use litentry_primitives::{
	ErrorDetail, ErrorString, IntoErrorDetail, ParentchainAccountId as AccountId,
};
use lru::LruCache;
use sp_core::H256;
use std::num::NonZeroUsize;
#[cfg(feature = "std")]
use std::sync::RwLock;
#[cfg(feature = "sgx")]
use std::sync::SgxRwLock as RwLock;

#[derive(Debug)]
pub enum OAuthStateStoreError {
	LockPoisoning,
	Other(String),
}

impl fmt::Display for OAuthStateStoreError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			OAuthStateStoreError::LockPoisoning => write!(f, "Lock poisoning"),
			OAuthStateStoreError::Other(msg) => write!(f, "{}", msg),
		}
	}
}

impl std::error::Error for OAuthStateStoreError {}

impl IntoErrorDetail for OAuthStateStoreError {
	fn into_error_detail(self) -> ErrorDetail {
		ErrorDetail::StfError(ErrorString::truncate_from(format!("{}", self).into()))
	}
}

lazy_static! {
	static ref STORE: RwLock<LruCache<String, String>> =
		RwLock::new(LruCache::new(NonZeroUsize::new(500).unwrap()));
}

pub struct OAuthStateStore;

type StateVerifier = String;

impl OAuthStateStore {
	pub fn insert(
		account_id: AccountId,
		identity_hash: H256,
		state_verifier: String,
	) -> Result<(), OAuthStateStoreError> {
		STORE
			.write()
			.map_err(|_| OAuthStateStoreError::LockPoisoning)?
			.put(hex::encode((account_id, identity_hash).encode()), state_verifier);
		Ok(())
	}

	pub fn get(
		account_id: &AccountId,
		identity_hash: H256,
	) -> Result<Option<StateVerifier>, OAuthStateStoreError> {
		let code = STORE
			.write()
			.map_err(|_| OAuthStateStoreError::LockPoisoning)?
			.pop(hex::encode((account_id, identity_hash).encode()).as_str());
		Ok(code)
	}
}
