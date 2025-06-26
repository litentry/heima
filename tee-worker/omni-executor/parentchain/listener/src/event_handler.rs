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
use executor_primitives::BlockEvent;
use parentchain_rpc_client::{
	metadata::{MetadataProvider, SubxtMetadataProvider},
	RpcClientHeader,
};
use std::marker::PhantomData;
use std::sync::Arc;
use subxt::{Config, Metadata};
use subxt_core::config::DefaultExtrinsicParams;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};
use tracing::debug;

pub struct EventHandler<
	MetadataT,
	MetadataProviderT: MetadataProvider<MetadataT>,
> {
	_metadata_provider: Arc<MetadataProviderT>,
	phantom_data: PhantomData<MetadataT>,
}

impl<
		MetadataT,
		MetadataProviderT: MetadataProvider<MetadataT>,
	> EventHandler<MetadataT, MetadataProviderT>
{
	pub fn new(
		metadata_provider: Arc<MetadataProviderT>,
	) -> Self {
		Self {
			_metadata_provider: metadata_provider,
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
	> EventHandlerTrait<BlockEvent>
	for EventHandler<
		Metadata,
		SubxtMetadataProvider<ChainConfig>,
	>
{
	async fn handle(&self, event: BlockEvent) -> Result<(), Error> {
		debug!("Got event: {:?}, variant name: {}", event.id, event.variant_name);

		if event.pallet_name != "OmniAccount" {
			// we are not interested in this event
			debug!("Not interested in {} events", event.pallet_name);
			return Ok(());
		}

		// Currently no OmniAccount events to handle after AccountStore removal
		debug!("OmniAccount event received but no handlers configured");

		Ok(())
	}
}
