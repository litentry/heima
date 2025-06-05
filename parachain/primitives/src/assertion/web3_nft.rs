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
pub enum Web3NftType {
	#[codec(index = 0)]
	WeirdoGhostGang,
	#[codec(index = 1)]
	Club3Sbt,
	#[codec(index = 2)]
	MFan,
	#[codec(index = 3)]
	Mvp,
}

impl Web3NftType {
	pub fn get_supported_networks(&self) -> Vec<Web3Network> {
		match self {
			Self::WeirdoGhostGang | Self::Mvp => vec![Web3Network::Ethereum],
			Self::Club3Sbt => vec![Web3Network::Bsc, Web3Network::Polygon, Web3Network::Arbitrum],
			Self::MFan => vec![Web3Network::Polygon],
		}
	}
}
