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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum EVMTokenType {
	#[codec(index = 0)]
	Ton,
	#[codec(index = 1)]
	Trx,
}

impl EVMTokenType {
	pub fn token_decimals(&self) -> f64 {
		match self {
			// TON on BSC&ETH decimals are both 9
			// https://bscscan.com/token/0x76a797a59ba2c17726896976b7b3747bfd1d220f
			// https://etherscan.io/token/0x582d872a1b094fc48f5de31d3b73f2d9be47def1
			EVMTokenType::Ton => 1_000_000_000.0,

			// TRX on BSC&ETH decimals are both 6
			// https://bscscan.com/token/0xce7de646e7208a4ef112cb6ed5038fa6cc6b12e3
			// https://etherscan.io/token/0x50327c6c5a14dcade707abad2e27eb517df87ab5
			EVMTokenType::Trx => 1_000_000.0,
		}
	}
}
