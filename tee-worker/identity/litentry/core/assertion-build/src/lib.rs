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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_large_err)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use http_req_sgx as http_req;
	pub use http_sgx as http;
	pub use rust_base58_sgx as rust_base58;
	pub use thiserror_sgx as thiserror;
}

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

pub mod a1;
pub mod a13;
pub mod a14;
pub mod a2;
pub mod a20;
pub mod a3;
pub mod a6;
pub mod a8;
pub mod achainable;
pub mod brc20;
pub mod dynamic;
pub mod generic_discord_role;
pub mod holding_time;
pub mod lit_staking;
pub mod nodereal;
pub mod oneblock;
pub mod vip3;

use blake2_rfc::blake2b::Blake2b;
use itp_types::AccountId;
use itp_utils::stringify::{account_id_to_string, account_id_to_string_without_prefix};
use lc_credentials::Credential;
use lc_data_providers::achainable::web3_network_to_chain;
use litentry_primitives::{
	p2pkh_address, p2sh_address, p2tr_address, p2wpkh_address, AchainableAmount,
	AchainableAmountHolding, AchainableAmountToken, AchainableAmounts, AchainableBasic,
	AchainableBetweenPercents, AchainableDate, AchainableDateInterval, AchainableDatePercent,
	AchainableParams, AchainableToken, Assertion, AssertionBuildRequest, DynamicParams,
	ErrorDetail, ErrorString, Identity, IdentityNetworkTuple, IntoErrorDetail, OneBlockCourseType,
	ParameterString, VCMPError as Error, Web3Network,
};
use log::*;
use rust_base58::ToBase58;
use ss58_registry::Ss58AddressFormat;
use std::{
	collections::HashSet,
	format,
	string::{String, ToString},
	sync::Arc,
	vec,
	vec::Vec,
};

pub type Result<T> = core::result::Result<T, Error>;

/// Transpose a vector of identities with web3network information, which is Vec<IdentityNetworkTuple>,
/// to the vector of hex addresses which share the same network type, which is Vec<(Web3Network, Vec<String>)>.
///
/// TODO: improve the logic
pub fn transpose_identity(
	identities: &Vec<IdentityNetworkTuple>,
) -> Vec<(Web3Network, Vec<String>)> {
	let mut addresses: Vec<(String, Web3Network)> = vec![];
	let mut networks_set: HashSet<Web3Network> = HashSet::new();
	for (id, networks) in identities {
		networks.clone().into_iter().for_each(|n| {
			match id {
				Identity::Substrate(address) => {
					let network = web3_network_to_chain(&n);
					let ss58_address = ss58_address_of(address.as_ref(), &network);
					if let Ok(address) = ss58_address {
						addresses.push((address, n));
						networks_set.insert(n);
					}
				},
				Identity::Evm(address) => {
					let address = account_id_to_string(address.as_ref());
					addresses.push((address, n));
					networks_set.insert(n);
				},
				Identity::Bitcoin(address) => {
					let address = account_id_to_string_without_prefix(address.as_ref());
					let address = pubkey_to_address(&n, &address);
					addresses.push((address, n));
					networks_set.insert(n);
				},
				Identity::Solana(address) => {
					let address = address.as_ref().to_base58();
					addresses.push((address, n));
					networks_set.insert(n);
				},
				_ => {},
			};
		});
	}

	networks_set
		.iter()
		.map(|n| {
			let address: Vec<String> = addresses
				.iter()
				.filter(|(_, network)| n == network)
				.map(|(addr, _)| addr.clone())
				.collect();
			(*n, address)
		})
		.collect()
}

// mostly copied from https://github.com/hack-ink/substrate-minimal/blob/main/subcryptor/src/lib.rs
// no_std version is used here
pub fn ss58_address_of(
	public_key: &[u8],
	network: &str,
) -> core::result::Result<String, ErrorDetail> {
	let network = Ss58AddressFormat::try_from(network).map_err(|_| ErrorDetail::ParseError)?;
	let prefix = u16::from(network);
	let mut bytes = match prefix {
		0..=63 => vec![prefix as u8],
		64..=16_383 => {
			let first = ((prefix & 0b0000_0000_1111_1100) as u8) >> 2;
			let second = ((prefix >> 8) as u8) | ((prefix & 0b0000_0000_0000_0011) as u8) << 6;

			vec![first | 0b01000000, second]
		},
		_ => Err(ErrorDetail::ParseError)?,
	};

	bytes.extend(public_key);

	let blake2b = {
		let mut context = Blake2b::new(64);
		context.update(b"SS58PRE");
		context.update(&bytes);
		context.finalize()
	};

	bytes.extend(&blake2b.as_bytes()[0..2]);

	Ok(bytes.to_base58())
}

fn pubkey_to_address(network: &Web3Network, pubkey: &str) -> String {
	match network {
		Web3Network::BitcoinP2tr => p2tr_address(pubkey),
		Web3Network::BitcoinP2pkh => p2pkh_address(pubkey),
		Web3Network::BitcoinP2sh => p2sh_address(pubkey),
		Web3Network::BitcoinP2wpkh => p2wpkh_address(pubkey),
		Web3Network::BitcoinP2wsh => "".to_string(),
		Web3Network::Polkadot
		| Web3Network::Kusama
		| Web3Network::Litentry
		| Web3Network::Litmus
		| Web3Network::LitentryRococo
		| Web3Network::Khala
		| Web3Network::SubstrateTestnet
		| Web3Network::Ethereum
		| Web3Network::Bsc
		| Web3Network::Polygon
		| Web3Network::Arbitrum
		| Web3Network::Combo
		| Web3Network::Solana => "".to_string(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use itp_utils::ToHexPrefixed;
	use litentry_primitives::IdentityString;
	use rust_base58::FromBase58;

	#[test]
	fn transpose_identity_works() {
		let mut identities: Vec<IdentityNetworkTuple> = vec![];
		let id1 = Identity::Twitter(IdentityString::new("alice1".as_bytes().to_vec()));
		let id2 = [
			122, 14, 95, 161, 63, 226, 172, 179, 35, 141, 125, 220, 137, 243, 163, 0, 157, 186,
			194, 62, 45, 146, 65, 73, 222, 151, 78, 242, 131, 85, 243, 21,
		]
		.into();
		let id3 = [
			80, 35, 128, 158, 237, 244, 169, 75, 45, 16, 131, 141, 244, 63, 115, 159, 160, 91, 181,
			12, 104, 74, 172, 100, 15, 193, 96, 5, 27, 206, 248, 12,
		]
		.into();
		let id4 = [4_u8; 20].into();
		let id5 = Identity::Solana(
			"EJpLyTeE8XHG9CeREeHd6pr6hNhaRnTRJx4Z5DPhEJJ6"
				.from_base58()
				.unwrap()
				.as_slice()
				.try_into()
				.unwrap(),
		);

		let network1: Vec<Web3Network> = vec![];
		let network2 = vec![Web3Network::Polkadot, Web3Network::Litentry];
		let network3 = vec![Web3Network::Litentry, Web3Network::Litmus, Web3Network::Kusama];
		let network4 = vec![Web3Network::Bsc];
		let network5 = vec![Web3Network::Solana];

		identities.push((id1, network1));
		identities.push((id2, network2));
		identities.push((id3, network3));
		identities.push((id4, network4));
		identities.push((id5, network5));

		let mut result = transpose_identity(&identities);
		result.sort();
		debug!(">> {result:?}");
		assert_eq!(result.len(), 6);
		assert_eq!(
			result.get(0).unwrap(),
			&(
				Web3Network::Polkadot,
				vec!["13m37Uzx2PHjfGPt15TWXjqKBHTGVKFXKpZdxsQXDG3fL7F5".into()]
			)
		);
		assert_eq!(
			result.get(1).unwrap(),
			&(Web3Network::Kusama, vec!["EPPtryxh22cfPGFQ6KY6Gb2No46hbhTcUuH5myMJNfbPkEf".into()])
		);
		assert_eq!(
			result.get(2).unwrap(),
			&(
				Web3Network::Litentry,
				vec![
					"49AV8EnSwQQGW5wG5ajFEg5VDKtPBzmfAyji34LrZmUn46Rt".into(),
					"48DXPdgeqTPhC5zhfXqE3QJM7sCdHuxZ5ky5vbd5jAujYwyR".into()
				]
			)
		);
		assert_eq!(
			result.get(3).unwrap(),
			&(
				Web3Network::Litmus,
				vec!["jcP3mX494vUPJj8LzjjTmsq2zZrcXxTx1F2xywPY4RGsmMWQx".into()]
			)
		);
		assert_eq!(result.get(4).unwrap(), &(Web3Network::Bsc, vec![[4u8; 20].to_hex()]));
		assert_eq!(
			result.get(5).unwrap(),
			&(Web3Network::Solana, vec!["EJpLyTeE8XHG9CeREeHd6pr6hNhaRnTRJx4Z5DPhEJJ6".into()])
		);
	}

	#[test]
	fn pubkey_to_address_works() {
		// p2wpkh
		let addr = "bc1qlht0muueu6wln5qqwtvczjepnfeerpaw480067";
		let pubkey = "0272cbf3e56e238897ca9ee9ca9594a82803cfdf19121bd939cbe3f2e1bcaffc7b";
		let network = Web3Network::BitcoinP2wpkh;
		let gen_addr = pubkey_to_address(&network, pubkey);
		assert_eq!(addr, gen_addr);

		// p2sh
		let addr = "35KQSeZpaABvNWmKAMXo7mAAtXZBqCd4sw";
		let pubkey = "02e46883d2101f09e875dd4a67ee5c2dce9d821b9a610a7e12ab0de7494b19b7d0";
		let network = Web3Network::BitcoinP2sh;
		let gen_addr = pubkey_to_address(&network, pubkey);
		assert_eq!(addr, gen_addr);

		// p2tr
		let addr = "bc1pwgqves622fs5s42h4sr8hu9y6ej232hga8uxal7xgkcsy8a3ryqqvgku7t";
		let pubkey = "031d867537093a8eaace96717ba0aa226a5bf368c6c84ca5dfb214d380bc91afbe";
		let network = Web3Network::BitcoinP2tr;
		let gen_addr = pubkey_to_address(&network, pubkey);
		assert_eq!(addr, gen_addr);

		// p2pkh
		let addr = "1CY8nArJbvLSHQmKp3SiG8T5WSBfnMJpJx";
		let pubkey = "02784a686e5ffc74d713f66cb8885d6b75c062c61df5f6de8f86f07c340ebc183c";
		let network = Web3Network::BitcoinP2pkh;
		let gen_addr = pubkey_to_address(&network, pubkey);
		assert_eq!(addr, gen_addr);

		// p2wsh
		let addr = "";
		let pubkey = "02e46883d2101f09e875dd4a67ee5c2dce9d821b9a610a7e12ab0de7494b19b7d0";
		let network = Web3Network::BitcoinP2wsh;
		let gen_addr = pubkey_to_address(&network, pubkey);
		assert_eq!(addr, gen_addr);
	}
}
