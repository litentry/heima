mod account_store;
pub use account_store::AccountStoreStorage;

use executor_core::storage::Storage;
use parentchain_rpc_client::{
	CustomConfig, SubstrateRpcClientFactory, SubxtClient, SubxtClientFactory,
};
use parity_scale_codec::Decode;
use subxt_core::utils::AccountId32 as AccountId;

const STORAGE_PATH: &str = "storage";

pub async fn init_storage(ws_rpc_endpoint: &str) -> Result<(), ()> {
	let client_factory: SubxtClientFactory<CustomConfig> = SubxtClientFactory::new(ws_rpc_endpoint);
	let client = client_factory.new_client().await.map_err(|e| {
		log::error!("Could not create client: {:?}", e);
	})?;

	init_account_store_storage(&client).await?;

	Ok(())
}

async fn init_account_store_storage(client: &SubxtClient<CustomConfig>) -> Result<(), ()> {
	let storage_query = parentchain_api_interface::storage().omni_account().account_store_iter();
	let mut account_store_iter = client
		.storage()
		.at_latest()
		.await
		.map_err(|e| {
			log::error!("Could not get storage at latest block: {:?}", e);
		})?
		.iter(storage_query)
		.await
		.map_err(|e| {
			log::error!("Could not iterate account store: {:?}", e);
		})?;

	let account_store_storage = AccountStoreStorage::new();

	while let Some(Ok(kv)) = account_store_iter.next().await {
		let account_id = AccountId::decode(&mut &kv.key_bytes[..]).map_err(|e| {
			log::error!("Error decoding account id: {:?}", e);
		})?;
		account_store_storage.insert(account_id, kv.value).map_err(|e| {
			log::error!("Error inserting account store: {:?}", e);
		})?;
	}

	Ok(())
}
