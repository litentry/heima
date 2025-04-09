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
use sp_runtime::RuntimeDebug;

use crate::{EthereumToken, SolanaToken};

// TODO: maybe using xcm Location is better
//       but we'd need enums for all foreign types, or use GeneralIndex
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ChainType {
    Heima,         // this chain
    Ethereum(u32), // with chain id
    Solana,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Hash, MaxEncodedLen)]
pub enum ChainAsset {
    // TODO: Revisit renaming Ethereum to Evm
    Ethereum(u32, EthereumToken), // with chain id
    Solana(SolanaToken),
}

impl ChainAsset {
    pub fn is_same_chain(&self, other: &Self) -> bool {
        match (self, other) {
            (ChainAsset::Ethereum(id1, _), ChainAsset::Ethereum(id2, _)) => id1 == id2,
            (ChainAsset::Solana(_), ChainAsset::Solana(_)) => true,
            _ => false,
        }
    }
}
