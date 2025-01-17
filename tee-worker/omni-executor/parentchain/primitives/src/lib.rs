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

pub mod signature;
pub use heima_primitives::*;

use executor_core::primitives::GetEventId;
use std::fmt::Debug;

/// Used to uniquely identify intent event on parentchain.
#[derive(Clone, Debug)]
pub struct EventId {
	pub block_num: u64,
	pub event_idx: u64,
}

impl EventId {
	pub fn new(block_num: u64, event_idx: u64) -> Self {
		Self { block_num, event_idx }
	}
}

pub struct BlockEvent {
	pub id: EventId,
	pub pallet_name: String,
	pub variant_name: String,
	pub variant_index: u8,
	pub field_bytes: Vec<u8>,
}

impl BlockEvent {
	pub fn new(
		id: EventId,
		pallet_name: String,
		variant_name: String,
		variant_index: u8,
		field_bytes: Vec<u8>,
	) -> Self {
		Self { id, pallet_name, variant_name, variant_index, field_bytes }
	}
}

impl GetEventId<EventId> for BlockEvent {
	fn get_event_id(&self) -> EventId {
		self.id.clone()
	}
}
