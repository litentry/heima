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
use frame_support::{
	traits::{Get, OnRuntimeUpgrade},
	StorageHasher, Twox128,
};
use sp_std::marker::PhantomData;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use crate::migration::clear_storage_prefix;
use frame_support::{migration::storage_key_iter, pallet_prelude::*, Twox64Concat};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_bounties::{Bounties, BountyIndex, BountyStatus};
use pallet_treasury::BalanceOf;
use parity_scale_codec::EncodeLike;
use sp_runtime::Saturating;
use sp_std::collections::btree_map::BTreeMap;

use crate::migration::DECIMAL_CONVERTOR;
