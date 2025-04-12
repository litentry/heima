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

pub mod auth_key_store;
pub mod constants;
pub mod signer_client;

mod pumpx_api;
pub use pumpx_api::*;

use base58::ToBase58;
use executor_primitives::ChainAsset;
use log::error;
use sp_core::keccak_256;

pub fn chain_asset_to_pumpx_chain_id(asset: &ChainAsset) -> u32 {
	match asset {
		ChainAsset::Ethereum(id, _) => *id,
		ChainAsset::Solana(_) => constants::SOLANA_CHAIN_ID,
	}
}

pub fn pubkey_to_evm_address(pubkey: &[u8]) -> Result<String, ()> {
	let pubkey: [u8; 33] = pubkey.try_into().map_err(|_| {
		error!("wrong pubkey length: expect 33 bytes");
	})?;
	let uncompressed_pubkey = libsecp256k1::PublicKey::parse_slice(
		&pubkey,
		Some(libsecp256k1::PublicKeyFormat::Compressed),
	)
	.map_err(|_| {
		error!("libsecp256k1 can't parse pubkey");
	})?
	.serialize();

	let address: String =
		format!("0x{}", hex::encode(&keccak_256(&uncompressed_pubkey[1..])[12..]));
	Ok(address.to_lowercase())
}

pub fn pubkey_to_solana_address(pubkey: &[u8]) -> Result<String, ()> {
	let pubkey: [u8; 32] = pubkey.try_into().map_err(|_| {
		error!("wrong pubkey length: expect 32 bytes");
	})?;
	Ok(pubkey.to_base58())
}
