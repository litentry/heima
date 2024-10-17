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

use crate::rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClientFactory};
use async_trait::async_trait;
use parity_scale_codec::Decode;
use subxt::{Config, Metadata};

#[async_trait]
pub trait MetadataProvider<M> {
	async fn get(&self, block_num: u64) -> M;
}

pub struct SubxtMetadataProvider<ChainConfig: Config> {
	client_factory: SubxtClientFactory<ChainConfig>,
}

impl<ChainConfig: Config> SubxtMetadataProvider<ChainConfig> {
	pub fn new(client_factory: SubxtClientFactory<ChainConfig>) -> Self {
		Self { client_factory }
	}
}

#[async_trait]
impl<ChainConfig: Config> MetadataProvider<Metadata> for SubxtMetadataProvider<ChainConfig> {
	async fn get(&self, block_num: u64) -> Metadata {
		let mut client = self.client_factory.new_client().await.unwrap();
		let raw_metadata = client.get_raw_metadata(block_num).await.unwrap();

		Metadata::decode(&mut raw_metadata.as_slice()).unwrap()
	}
}
