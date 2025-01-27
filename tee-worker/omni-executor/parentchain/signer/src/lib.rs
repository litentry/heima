pub mod key_store;
mod signer;
pub use signer::TransactionSigner;

use executor_core::key_store::KeyStore;
use key_store::SubstrateKeyStore;
use log::{error, info};
use std::sync::Arc;
use subxt_core::utils::AccountId32;
use subxt_signer::sr25519::Keypair;

pub fn get_signer(key_store: Arc<SubstrateKeyStore>) -> Keypair {
	let secret_key_bytes = key_store
		.read()
		.map_err(|e| {
			error!("Could not unseal key: {:?}", e);
		})
		.unwrap();
	let signer = subxt_signer::sr25519::Keypair::from_secret_key(secret_key_bytes)
		.map_err(|e| {
			error!("Could not create secret key: {:?}", e);
		})
		.unwrap();

	info!("Substrate signer address: {}", AccountId32::from(signer.public_key()));
	signer
}
