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
// along with Litentry. If not, see <https://www.gnu.org/licenses/>.

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use litentry_primitives::Web3NftType;

use crate::Web3Network;

pub trait NftName {
	fn get_nft_name(&self) -> &'static str;
}

impl NftName for Web3NftType {
	fn get_nft_name(&self) -> &'static str {
		match self {
			Self::WeirdoGhostGang => "Weirdo Ghost Gang",
			Self::Club3Sbt => "Club3 SBT",
			Self::MFan => "MFAN",
			Self::Mvp => "MEME VIP PASS",
		}
	}
}

pub trait NftAddress {
	fn get_nft_address(&self, network: Web3Network) -> Option<&'static str>;
}

impl NftAddress for Web3NftType {
	fn get_nft_address(&self, network: Web3Network) -> Option<&'static str> {
		match (self, network) {
			// WeirdoGhostGang
			(Self::WeirdoGhostGang, Web3Network::Ethereum) => {
				Some("0x9401518f4EBBA857BAA879D9f76E1Cc8b31ed197")
			},
			// Club3Sbt
			(Self::Club3Sbt, Web3Network::Bsc) => {
				Some("0x9f488C0dafb1B3bFeeD3e886e0E6E5f3f4517925")
			},
			(Self::Club3Sbt, Web3Network::Polygon) => {
				Some("0xAc2e4e67cffa5E82bfA1e169e5F9aa405114C982")
			},
			(Self::Club3Sbt, Web3Network::Arbitrum) => {
				Some("0xcccFF19FB8a4a2A206d07842b8F8c8c0A11904C2")
			},
			// MFan
			(Self::MFan, Web3Network::Polygon) => {
				Some("0x9aBc7C604C27622f9CD56bd1628F6321c32bBBf6")
			},
			// Mvp
			(Self::Mvp, Web3Network::Ethereum) => {
				Some("0xAA813F8691B10Dc62bd616ae90b05A52f0C40C1D")
			},
			_ => None,
		}
	}
}
