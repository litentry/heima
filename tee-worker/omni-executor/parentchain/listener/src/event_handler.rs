// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use async_trait::async_trait;
use executor_core::event_handler::{Error, EventHandler as EventHandlerTrait};
use executor_primitives::{AccountId, BlockEvent, Hash, MemberAccount};
use executor_storage::Storage;
use parentchain_api_interface::omni_account::events::AccountStoreUpdated;
use parentchain_rpc_client::{
	metadata::{MetadataProvider, SubxtMetadataProvider},
	RpcClientHeader,
};
use parity_scale_codec::{Decode, Encode};
use std::marker::PhantomData;
use std::{sync::Arc, vec::Vec};
use subxt::ext::scale_decode;
use subxt::ext::scale_decode::DecodeAsFields;
use subxt::{events::StaticEvent, Config, Metadata};
use subxt_core::config::DefaultExtrinsicParams;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};

type AccountStore = Vec<MemberAccount>;

pub struct EventHandler<
	MetadataT,
	MetadataProviderT: MetadataProvider<MetadataT>,
	AccountStoreStorage: Storage<AccountId, AccountStore>,
	MemberOmniAccountStorage: Storage<Hash, AccountId>,
> {
	metadata_provider: Arc<MetadataProviderT>,
	account_store_storage: Arc<AccountStoreStorage>,
	member_account_storage: Arc<MemberOmniAccountStorage>,
	phantom_data: PhantomData<MetadataT>,
}

impl<
		MetadataT,
		MetadataProviderT: MetadataProvider<MetadataT>,
		AccountStoreStorage: Storage<AccountId, AccountStore>,
		MemberOmniAccountStorage: Storage<Hash, AccountId>,
	> EventHandler<MetadataT, MetadataProviderT, AccountStoreStorage, MemberOmniAccountStorage>
{
	pub fn new(
		metadata_provider: Arc<MetadataProviderT>,
		account_store_storage: Arc<AccountStoreStorage>,
		member_account_storage: Arc<MemberOmniAccountStorage>,
	) -> Self {
		Self {
			metadata_provider,
			account_store_storage,
			member_account_storage,
			phantom_data: Default::default(),
		}
	}
}

#[async_trait]
impl<
		ChainConfig: Config<
			ExtrinsicParams = DefaultExtrinsicParams<ChainConfig>,
			AccountId = AccountId32,
			Address = MultiAddress<AccountId32, u32>,
			Signature = MultiSignature,
			Header = RpcClientHeader,
		>,
		AccountStoreStorage: Storage<AccountId, AccountStore> + Send + Sync,
		MemberOmniAccountStorage: Storage<Hash, AccountId> + Send + Sync,
	> EventHandlerTrait<BlockEvent>
	for EventHandler<
		Metadata,
		SubxtMetadataProvider<ChainConfig>,
		AccountStoreStorage,
		MemberOmniAccountStorage,
	>
{
	async fn handle(&self, event: BlockEvent) -> Result<(), Error> {
		log::debug!("Got event: {:?}, variant name: {}", event.id, event.variant_name);

		if event.pallet_name != "OmniAccount" {
			// we are not interested in this event
			log::debug!("Not interested in {} events", event.pallet_name);
			return Ok(());
		}

		log::debug!("Got IntentRequested event: {:?}", event.id);

		let metadata = self.metadata_provider.get(Some(event.id.block_num)).await;

		let pallet = metadata.pallet_by_name(&event.pallet_name).ok_or_else(move || {
			log::error!(
				"No pallet metadata found for event {} and pallet {} ",
				event.id.block_num,
				event.pallet_name
			);
			Error::NonRecoverableError
		})?;
		let variant = pallet.event_variant_by_index(event.variant_index).ok_or_else(move || {
			log::error!(
				"No event variant metadata found for event {} and variant {}",
				event.id.block_num,
				event.variant_index
			);
			Error::NonRecoverableError
		})?;

		let fields = variant
			.fields
			.iter()
			.map(|f| scale_decode::Field::new(f.ty.id, f.name.as_deref()));

		match variant.name.as_str() {
			AccountStoreUpdated::EVENT => {
				let account_store_updated: AccountStoreUpdated =
					AccountStoreUpdated::decode_as_fields(
						&mut event.field_bytes.as_slice(),
						&mut fields.clone(),
						metadata.types(),
					)
					.map_err(|_| {
						log::error!("Could not decode event {:?}", event.id);
						Error::NonRecoverableError
					})?;

				let omni_account = AccountId::new(account_store_updated.who.0);
				let mut account_store: Vec<MemberAccount> = Vec::new();

				for member in account_store_updated.account_store.0.iter() {
					let member_bytes = member.encode();
					let member_account: MemberAccount = Decode::decode(&mut &member_bytes[..])
						.map_err(|e| {
							log::error!("Error decoding member account: {:?}", e);
							Error::NonRecoverableError
						})?;
					self.member_account_storage
						.insert(member_account.hash(), omni_account.clone())
						.map_err(|e| {
							log::error!("Error inserting member account hash: {:?}", e);
							Error::NonRecoverableError
						})?;
					account_store.push(member_account);
				}
				self.account_store_storage.insert(omni_account, account_store).map_err(|_| {
					log::error!("Could not insert account store into storage");
					Error::NonRecoverableError
				})?;
			},
			_ => {
				log::debug!("Not interested in {} events", event.variant_name);
			},
		}

		Ok(())
	}
}
