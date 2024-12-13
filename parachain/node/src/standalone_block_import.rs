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

// based on Acala's instant_finalize implementation
// used in standalone mode to help to immediately finalize the block

use sc_consensus::BlockImport;
use sp_runtime::traits::Block as BlockT;

pub struct StandaloneBlockImport<I> {
	inner: I,
}

impl<I> StandaloneBlockImport<I> {
	pub fn new(inner: I) -> Self {
		Self { inner }
	}
}

#[async_trait::async_trait]
impl<Block, I> BlockImport<Block> for StandaloneBlockImport<I>
where
	Block: BlockT,
	I: BlockImport<Block> + Send + std::marker::Sync,
{
	type Error = I::Error;

	async fn check_block(
		&self,
		block: sc_consensus::BlockCheckParams<Block>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		self.inner.check_block(block).await
	}

	async fn import_block(
		&mut self,
		mut block_import_params: sc_consensus::BlockImportParams<Block>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		// immediately finalize the block
		block_import_params.finalized = true;
		self.inner.import_block(block_import_params).await
	}
}
