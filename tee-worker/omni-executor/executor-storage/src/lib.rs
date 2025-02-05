mod member_omni_account;
pub use member_omni_account::MemberOmniAccountStorage;
mod verification_code;
pub use verification_code::VerificationCodeStorage;
mod account_store;
pub use account_store::AccountStoreStorage;
mod oauth2_state_verifier;
pub use oauth2_state_verifier::OAuth2StateVerifierStorage;

use executor_crypto::hashing::{blake2_128, twox_128};
use executor_primitives::{AccountId, MemberAccount, TryFromSubxtType};
use frame_support::sp_runtime::traits::BlakeTwo256;
use frame_support::storage::storage_prefix;
use parentchain_api_interface::omni_account::storage::types::account_store::AccountStore;
use parentchain_rpc_client::{
	CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient, SubxtClientFactory,
};
use parity_scale_codec::Decode;
use rocksdb::DB;
use sp_state_machine::{read_proof_check, StorageProof};
use std::sync::Arc;

const STORAGE_DB_PATH: &str = "storage_db";

pub type StorageDB = DB;

pub trait Storage<K, V> {
	fn get(&self, key: &K) -> Option<V>;
	fn insert(&self, key: K, value: V) -> Result<(), ()>;
	fn remove(&self, key: &K) -> Result<(), ()>;
	fn contains_key(&self, key: &K) -> bool;
}

fn storage_key(storage_name: &str, key: &[u8]) -> Vec<u8> {
	twox_128(storage_name.as_bytes())
		.iter()
		.chain(blake2_128(key).iter().chain(key.iter())) // blake2_128_concat
		.cloned()
		.collect()
}

pub async fn init_storage(ws_rpc_endpoint: &str) -> Result<Arc<StorageDB>, ()> {
	let db = Arc::new(StorageDB::open_default(STORAGE_DB_PATH).map_err(|e| {
		log::error!("Could not open db: {:?}", e);
	})?);
	let client_factory: SubxtClientFactory<CustomConfig> = SubxtClientFactory::new(ws_rpc_endpoint);
	let mut client = client_factory.new_client().await.map_err(|e| {
		log::error!("Could not create client: {:?}", e);
	})?;

	init_omni_account_storages(&mut client, db.clone()).await?;

	Ok(db)
}

const ACCOUNT_STORE_KEYS_PAGE_SIZE: u32 = 300;

async fn init_omni_account_storages(
	client: &mut SubxtClient<CustomConfig>,
	storage_db: Arc<StorageDB>,
) -> Result<(), ()> {
	let account_store_storage = AccountStoreStorage::new(storage_db.clone());
	let member_omni_account_storage = MemberOmniAccountStorage::new(storage_db.clone());
	let account_store_key_prefix = storage_prefix(b"OmniAccount", b"AccountStore");
	let mut start_key: Option<Vec<u8>> = None;

	loop {
		let storage_keys_paged = client
			.get_storage_keys_paged(
				account_store_key_prefix.into(),
				ACCOUNT_STORE_KEYS_PAGE_SIZE,
				start_key.clone(),
			)
			.await
			.map_err(|e| {
				log::error!("Could not get storage keys paged: {:?}", e);
			})?;
		if storage_keys_paged.is_empty() || storage_keys_paged.last().cloned() == start_key {
			break;
		}
		start_key = storage_keys_paged.last().cloned();
		let proof =
			client
				.get_storage_proof_by_keys(storage_keys_paged.clone())
				.await
				.map_err(|e| {
					log::error!("Could not get storage proof by keys: {:?}", e);
				})?;
		let header = match client.get_last_finalized_header().await {
			Ok(Some(header)) => header,
			_ => {
				log::error!("Could not get last finalized header");
				return Err(());
			},
		};
		let storage_proof = StorageProof::new(proof);
		let storage_map = read_proof_check::<BlakeTwo256, _>(
			header.state_root,
			storage_proof,
			&storage_keys_paged,
		)
		.map_err(|e| {
			log::error!("Could not read proof check: {:?}", e);
		})?;

		for key in storage_keys_paged.iter() {
			match storage_map.get(key) {
				Some(Some(value)) => {
					let omni_account: AccountId = extract_account_id_from_storage_key(key)?;
					let maybe_storage_value = client
						.storage()
						.at_latest()
						.await
						.map_err(|e| {
							log::error!("Could not get storage at latest block: {:?}", e);
						})?
						.fetch_raw(key.clone())
						.await
						.map_err(|e| {
							log::error!("Could not fetch storage value: {:?}", e);
						})?;
					let Some(storage_value) = maybe_storage_value else {
						log::error!("Storage value not found for account_id: {:?}", omni_account);
						return Err(());
					};
					if storage_value != *value {
						log::error!("Storage value mismatch for account_id: {:?}", omni_account);
						return Err(());
					}
					let account_store: AccountStore =
						Decode::decode(&mut &value[..]).map_err(|e| {
							log::error!("Error decoding account store: {:?}", e);
						})?;
					for member in account_store.0.iter() {
						let member_account =
							MemberAccount::try_from_subxt_type(member).map_err(|e| {
								log::error!("Error decoding member account: {:?}", e);
							})?;
						member_omni_account_storage
							.insert(member_account.hash(), omni_account.clone())
							.map_err(|e| {
								log::error!("Error inserting member account hash: {:?}", e);
							})?;
					}
					account_store_storage.insert(omni_account, account_store).map_err(|e| {
						log::error!("Error inserting account store: {:?}", e);
					})?;
				},
				_ => {
					log::error!("No value found for key: {:?}", key);
				},
			}
		}
	}

	Ok(())
}

fn extract_account_id_from_storage_key<K: Decode>(raw_storage_key: &[u8]) -> Result<K, ()> {
	if raw_storage_key.len() < 32 {
		return Err(());
	}
	let mut raw_key = &raw_storage_key[raw_storage_key.len() - 32..];
	K::decode(&mut raw_key).map_err(|e| {
		log::error!("Error decoding account id: {:?}", e);
	})
}
