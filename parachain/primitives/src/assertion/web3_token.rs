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

use crate::assertion::network::Web3Network;
use crate::{vec, Vec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum Web3TokenType {
	#[codec(index = 0)]
	Bnb,
	#[codec(index = 1)]
	Eth,
	#[codec(index = 2)]
	SpaceId,
	#[codec(index = 3)]
	Lit,
	#[codec(index = 4)]
	Wbtc,
	#[codec(index = 5)]
	Usdc,
	#[codec(index = 6)]
	Usdt,
	#[codec(index = 7)]
	Crv,
	#[codec(index = 8)]
	Matic,
	#[codec(index = 9)]
	Dydx,
	#[codec(index = 10)]
	Amp,
	#[codec(index = 11)]
	Cvx,
	#[codec(index = 12)]
	Tusd,
	#[codec(index = 13)]
	Usdd,
	#[codec(index = 14)]
	Gusd,
	#[codec(index = 15)]
	Link,
	#[codec(index = 16)]
	Grt,
	#[codec(index = 17)]
	Comp,
	#[codec(index = 18)]
	People,
	#[codec(index = 19)]
	Gtc,
	#[codec(index = 20)]
	Ton,
	#[codec(index = 21)]
	Trx,
	#[codec(index = 22)]
	Nfp,
	#[codec(index = 23)]
	Sol,
	#[codec(index = 24)]
	Mcrt,
	#[codec(index = 25)]
	Btc,
	#[codec(index = 26)]
	Ada,
	#[codec(index = 27)]
	Doge,
	#[codec(index = 28)]
	Shib,
	#[codec(index = 29)]
	Uni,
	#[codec(index = 30)]
	Bch,
	#[codec(index = 31)]
	Etc,
	#[codec(index = 32)]
	Atom,
	#[codec(index = 33)]
	Dai,
	#[codec(index = 34)]
	Leo,
	#[codec(index = 35)]
	Fil,
	#[codec(index = 36)]
	Imx,
	#[codec(index = 37)]
	Cro,
	#[codec(index = 38)]
	Inj,
	#[codec(index = 39)]
	Bean,
	#[codec(index = 40)]
	An,
	#[codec(index = 41)]
	Tuna,
}

impl Web3TokenType {
	pub fn get_supported_networks(&self) -> Vec<Web3Network> {
		match self {
			Self::Bnb | Self::Eth | Self::SpaceId | Self::Ton | Self::Trx | Self::Inj => {
				vec![Web3Network::Bsc, Web3Network::Ethereum]
			},
			Self::Lit => vec![Web3Network::Bsc, Web3Network::Ethereum, Web3Network::Litentry],
			Self::Nfp | Self::Ada | Self::Doge | Self::Bch | Self::Etc | Self::Fil => {
				vec![Web3Network::Bsc]
			},
			Self::Sol | Self::Mcrt => {
				vec![Web3Network::Bsc, Web3Network::Ethereum, Web3Network::Solana]
			},
			Self::Btc => vec![
				Web3Network::BitcoinP2tr,
				Web3Network::BitcoinP2pkh,
				Web3Network::BitcoinP2sh,
				Web3Network::BitcoinP2wpkh,
				Web3Network::BitcoinP2wsh,
			],
			Self::Usdc | Self::Uni | Self::Dai => vec![
				Web3Network::Ethereum,
				Web3Network::Bsc,
				Web3Network::Solana,
				Web3Network::Arbitrum,
				Web3Network::Polygon,
			],
			Self::Shib | Self::Leo | Self::Imx => vec![Web3Network::Ethereum],
			Self::Atom => vec![Web3Network::Ethereum, Web3Network::Bsc, Web3Network::Polygon],
			Self::Cro => vec![Web3Network::Ethereum, Web3Network::Solana],
			Self::Bean => vec![Web3Network::Bsc, Web3Network::Combo],
			Self::An => vec![Web3Network::Bsc],
			_ => vec![Web3Network::Ethereum],
		}
	}
}
