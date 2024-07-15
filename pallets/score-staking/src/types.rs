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

use crate::{Perbill, DAYS};
use frame_support::pallet_prelude::*;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub type RoundIndex = u32;
/// we force u32 type, BlockNumberFor<T> implements `AtLeast32BitUnsigned` so it's safe
pub type RoundDuration = u32;
pub type Score = u32;

pub const DEFAULT_ROUND_INTERVAL: RoundDuration = 7 * DAYS;
pub const DEFAULT_SCORE_COEFFICIENT: Perbill = Perbill::from_percent(80);
pub const DEFAULT_STAKE_COEFFICIENT: Perbill = Perbill::from_percent(20);

/// an on/off flag
#[derive(
	Clone, Copy, Default, PartialEq, Eq, Encode, Decode, Debug, TypeInfo, Deserialize, Serialize,
)]
pub enum PoolState {
	#[default]
	#[codec(index = 0)]
	Stopped,
	#[codec(index = 1)]
	Running,
}

#[derive(
	Copy,
	Clone,
	Default,
	PartialEq,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	Deserialize,
	Serialize,
)]
pub struct RoundInfo<BlockNumber> {
	/// Current round index
	pub index: RoundIndex,
	/// The start block of the current round
	pub start_block: BlockNumber,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct RoundSetting {
	/// Interval of rounds in block number
	pub interval: RoundDuration,
	/// Coeffient applied to the score when calculating the reward proportion
	pub score_coefficient: Perbill,
	/// Coeffient applied to the stake amount when calculating the reward proportion
	pub stake_coefficient: Perbill,
}

#[derive(
	Copy,
	Clone,
	Default,
	PartialEq,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	Deserialize,
	Serialize,
)]
pub struct ScorePayment<Balance> {
	pub score: Score,
	pub unpaid: Balance,
}
