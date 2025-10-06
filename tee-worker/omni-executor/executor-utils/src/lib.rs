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

pub mod response_types;
pub use response_types::*;

use base58::ToBase58;
use ethers::core::utils::to_checksum;
use executor_primitives::ChainAsset;
use sp_core::keccak_256;
use tracing::error;

pub const SOLANA_CHAIN_ID: u32 = 101;

pub fn chain_asset_to_chain_id(asset: &ChainAsset) -> u32 {
	match asset {
		ChainAsset::Ethereum(id, _) => *id,
		ChainAsset::Solana(_) => SOLANA_CHAIN_ID,
	}
}

fn pubkey_to_evm_address_bytes(pubkey: &[u8]) -> Result<[u8; 20], ()> {
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
	Ok(keccak_256(&uncompressed_pubkey[1..])[12..].try_into().unwrap())
}

pub fn pubkey_to_evm_address(pubkey: &[u8]) -> Result<String, ()> {
	pubkey_to_evm_address_bytes(pubkey).map(|bytes| to_checksum(&bytes.into(), None))
}

pub fn pubkey_to_solana_address(pubkey: &[u8]) -> Result<String, ()> {
	let pubkey: [u8; 32] = pubkey.try_into().map_err(|_| {
		error!("wrong pubkey length: expect 32 bytes");
	})?;
	Ok(pubkey.to_base58())
}

// Re-export ChainType from signer-client for convenience
pub use signer_client::ChainType;

pub fn pubkey_to_address(
	chain_type: signer_client::ChainType,
	pubkey: &[u8],
) -> Result<String, ()> {
	match chain_type {
		signer_client::ChainType::Evm => pubkey_to_evm_address(pubkey),
		signer_client::ChainType::Solana => pubkey_to_solana_address(pubkey),
		_ => {
			error!("Unsupported {:?} wallet address", chain_type);
			Err(())
		},
	}
}
