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

mod validation_data;
pub use validation_data::{
	DiscordValidationData, TwitterValidationData, ValidationData, Web2ValidationData,
	Web3ValidationData,
};
mod auth;
pub use auth::*;

pub mod signature;
pub mod utils;
pub use heima_primitives::{
	identity::Address32, omni::*, teebag::DcapQuote, AccountId, BlockNumber, ChainAsset, Hash,
	Hashable, Identity, IntentId, MrEnclave, Nonce, ShardIdentifier, Web2IdentityType,
};
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

#[derive(Encode, Decode)]
pub struct PumpxAccountProfile {
	pub wallet_exported: bool,
}

/// Represents supported blockchain networks for omni operations
#[derive(
	Clone, Debug, Hash, Encode, Decode, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub enum Chain {
	Evm(u64),
	Solana,
}

impl Chain {
	pub fn evm_chain_id(&self) -> Option<u64> {
		match self {
			Chain::Evm(chain_id) => Some(*chain_id),
			Chain::Solana => None,
		}
	}

	pub fn is_evm(&self) -> bool {
		matches!(self, Chain::Evm(_))
	}

	pub fn is_solana(&self) -> bool {
		matches!(self, Chain::Solana)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_chain_evm_variants() {
		let eth_mainnet = Chain::Evm(1);
		let sepolia = Chain::Evm(11155111);
		let local = Chain::Evm(31337);

		assert!(eth_mainnet.is_evm());
		assert_eq!(eth_mainnet.evm_chain_id(), Some(1));

		assert!(sepolia.is_evm());
		assert_eq!(sepolia.evm_chain_id(), Some(11155111));

		assert!(local.is_evm());
		assert_eq!(local.evm_chain_id(), Some(31337));
	}

	#[test]
	fn test_chain_solana_variant() {
		let solana = Chain::Solana;

		assert!(solana.is_solana());
		assert!(!solana.is_evm());
		assert_eq!(solana.evm_chain_id(), None);
	}

	#[test]
	fn test_chain_serialization() {
		let eth_mainnet = Chain::Evm(1);
		let solana = Chain::Solana;

		// Test that serialization/deserialization works
		let eth_json = serde_json::to_string(&eth_mainnet).unwrap();
		let solana_json = serde_json::to_string(&solana).unwrap();

		let eth_deserialized: Chain = serde_json::from_str(&eth_json).unwrap();
		let solana_deserialized: Chain = serde_json::from_str(&solana_json).unwrap();

		assert_eq!(eth_mainnet, eth_deserialized);
		assert_eq!(solana, solana_deserialized);
	}
}
