pub mod email;
pub mod google;

use executor_primitives::{Identity, Web2IdentityType, Web2ValidationData};
use executor_storage::{Storage, StorageDB, VerificationCodeStorage};
use std::sync::Arc;

pub fn verify_identity(
	identity: &Identity,
	_raw_msg: &[u8],
	validation_data: &Web2ValidationData,
	storage_db: Arc<StorageDB>,
) -> Result<(), ()> {
	let username = match validation_data {
		Web2ValidationData::Twitter(_) => {
			log::warn!("Twitter validation data is not implemented yet");
			Err(())
		},
		Web2ValidationData::Discord(_) => {
			log::warn!("Discord validation data is not implemented yet");
			Err(())
		},
		Web2ValidationData::Email(email_validation_data) => {
			let email = vec_to_string(email_validation_data.email.to_vec())?;
			let verification_code =
				vec_to_string(email_validation_data.verification_code.to_vec())?;
			let email_identity = Identity::from_web2_account(&email, Web2IdentityType::Email);
			let verification_code_storage = VerificationCodeStorage::new(storage_db);
			let Some(code) = verification_code_storage.get(&email_identity.hash()) else {
				return Err(());
			};
			if verification_code.ne(&code) {
				return Err(());
			}
			Ok(email)
		},
	}?;

	match identity {
		Identity::Twitter(_handle) => {
			// - twitter's username is case insensitive
			return Err(());
		},
		Identity::Discord(_handle) => {
			// - discord's username is case sensitive
			return Err(());
		},
		Identity::Email(email) => {
			let email = std::str::from_utf8(email.inner_ref()).map_err(|_| ())?;
			if username.ne(email) {
				return Err(());
			}
		},
		_ => {
			log::error!("Invalid identity type {:?}", identity);
			return Err(());
		},
	}

	Ok(())
}

pub fn vec_to_string(vec: Vec<u8>) -> Result<String, ()> {
	let tmp = String::from_utf8(vec.to_vec()).map_err(|_| ())?;
	let tmp = tmp.trim();
	if tmp.is_empty() {
		return Err(());
	}
	Ok(tmp.to_string())
}
