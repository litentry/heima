// Copyright 2020-2025 Trust Computing GmbH.
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

//! Multi-block migration that rescales every `pallet_vesting` schedule when the chain block time
//! is halved from 12s to 6s.
//!
//! `VestingInfo` stores `{ locked, per_block, starting_block }`. `per_block` is a *per-block* drip
//! rate and `starting_block` is an *absolute* block. Once blocks arrive twice as fast in wall-clock
//! terms, an unmigrated schedule would finish vesting in **half** the intended real time. We rescale
//! so the *remaining* lock drains over the same wall-clock duration as before:
//!
//! Anchored at the upgrade block `now`:
//! * For a schedule that has **already started** (`starting_block <= now`):
//!   - `still_locked = locked_at(now)` (funds already vested stay vested),
//!   - new schedule `{ locked: still_locked, per_block: old_per_block / 2, starting_block: now }`.
//!     At half the drip rate, `still_locked` now drains over ~2x the blocks => same wall-clock.
//! * For a schedule that has **not started yet** (`starting_block > now`):
//!   - keep `locked`, halve `per_block`, and push the start out proportionally:
//!     `starting_block' = now + 2 * (starting_block - now)`.
//!
//! `per_block` is clamped to `>= 1` so a schedule can never become non-terminating, and any
//! schedule that is already fully vested at `now` is dropped (its tokens are unlocked anyway).
//!
//! This is a `SteppedMigration` (registered via `pallet_migrations::Config::Migrations`) because the
//! `Vesting` map is large on mainnet (~2770 accounts) and a single-block pass could exceed block
//! weight.

extern crate alloc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{
	migrations::{SteppedMigration, SteppedMigrationError},
	pallet_prelude::*,
	weights::WeightMeter,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_vesting::{Vesting, VestingInfo};
use sp_runtime::{traits::Zero, Saturating};

/// Unique identifier for this migration. Bump the version byte if it ever needs to re-run.
const PALLET_MIGRATION_ID: &[u8; 18] = b"vesting-rescale-6s";

type BalanceOf<T> = <<T as pallet_vesting::Config>::Currency as frame_support::traits::Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

/// An account's full vesting-schedule list, as stored by `pallet_vesting`.
type ScheduleList<T> = BoundedVec<
	VestingInfo<BalanceOf<T>, BlockNumberFor<T>>,
	pallet_vesting::MaxVestingSchedulesGet<T>,
>;

pub struct VestingRescaleMigration<T>(PhantomData<T>);

impl<T> VestingRescaleMigration<T>
where
	T: pallet_vesting::Config,
	BalanceOf<T>:
		Saturating + Copy + Zero + PartialOrd + core::ops::Div<Output = BalanceOf<T>> + From<u32>,
{
	/// Rescale a single account's schedule list. Returns `None` when, after rescaling, the account
	/// no longer has any live schedule (everything was fully vested).
	fn rescale_account(
		now: BlockNumberFor<T>,
		schedules: ScheduleList<T>,
	) -> Option<ScheduleList<T>> {
		let mut out: Vec<VestingInfo<BalanceOf<T>, BlockNumberFor<T>>> = Vec::new();

		for s in schedules.into_iter() {
			// `locked_at` needs the runtime's `BlockNumberToBalance` converter; compute it here and
			// hand the result to the pure rescale helper so the arithmetic is independently testable.
			let still_locked =
				s.locked_at::<<T as pallet_vesting::Config>::BlockNumberToBalance>(now);
			if let Some(info) = rescale_schedule::<BalanceOf<T>, BlockNumberFor<T>>(
				now,
				s.locked(),
				s.per_block(),
				s.starting_block(),
				still_locked,
			) {
				out.push(info);
			}
		}

		if out.is_empty() {
			None
		} else {
			// The list came from a `BoundedVec` of the same bound and we never grow it, so this
			// `try_from` cannot fail.
			Some(BoundedVec::try_from(out).expect("rescaled list never exceeds the original bound"))
		}
	}
}

impl<T> SteppedMigration for VestingRescaleMigration<T>
where
	T: pallet_vesting::Config,
	BalanceOf<T>:
		Saturating + Copy + Zero + PartialOrd + core::ops::Div<Output = BalanceOf<T>> + From<u32>,
{
	type Cursor = T::AccountId;
	type Identifier = MigrationId<18>;

	fn id() -> Self::Identifier {
		MigrationId { pallet_id: *PALLET_MIGRATION_ID, version_from: 0, version_to: 1 }
	}

	fn step(
		mut cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		// One read + one write per account; require enough headroom for at least one account so the
		// migration can always make progress.
		let required = T::DbWeight::get().reads_writes(1, 1);
		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}

		let now = frame_system::Pallet::<T>::block_number();

		loop {
			if meter.try_consume(required).is_err() {
				// Out of weight for this block; resume from `cursor` next block.
				return Ok(cursor);
			}

			// Resume just past the last processed account, else start at the top of the map.
			let mut iter = match &cursor {
				Some(last) => Vesting::<T>::iter_from(Vesting::<T>::hashed_key_for(last)),
				None => Vesting::<T>::iter(),
			};

			match iter.next() {
				Some((account, schedules)) => {
					match Self::rescale_account(now, schedules) {
						Some(new_schedules) => Vesting::<T>::insert(&account, new_schedules),
						None => Vesting::<T>::remove(&account),
					}
					cursor = Some(account);
				},
				// Reached the end of the map: migration complete.
				None => return Ok(None),
			}
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		let count = Vesting::<T>::iter().count() as u32;
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		// Every remaining schedule must be valid (non-zero locked & per_block).
		for (_who, schedules) in Vesting::<T>::iter() {
			for s in schedules.into_iter() {
				frame_support::ensure!(s.is_valid(), "vesting schedule invalid after rescale");
			}
		}
		Ok(())
	}
}

/// A small, self-describing identifier so two block-time migrations can never collide.
#[derive(MaxEncodedLen, Encode, Decode)]
pub struct MigrationId<const N: usize> {
	pub pallet_id: [u8; N],
	pub version_from: u8,
	pub version_to: u8,
}

/// Pure rescale of a single vesting schedule for the 12s -> 6s block-time change.
///
/// `still_locked` must be the schedule's `locked_at(now)` (computed by the caller using the
/// runtime's `BlockNumberToBalance`). Returns the new schedule, or `None` if it is already fully
/// vested at `now` (in which case the schedule is dropped — its funds are unlocked anyway).
///
/// * Already started (`starting_block <= now`): keep already-vested funds vested; the remaining
///   `still_locked` drains from `now` at half the old rate, i.e. over ~2x the blocks => same
///   wall-clock.
/// * Not started yet (`starting_block > now`): keep `locked`, halve the rate, and push the start
///   out proportionally so it still begins at the same wall-clock moment.
///
/// `per_block` is clamped to `>= 1` so the schedule can never become non-terminating.
fn rescale_schedule<Balance, BlockNumber>(
	now: BlockNumber,
	locked: Balance,
	per_block: Balance,
	starting_block: BlockNumber,
	still_locked: Balance,
) -> Option<VestingInfo<Balance, BlockNumber>>
where
	Balance: sp_runtime::traits::AtLeast32BitUnsigned + Copy,
	BlockNumber: sp_runtime::traits::AtLeast32BitUnsigned + Copy + sp_runtime::traits::Bounded,
{
	let halved_rate = per_block / Balance::from(2u32);
	let new_per_block = if halved_rate.is_zero() { Balance::from(1u32) } else { halved_rate };

	let info = if starting_block > now {
		// Not started yet: delay the start proportionally, keep the full locked amount.
		let delay = starting_block.saturating_sub(now);
		let new_start = now.saturating_add(delay.saturating_mul(2u32.into()));
		VestingInfo::new(locked, new_per_block, new_start)
	} else {
		// Already vesting: rebase the still-locked remainder to start draining from `now`.
		if still_locked.is_zero() {
			return None;
		}
		VestingInfo::new(still_locked, new_per_block, now)
	};

	if info.is_valid() {
		Some(info)
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::rescale_schedule;

	// Balance = u128, BlockNumber = u32 (matches the heima runtime).
	type Info = pallet_vesting::VestingInfo<u128, u32>;

	fn rescale(
		now: u32,
		locked: u128,
		per_block: u128,
		starting_block: u32,
		still_locked: u128,
	) -> Option<Info> {
		rescale_schedule::<u128, u32>(now, locked, per_block, starting_block, still_locked)
	}

	#[test]
	fn already_vesting_halves_rate_and_rebases_to_now() {
		// 1000 locked, 10/block from block 0; at now=40, 400 vested, 600 still locked.
		let out = rescale(40, 1000, 10, 0, 600).expect("still vesting");
		assert_eq!(out.locked(), 600, "only the still-locked remainder carries over");
		assert_eq!(out.per_block(), 5, "drip rate halved");
		assert_eq!(out.starting_block(), 40, "rebased to the upgrade block");
		// Same wall-clock: old remaining 600/10 = 60 blocks @12s; new 600/5 = 120 blocks @6s.
	}

	#[test]
	fn not_started_delays_start_proportionally() {
		// Starts at block 100, now is 40 => 60 blocks away. After: 40 + 2*60 = 160.
		let out = rescale(40, 1000, 10, 100, 1000).expect("not fully vested");
		assert_eq!(out.locked(), 1000, "full amount preserved before start");
		assert_eq!(out.per_block(), 5, "drip rate halved");
		assert_eq!(out.starting_block(), 160, "start pushed out so wall-clock start is unchanged");
	}

	#[test]
	fn fully_vested_is_dropped() {
		// now past the end; nothing still locked.
		assert!(rescale(1000, 1000, 10, 0, 0).is_none());
	}

	#[test]
	fn per_block_clamped_to_at_least_one() {
		// per_block = 1 halves to 0 -> clamped to 1 so the schedule still terminates.
		let out = rescale(0, 100, 1, 0, 100).expect("still vesting");
		assert_eq!(out.per_block(), 1, "never drops below 1");
		assert!(out.is_valid());
	}
}
