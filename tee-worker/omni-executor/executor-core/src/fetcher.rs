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
use primitives::GetEventId;

/// Returns the last finalized block number
#[async_trait]
pub trait LastFinalizedBlockNumFetcher {
	async fn get_last_finalized_block_num(&mut self) -> Result<Option<u64>, ()>;
}

/// Returns all events emitted on given chain
#[async_trait]
pub trait EventsFetcher<EventId, BlockEvent: GetEventId<EventId>> {
	async fn get_block_events(&mut self, block_num: u64) -> Result<Vec<BlockEvent>, ()>;
}
