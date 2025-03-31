/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

#[cfg(test)]
use mockall::predicate::*;
#[cfg(test)]
use mockall::*;

use super::{
	storage::{LastSidechainBlock, SidechainStorage},
	Result,
};
use its_primitives::{
	traits::{ShardIdentifierFor, SignedBlock as SignedBlockT},
	types::{BlockHash, BlockNumber},
};
use parking_lot::RwLock;
use std::path::PathBuf;

/// Lock wrapper around sidechain storage
pub struct SidechainStorageLock<SignedBlock: SignedBlockT> {
	storage: RwLock<SidechainStorage<SignedBlock>>,
}

impl<SignedBlock: SignedBlockT> SidechainStorageLock<SignedBlock> {
	pub fn from_base_path(path: PathBuf) -> Result<SidechainStorageLock<SignedBlock>> {
		Ok(SidechainStorageLock {
			storage: RwLock::new(SidechainStorage::<SignedBlock>::load_from_base_path(path)?),
		})
	}
}

/// Storage interface Trait
#[cfg_attr(test, automock)]
pub trait BlockStorage<SignedBlock: SignedBlockT> {
	// Type is not working because broadcaster needs to work with the same block type,
	// so it needs to be defined somewhere more global.
	// type SignedBlock: SignedBlockT;
	fn store_blocks(&self, blocks: Vec<SignedBlock>) -> Result<()>;
}

pub trait BlockPruner {
	/// Prune all blocks except the newest n, where n = `number_of_blocks_to_keep`.
	fn prune_blocks_except(&self, number_of_blocks_to_keep: u64);
}

#[cfg_attr(test, automock)]
pub trait FetchBlocks<SignedBlock: SignedBlockT> {
	/// Fetch all child blocks of a specified block.
	///
	/// Returns an empty vector if specified block hash cannot be found in storage.
	fn fetch_all_blocks_after(
		&self,
		block_hash: &BlockHash,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Result<Vec<SignedBlock>>;

	/// Fetch all blocks within a range, defined by a starting block (lower bound) and end block (upper bound) hash.
	///
	/// Does NOT include the bound defining blocks in the result. ]from..until[.
	/// Returns an empty vector if 'from' cannot be found in storage.
	/// Returns the same as 'fetch_all_blocks_after' if 'until' cannot be found in storage.
	fn fetch_blocks_in_range(
		&self,
		block_hash_from: &BlockHash,
		block_hash_until: &BlockHash,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Result<Vec<SignedBlock>>;

	// litentry
	fn latest_block(
		&self,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Option<LastSidechainBlock>;

	fn block_hash(
		&self,
		block_number: BlockNumber,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Option<LastSidechainBlock>;
}

impl<SignedBlock: SignedBlockT> BlockStorage<SignedBlock> for SidechainStorageLock<SignedBlock> {
	fn store_blocks(&self, blocks: Vec<SignedBlock>) -> Result<()> {
		self.storage.write().store_blocks(blocks)
	}
}

impl<SignedBlock: SignedBlockT> BlockPruner for SidechainStorageLock<SignedBlock> {
	fn prune_blocks_except(&self, number_of_blocks_to_keep: BlockNumber) {
		self.storage.write().prune_shards(number_of_blocks_to_keep);
	}
}

impl<SignedBlock: SignedBlockT> FetchBlocks<SignedBlock> for SidechainStorageLock<SignedBlock> {
	fn fetch_all_blocks_after(
		&self,
		block_hash: &BlockHash,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Result<Vec<SignedBlock>> {
		self.storage.read().get_blocks_after(block_hash, shard_identifier)
	}

	fn fetch_blocks_in_range(
		&self,
		block_hash_from: &BlockHash,
		block_hash_until: &BlockHash,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Result<Vec<SignedBlock>> {
		self.storage
			.read()
			.get_blocks_in_range(block_hash_from, block_hash_until, shard_identifier)
	}

	fn latest_block(
		&self,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Option<LastSidechainBlock> {
		self.storage
			.read()
			.last_block_of_shard(shard_identifier)
			.map(|e| LastSidechainBlock { hash: e.hash, number: e.number })
	}

	fn block_hash(
		&self,
		block_number: BlockNumber,
		shard_identifier: &ShardIdentifierFor<SignedBlock>,
	) -> Option<LastSidechainBlock> {
		match self.storage.read().get_block_hash(shard_identifier, block_number) {
			Ok(Some(block_hash)) => {
				Some(LastSidechainBlock { hash: block_hash, number: block_number })
			},
			Ok(None) => None,
			Err(e) => {
				log::error!("failed to get block_hash. due to:{:?}", e);
				None
			},
		}
	}
}
