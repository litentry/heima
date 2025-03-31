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

use litentry_primitives::{Web3Network, Web3TokenType};

pub const DEFAULT_TOKEN_DECIMALS: u32 = 1;

pub const TOKEN_DECIMALS_2: (u32, [(Web3TokenType, Web3Network); 1]) = (
	2,
	[
		// Gusd
		(Web3TokenType::Gusd, Web3Network::Ethereum),
	],
);

pub const TOKEN_DECIMALS_6: (u32, [(Web3TokenType, Web3Network); 9]) = (
	6,
	[
		// Usdc
		(Web3TokenType::Usdc, Web3Network::Ethereum),
		(Web3TokenType::Usdc, Web3Network::Solana),
		(Web3TokenType::Usdc, Web3Network::Arbitrum),
		(Web3TokenType::Usdc, Web3Network::Polygon),
		// Usdt
		(Web3TokenType::Usdt, Web3Network::Ethereum),
		// Trx
		(Web3TokenType::Trx, Web3Network::Bsc),
		(Web3TokenType::Trx, Web3Network::Ethereum),
		// Atom
		(Web3TokenType::Atom, Web3Network::Ethereum),
		(Web3TokenType::Atom, Web3Network::Polygon),
	],
);

pub const TOKEN_DECIMALS_8: (u32, [(Web3TokenType, Web3Network); 13]) = (
	8,
	[
		// Wbtc
		(Web3TokenType::Wbtc, Web3Network::Bsc),
		(Web3TokenType::Wbtc, Web3Network::Ethereum),
		// Mcrt
		(Web3TokenType::Mcrt, Web3Network::Solana),
		// Btc
		(Web3TokenType::Btc, Web3Network::BitcoinP2tr),
		(Web3TokenType::Btc, Web3Network::BitcoinP2pkh),
		(Web3TokenType::Btc, Web3Network::BitcoinP2sh),
		(Web3TokenType::Btc, Web3Network::BitcoinP2wpkh),
		(Web3TokenType::Btc, Web3Network::BitcoinP2wsh),
		// Doge
		(Web3TokenType::Doge, Web3Network::Bsc),
		// Uni
		(Web3TokenType::Uni, Web3Network::Solana),
		// Dai
		(Web3TokenType::Dai, Web3Network::Solana),
		// Cro
		(Web3TokenType::Cro, Web3Network::Ethereum),
		(Web3TokenType::Cro, Web3Network::Solana),
	],
);

pub const TOKEN_DECIMALS_9: (u32, [(Web3TokenType, Web3Network); 4]) = (
	9,
	[
		// Ton
		(Web3TokenType::Ton, Web3Network::Bsc),
		(Web3TokenType::Ton, Web3Network::Ethereum),
		// Mcrt
		(Web3TokenType::Mcrt, Web3Network::Bsc),
		(Web3TokenType::Mcrt, Web3Network::Ethereum),
	],
);

pub const TOKEN_DECIMALS_18: (u32, [(Web3TokenType, Web3Network); 52]) = (
	18,
	[
		// Bnb
		(Web3TokenType::Bnb, Web3Network::Bsc),
		(Web3TokenType::Bnb, Web3Network::Ethereum),
		// Eth
		(Web3TokenType::Eth, Web3Network::Bsc),
		(Web3TokenType::Eth, Web3Network::Ethereum),
		// SpaceId
		(Web3TokenType::SpaceId, Web3Network::Bsc),
		(Web3TokenType::SpaceId, Web3Network::Ethereum),
		// Lit
		(Web3TokenType::Lit, Web3Network::Bsc),
		(Web3TokenType::Lit, Web3Network::Ethereum),
		// Usdc
		(Web3TokenType::Usdc, Web3Network::Bsc),
		// Usdt
		(Web3TokenType::Usdt, Web3Network::Bsc),
		// Crv
		(Web3TokenType::Crv, Web3Network::Ethereum),
		// Matic
		(Web3TokenType::Matic, Web3Network::Bsc),
		(Web3TokenType::Matic, Web3Network::Ethereum),
		// Dydx
		(Web3TokenType::Dydx, Web3Network::Ethereum),
		// Amp
		(Web3TokenType::Amp, Web3Network::Ethereum),
		// Cvx
		(Web3TokenType::Cvx, Web3Network::Ethereum),
		// Tusd
		(Web3TokenType::Tusd, Web3Network::Bsc),
		(Web3TokenType::Tusd, Web3Network::Ethereum),
		// Usdd
		(Web3TokenType::Usdd, Web3Network::Bsc),
		(Web3TokenType::Usdd, Web3Network::Ethereum),
		// Link
		(Web3TokenType::Link, Web3Network::Bsc),
		(Web3TokenType::Link, Web3Network::Ethereum),
		// Grt
		(Web3TokenType::Grt, Web3Network::Bsc),
		(Web3TokenType::Grt, Web3Network::Ethereum),
		// Comp
		(Web3TokenType::Comp, Web3Network::Ethereum),
		// People
		(Web3TokenType::People, Web3Network::Ethereum),
		// Gtc
		(Web3TokenType::Gtc, Web3Network::Ethereum),
		// Nfp
		(Web3TokenType::Nfp, Web3Network::Bsc),
		// Sol
		(Web3TokenType::Sol, Web3Network::Bsc),
		(Web3TokenType::Sol, Web3Network::Ethereum),
		// Ada
		(Web3TokenType::Ada, Web3Network::Bsc),
		// Shib
		(Web3TokenType::Shib, Web3Network::Ethereum),
		// Uni
		(Web3TokenType::Uni, Web3Network::Ethereum),
		(Web3TokenType::Uni, Web3Network::Bsc),
		(Web3TokenType::Uni, Web3Network::Arbitrum),
		(Web3TokenType::Uni, Web3Network::Polygon),
		// Bch
		(Web3TokenType::Bch, Web3Network::Bsc),
		// Etc
		(Web3TokenType::Etc, Web3Network::Bsc),
		// Atom
		(Web3TokenType::Atom, Web3Network::Bsc),
		// Dai
		(Web3TokenType::Dai, Web3Network::Ethereum),
		(Web3TokenType::Dai, Web3Network::Bsc),
		(Web3TokenType::Dai, Web3Network::Polygon),
		(Web3TokenType::Dai, Web3Network::Arbitrum),
		// Leo
		(Web3TokenType::Leo, Web3Network::Ethereum),
		// Fil
		(Web3TokenType::Fil, Web3Network::Bsc),
		// Imx
		(Web3TokenType::Imx, Web3Network::Ethereum),
		// Inj
		(Web3TokenType::Inj, Web3Network::Ethereum),
		(Web3TokenType::Inj, Web3Network::Bsc),
		// Bean
		(Web3TokenType::Bean, Web3Network::Bsc),
		(Web3TokenType::Bean, Web3Network::Combo),
		// An
		(Web3TokenType::An, Web3Network::Bsc),
		// Tuna
		(Web3TokenType::Tuna, Web3Network::Ethereum),
	],
);

pub struct TokenDecimalsFilter;
impl TokenDecimalsFilter {
	pub fn filter(token: Web3TokenType, network: Web3Network) -> u32 {
		let target = (token, network);

		let (decimals, data) = TOKEN_DECIMALS_18;
		if data.contains(&target) {
			return decimals;
		}

		let (decimals, data) = TOKEN_DECIMALS_8;
		if data.contains(&target) {
			return decimals;
		}

		let (decimals, data) = TOKEN_DECIMALS_9;
		if data.contains(&target) {
			return decimals;
		}

		let (decimals, data) = TOKEN_DECIMALS_6;
		if data.contains(&target) {
			return decimals;
		}

		let (decimals, data) = TOKEN_DECIMALS_2;
		if data.contains(&target) {
			return decimals;
		}

		DEFAULT_TOKEN_DECIMALS
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn filter_token_decimal_18_works() {
		let target = (Web3TokenType::Bnb, Web3Network::Bsc);
		let d = TokenDecimalsFilter::filter(target.0, target.1);
		assert_eq!(d, 18);
	}

	#[test]
	fn filter_token_decimal_8_works() {
		let target = (Web3TokenType::Mcrt, Web3Network::Solana);
		let d = TokenDecimalsFilter::filter(target.0, target.1);
		assert_eq!(d, 8);
	}

	#[test]
	fn filter_token_decimal_9_works() {
		let target = (Web3TokenType::Mcrt, Web3Network::Bsc);
		let d = TokenDecimalsFilter::filter(target.0, target.1);
		assert_eq!(d, 9);
	}

	#[test]
	fn filter_token_decimal_6_works() {
		let target = (Web3TokenType::Usdt, Web3Network::Ethereum);
		let d = TokenDecimalsFilter::filter(target.0, target.1);
		assert_eq!(d, 6);
	}

	#[test]
	fn filter_token_decimal_2_works() {
		let target = (Web3TokenType::Gusd, Web3Network::Ethereum);
		let d = TokenDecimalsFilter::filter(target.0, target.1);
		assert_eq!(d, 2);
	}

	#[test]
	fn filter_token_decimal_default_works() {
		let target = (Web3TokenType::Gusd, Web3Network::Arbitrum);
		let d = TokenDecimalsFilter::filter(target.0, target.1);
		assert_eq!(d, 1);
	}
}
