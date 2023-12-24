// Copyright 2020-2023 Trust Computing GmbH.
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

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum EVMTokenType {
	#[codec(index = 0)]
	Ton,
	#[codec(index = 1)]
	Trx,
	#[codec(index = 2)]
	Pepe,
	#[codec(index = 3)]
	Shib,
	#[codec(index = 4)]
	Uni,
	#[codec(index = 5)]
	Bnb,
	#[codec(index = 6)]
	Link,
	#[codec(index = 7)]
	Blur,
	#[codec(index = 8)]
	Arb,
	#[codec(index = 9)]
	Bat,
	#[codec(index = 10)]
	Inj,
	#[codec(index = 11)]
	Aave,
	#[codec(index = 12)]
	Wld,
	#[codec(index = 13)]
	Ftt,
	#[codec(index = 14)]
	Cake,
	#[codec(index = 15)]
	Lit,
	#[codec(index = 16)]
	Eth,
}
