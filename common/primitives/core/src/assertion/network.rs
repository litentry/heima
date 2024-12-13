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

use crate::{String, Vec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::ConstU32, BoundedVec};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};

pub const MAX_WEB3NETWORK_LEN: u32 = 128;
pub type BoundedWeb3Network = BoundedVec<Web3Network, ConstU32<MAX_WEB3NETWORK_LEN>>;

/// supported web3 networks
/// use a flattened style to avoid overly nested structure like:
/// {
///   Substrate(SubstrateNetwork),
///   Evm(EvmNetwork),
/// }
///
/// TODO: theoretically this should the union of the supported networks of all data providers
///
/// Since the incorporation of Bitcoin network, the name `Web3Network` might not be the best word,
/// as different kinds of bitcoin types (BitcoinP2tr, BitcoinP2pkh, ...) still belong to the same
/// network (bitcoin mainnet) despite of having 5 entries in this enum.
///
/// More precisely, it should reflect "the way" how the same identity handle (e.g. pubkey) is
/// differently used: either in different networks (e.g. eth vs bsc), or as different addresses in
/// the same network or not (e.g. bitcoin/substrate)
#[derive(
    Encode,
    Decode,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TypeInfo,
    MaxEncodedLen,
    EnumIter,
    IntoStaticStr,
)]
pub enum Web3Network {
    // substrate
    #[codec(index = 0)]
    Polkadot,
    #[codec(index = 1)]
    Kusama,
    #[codec(index = 2)]
    Litentry,
    #[codec(index = 3)]
    // TODO: we sunset Litmus already, this entry is kept temporarily to not break anything.
    //       However, it should not be used in any of the vc building logic
    Litmus,
    #[codec(index = 4)]
    LitentryRococo,
    #[codec(index = 5)]
    Khala,
    #[codec(index = 6)]
    SubstrateTestnet,

    // evm
    #[codec(index = 7)]
    Ethereum,
    #[codec(index = 8)]
    Bsc,

    // btc, see https://github.com/rust-bitcoin/rust-bitcoin/blob/9ea3e29d61569479b7b4618c8ae1992612f3d01a/bitcoin/src/address/mod.rs#L64-L75
    #[codec(index = 9)]
    BitcoinP2tr,
    #[codec(index = 10)]
    BitcoinP2pkh,
    #[codec(index = 11)]
    BitcoinP2sh,
    #[codec(index = 12)]
    BitcoinP2wpkh,
    #[codec(index = 13)]
    BitcoinP2wsh,

    // evm
    #[codec(index = 14)]
    Polygon,
    #[codec(index = 15)]
    Arbitrum,
    // solana
    #[codec(index = 16)]
    Solana,
    // combo L2 of BSC
    #[codec(index = 17)]
    Combo,
}

// mainly used in CLI
impl TryFrom<&str> for Web3Network {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Web3Network::iter()
            .find(|n| <Self as Into<&'static str>>::into(*n).to_lowercase() == value.to_lowercase())
            .ok_or(())
    }
}

impl Web3Network {
    pub fn is_substrate(&self) -> bool {
        matches!(
            self,
            Self::Polkadot
                | Self::Kusama
                | Self::Litentry
                | Self::Litmus
                | Self::LitentryRococo
                | Self::Khala
                | Self::SubstrateTestnet
        )
    }

    pub fn is_evm(&self) -> bool {
        matches!(
            self,
            Self::Ethereum | Self::Bsc | Self::Polygon | Self::Arbitrum | Self::Combo
        )
    }

    pub fn is_bitcoin(&self) -> bool {
        matches!(
            self,
            Self::BitcoinP2tr
                | Self::BitcoinP2pkh
                | Self::BitcoinP2sh
                | Self::BitcoinP2wpkh
                | Self::BitcoinP2wsh
        )
    }

    pub fn is_solana(&self) -> bool {
        matches!(self, Self::Solana)
    }

    pub fn get_code(&self) -> u8 {
        match self {
            Web3Network::Polkadot => 0,
            Web3Network::Kusama => 1,
            Web3Network::Litentry => 2,
            Web3Network::Litmus => 3,
            Web3Network::LitentryRococo => 4,
            Web3Network::Khala => 5,
            Web3Network::SubstrateTestnet => 6,
            Web3Network::Ethereum => 7,
            Web3Network::Bsc => 8,
            Web3Network::BitcoinP2tr => 9,
            Web3Network::BitcoinP2pkh => 10,
            Web3Network::BitcoinP2sh => 11,
            Web3Network::BitcoinP2wpkh => 12,
            Web3Network::BitcoinP2wsh => 13,
            Web3Network::Polygon => 14,
            Web3Network::Arbitrum => 15,
            Web3Network::Solana => 16,
            Web3Network::Combo => 17,
        }
    }

    pub fn from_code(code: u8) -> Option<Web3Network> {
        match code {
            0 => Some(Web3Network::Polkadot),
            1 => Some(Web3Network::Kusama),
            2 => Some(Web3Network::Litentry),
            3 => Some(Web3Network::Litmus),
            4 => Some(Web3Network::LitentryRococo),
            5 => Some(Web3Network::Khala),
            6 => Some(Web3Network::SubstrateTestnet),
            7 => Some(Web3Network::Ethereum),
            8 => Some(Web3Network::Bsc),
            9 => Some(Web3Network::BitcoinP2tr),
            10 => Some(Web3Network::BitcoinP2pkh),
            11 => Some(Web3Network::BitcoinP2sh),
            12 => Some(Web3Network::BitcoinP2wpkh),
            13 => Some(Web3Network::BitcoinP2wsh),
            14 => Some(Web3Network::Polygon),
            15 => Some(Web3Network::Arbitrum),
            16 => Some(Web3Network::Solana),
            17 => Some(Web3Network::Combo),
            _ => None,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Web3Network::Polkadot => "polkadot".into(),
            Web3Network::Kusama => "kusama".into(),
            Web3Network::Litentry => "litentry".into(),
            Web3Network::Litmus => "litmus".into(),
            Web3Network::LitentryRococo => "litentry_rococo".into(),
            Web3Network::Khala => "khala".into(),
            Web3Network::SubstrateTestnet => "substrate_testnet".into(),
            Web3Network::Ethereum => "ethereum".into(),
            Web3Network::Bsc => "bsc".into(),
            Web3Network::BitcoinP2tr => "bitcoin_p2tr".into(),
            Web3Network::BitcoinP2pkh => "bitcoin_p2pkh".into(),
            Web3Network::BitcoinP2sh => "bitcoin_p2sh".into(),
            Web3Network::BitcoinP2wpkh => "bitcoin_p2wpkh".into(),
            Web3Network::BitcoinP2wsh => "bitcoin_p2wsh".into(),
            Web3Network::Polygon => "polygon".into(),
            Web3Network::Arbitrum => "arbitrum".into(),
            Web3Network::Solana => "solana".into(),
            Web3Network::Combo => "combo".into(),
        }
    }
}

pub fn all_web3networks() -> Vec<Web3Network> {
    Web3Network::iter().collect()
}

pub fn all_substrate_web3networks() -> Vec<Web3Network> {
    Web3Network::iter().filter(|n| n.is_substrate()).collect()
}

pub fn all_evm_web3networks() -> Vec<Web3Network> {
    Web3Network::iter().filter(|n| n.is_evm()).collect()
}

pub fn all_bitcoin_web3networks() -> Vec<Web3Network> {
    Web3Network::iter().filter(|n| n.is_bitcoin()).collect()
}

pub fn all_solana_web3networks() -> Vec<Web3Network> {
    Web3Network::iter().filter(|n| n.is_solana()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web3network_length_ok() {
        let networks: Vec<Web3Network> = Web3Network::iter().collect();
        assert!(networks.len() < MAX_WEB3NETWORK_LEN.try_into().unwrap());
    }

    #[test]
    fn is_evm_works() {
        Web3Network::iter().for_each(|network| {
            assert_eq!(
                network.is_evm(),
                match network {
                    Web3Network::Polkadot => false,
                    Web3Network::Kusama => false,
                    Web3Network::Litentry => false,
                    Web3Network::Litmus => false,
                    Web3Network::LitentryRococo => false,
                    Web3Network::Khala => false,
                    Web3Network::SubstrateTestnet => false,
                    Web3Network::Ethereum => true,
                    Web3Network::Bsc => true,
                    Web3Network::BitcoinP2tr => false,
                    Web3Network::BitcoinP2pkh => false,
                    Web3Network::BitcoinP2sh => false,
                    Web3Network::BitcoinP2wpkh => false,
                    Web3Network::BitcoinP2wsh => false,
                    Web3Network::Polygon => true,
                    Web3Network::Arbitrum => true,
                    Web3Network::Solana => false,
                    Web3Network::Combo => true,
                }
            )
        })
    }

    #[test]
    fn is_substrate_works() {
        Web3Network::iter().for_each(|network| {
            assert_eq!(
                network.is_substrate(),
                match network {
                    Web3Network::Polkadot => true,
                    Web3Network::Kusama => true,
                    Web3Network::Litentry => true,
                    Web3Network::Litmus => true,
                    Web3Network::LitentryRococo => true,
                    Web3Network::Khala => true,
                    Web3Network::SubstrateTestnet => true,
                    Web3Network::Ethereum => false,
                    Web3Network::Bsc => false,
                    Web3Network::BitcoinP2tr => false,
                    Web3Network::BitcoinP2pkh => false,
                    Web3Network::BitcoinP2sh => false,
                    Web3Network::BitcoinP2wpkh => false,
                    Web3Network::BitcoinP2wsh => false,
                    Web3Network::Polygon => false,
                    Web3Network::Arbitrum => false,
                    Web3Network::Solana => false,
                    Web3Network::Combo => false,
                }
            )
        })
    }

    #[test]
    fn is_bitcoin_works() {
        Web3Network::iter().for_each(|network| {
            assert_eq!(
                network.is_bitcoin(),
                match network {
                    Web3Network::Polkadot => false,
                    Web3Network::Kusama => false,
                    Web3Network::Litentry => false,
                    Web3Network::Litmus => false,
                    Web3Network::LitentryRococo => false,
                    Web3Network::Khala => false,
                    Web3Network::SubstrateTestnet => false,
                    Web3Network::Ethereum => false,
                    Web3Network::Bsc => false,
                    Web3Network::BitcoinP2tr => true,
                    Web3Network::BitcoinP2pkh => true,
                    Web3Network::BitcoinP2sh => true,
                    Web3Network::BitcoinP2wpkh => true,
                    Web3Network::BitcoinP2wsh => true,
                    Web3Network::Polygon => false,
                    Web3Network::Arbitrum => false,
                    Web3Network::Solana => false,
                    Web3Network::Combo => false,
                }
            )
        })
    }

    #[test]
    fn is_solana_works() {
        Web3Network::iter().for_each(|network| {
            assert_eq!(
                network.is_solana(),
                match network {
                    Web3Network::Polkadot => false,
                    Web3Network::Kusama => false,
                    Web3Network::Litentry => false,
                    Web3Network::Litmus => false,
                    Web3Network::LitentryRococo => false,
                    Web3Network::Khala => false,
                    Web3Network::SubstrateTestnet => false,
                    Web3Network::Ethereum => false,
                    Web3Network::Bsc => false,
                    Web3Network::BitcoinP2tr => false,
                    Web3Network::BitcoinP2pkh => false,
                    Web3Network::BitcoinP2sh => false,
                    Web3Network::BitcoinP2wpkh => false,
                    Web3Network::BitcoinP2wsh => false,
                    Web3Network::Polygon => false,
                    Web3Network::Arbitrum => false,
                    Web3Network::Solana => true,
                    Web3Network::Combo => false,
                }
            )
        })
    }

    #[test]
    fn try_from_str_works() {
        let mut n: Result<Web3Network, ()> = "polkadot".try_into();
        assert_eq!(n.unwrap(), Web3Network::Polkadot);
        n = "poLkAdOt".try_into();
        assert_eq!(n.unwrap(), Web3Network::Polkadot);
        n = "NonExist".try_into();
        assert_eq!(n, Err(()))
    }
}
