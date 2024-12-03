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
#![allow(clippy::identity_op)]

mod error;
pub use error::*;

mod vc;
pub use vc::*;

pub mod teebag;
pub use teebag::*;

pub mod assertion;
pub use assertion::Assertion;

pub mod identity;
pub use identity::*;

pub mod intent;
pub use intent::*;

pub mod omni_account;
pub use omni_account::*;

extern crate alloc;
extern crate core;
use alloc::{format, str, str::FromStr, string::String, vec, vec::Vec};
use sp_runtime::{traits::ConstU32, BoundedVec};

pub use constants::*;
pub use litentry_proc_macros::*;
pub use opaque::*;
pub use types::*;

pub type ParameterString = BoundedVec<u8, ConstU32<64>>;

/// Common types of parachains.
mod types {
    use sp_core::H256;
    use sp_runtime::{
        traits::{IdentifyAccount, Verify},
        MultiSignature,
    };

    /// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
    pub type Signature = MultiSignature;

    /// Some way of identifying an account on the chain. We intentionally make it equivalent
    /// to the public key of our transaction signing scheme.
    pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

    /// Signed version of Balance
    pub type Amount = i128;

    /// AssetId type
    pub type AssetId = u128;

    /// Balance of an account.
    pub type Balance = u128;

    /// Index of a transaction in the chain.
    pub type Nonce = u32;

    /// A hash of some data used by the chain.
    pub type Hash = H256;

    /// An index to a block.
    pub type BlockNumber = u32;

    // Weight Type for XCM
    pub type Weight = frame_support::weights::Weight;

    // Aura consensus authority.
    pub type AuraId = sp_consensus_aura::sr25519::AuthorityId;
}

/// Common constants of parachains.
mod constants {
    use super::types::BlockNumber;

    pub const LITENTRY_PARA_ID: u32 = 2013;
    pub const PASEO_PARA_ID: u32 = 2106;

    /// This determines the average expected block time that we are targeting.
    /// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
    /// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
    /// up by `pallet_aura` to implement `fn slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: u64 = 12000;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
    // 365.25 DAYS = 8766 HOURS
    pub const YEARS: BlockNumber = HOURS * 8766;
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    use sp_runtime::{generic, traits::BlakeTwo256};

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}
