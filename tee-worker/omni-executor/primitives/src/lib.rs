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
pub mod utils;
pub use heima_primitives::{
	omni_account::{MemberAccount, OmniAccountAuthType},
	BlockNumber, Hash, Identity, Nonce, ShardIdentifier, Web2IdentityType,
};
pub use sp_core::crypto::AccountId32 as AccountId;

use parity_scale_codec::{Decode, Encode};
use std::fmt::Debug;

pub trait GetEventId<Id> {
	fn get_event_id(&self) -> Id;
}

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

pub trait TryFromSubxtType<T: Encode>: Sized {
	fn try_from_subxt_type(t: T) -> Result<Self, ()>;
}

impl<T: Encode> TryFromSubxtType<T> for Identity {
	fn try_from_subxt_type(t: T) -> Result<Self, ()> {
		let bytes = t.encode();
		Identity::decode(&mut &bytes[..]).map_err(|_| ())
	}
}

impl<T: Encode> TryFromSubxtType<T> for MemberAccount {
	fn try_from_subxt_type(t: T) -> Result<Self, ()> {
		let bytes = t.encode();
		MemberAccount::decode(&mut &bytes[..]).map_err(|_| ())
	}
}
