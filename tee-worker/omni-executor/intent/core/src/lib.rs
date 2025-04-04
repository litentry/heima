use async_trait::async_trait;
use executor_primitives::AccountId;
use std::collections::HashMap;
use tokio::sync::RwLock;

type IntentId = heima_primitives::IntentId;

#[async_trait]
pub trait IntentIdStore {
	async fn get(&self, account_id: &AccountId) -> Result<Option<IntentId>, ()>;
	async fn update(&self, account_id: AccountId, id: IntentId) -> Result<(), ()>;
}

pub struct InMemoryIntentIdStore {
	inner: RwLock<HashMap<AccountId, IntentId>>,
}
#[async_trait]
impl IntentIdStore for InMemoryIntentIdStore {
	async fn get(&self, account_id: &AccountId) -> Result<Option<IntentId>, ()> {
		Ok(self.inner.read().await.get(account_id).copied())
	}

	async fn update(&self, account_id: AccountId, id: IntentId) -> Result<(), ()> {
		self.inner.write().await.insert(account_id, id);
		Ok(())
	}
}
