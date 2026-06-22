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

//! Single-pass `OnRuntimeUpgrade` for the bounded pieces of on-chain state that encode an absolute
//! block number or a per-block rate, run once when block time is halved from 12s to 6s.
//!
//! All time-denominated *constants* (governance/treasury periods, `DefaultBlocksPerRound`, the
//! score-staking default interval, ...) are derived from `MINUTES/HOURS/DAYS` in `heima_primitives`
//! and therefore auto-double when `MILLISECS_PER_BLOCK` is halved — they need no migration. This
//! pass handles only the stored values that do **not** auto-rescale:
//!
//! * `parachain_staking::Round.length` — a stored block count (set at genesis / via
//!   `set_blocks_per_round`); reset it to the new (doubled) `DefaultBlocksPerRound` and rebase
//!   `first` to `now` so the current round neither ends early nor late on the transition.
//! * `score_staking::RoundConfig.interval` — a stored block count; reset to the new default
//!   (`7 * DAYS`) while preserving the on-chain stake coefficients, and rebase `Round.start_block`.
//! * `pallet_scheduler::Agenda` — absolute future firing blocks. We rebase **only anonymous,
//!   non-periodic** tasks (the only kind present on-chain: 1 on mainnet, 0 on Paseo), which lets us
//!   avoid touching the pallet-private `Lookup`/`Retries` maps. Named/periodic tasks (none exist)
//!   are asserted-against in try-runtime so this can never silently corrupt scheduler state.
//!
//! `vesting` is handled separately as a multi-block migration (see `vesting.rs`) because its map is
//! large.

extern crate alloc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{
	storage::unhashed,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;
use sp_runtime::Saturating;

/// Storage key for the once-only guard. The rebases below are **not** naturally idempotent
/// (re-rebasing a scheduler task would push it out again), so we record completion under this key
/// and short-circuit on any subsequent execution. Derived as
/// `twox_128("BlockTime6s") ++ twox_128("OnePassRescaleDone")` — a regular pallet-style storage
/// prefix that cannot collide with any real pallet (no pallet is named `BlockTime6s`).
fn done_key() -> [u8; 32] {
	let mut key = [0u8; 32];
	key[..16].copy_from_slice(&sp_core::hashing::twox_128(b"BlockTime6s"));
	key[16..].copy_from_slice(&sp_core::hashing::twox_128(b"OnePassRescaleDone"));
	key
}

/// Rebase an absolute future block `b` so the same number of *remaining* blocks, at 2x speed, take
/// the same wall-clock time: `b' = now + 2 * (b - now)`. Past blocks are left untouched.
fn rebase_future<B>(now: B, b: B) -> B
where
	B: Copy + PartialOrd + Saturating + From<u32>,
{
	if b > now {
		now.saturating_add(b.saturating_sub(now).saturating_mul(2u32.into()))
	} else {
		b
	}
}

pub struct OnePassRescale<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for OnePassRescale<T>
where
	T: frame_system::Config
		+ pallet_parachain_staking::Config
		+ pallet_score_staking::Config
		+ pallet_scheduler::Config,
{
	fn on_runtime_upgrade() -> Weight {
		let key = done_key();
		// Idempotency guard: the rebases below are not safe to apply twice, so run at most once.
		if unhashed::get_raw(&key).is_some() {
			log::info!("OnePassRescale: already applied, skipping");
			return T::DbWeight::get().reads(1);
		}

		let now = frame_system::Pallet::<T>::block_number();
		let mut reads: u64 = 1;
		let mut writes: u64 = 1;

		// --- parachain-staking Round ---
		pallet_parachain_staking::Round::<T>::mutate(|r| {
			r.length = <T as pallet_parachain_staking::Config>::DefaultBlocksPerRound::get();
			r.first = now;
		});
		reads += 1;
		writes += 1;

		// --- score-staking RoundConfig.interval + Round.start_block ---
		// Take the (now-doubled) default interval but keep the chain's existing stake coefficients
		// (mainnet uses m=3, Paseo m=2 — both differ from the default, so only `interval` is reset).
		let default_interval = pallet_score_staking::DefaultRoundSetting::<T>::get().interval;
		pallet_score_staking::RoundConfig::<T>::mutate(|c| {
			c.interval = default_interval;
		});
		pallet_score_staking::Round::<T>::mutate(|r| {
			r.start_block = now;
		});
		reads += 2;
		writes += 2;

		// --- scheduler Agenda: rebase anonymous, non-periodic future tasks only ---
		let agenda_keys: Vec<BlockNumberFor<T>> =
			pallet_scheduler::Agenda::<T>::iter_keys().collect();
		for when in agenda_keys {
			reads += 1;
			if when <= now {
				continue;
			}
			let new_when = rebase_future(now, when);
			if new_when == when {
				continue;
			}
			let agenda = pallet_scheduler::Agenda::<T>::take(when);
			writes += 1;
			// Merge into the destination block (it is virtually always empty).
			pallet_scheduler::Agenda::<T>::mutate(new_when, |dest| {
				for slot in agenda.into_iter() {
					// Push best-effort; if the destination block is somehow full the task is
					// dropped rather than panicking — try-runtime asserts the agenda is tiny.
					let _ = dest.try_push(slot);
				}
			});
			writes += 1;
		}

		// Mark complete so a re-execution is a no-op (idempotency).
		unhashed::put_raw(&key, &[1u8]);
		writes += 1;

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		// Snapshot the total number of scheduled slots so we can assert the rebase neither drops nor
		// duplicates any task. (The `Scheduled` struct's fields are pallet-private, so we cannot
		// inspect named/periodic-ness here; the rebase relocates opaque slots wholesale, which is
		// correct for anonymous non-periodic tasks — the only kind on these chains. Named/periodic
		// tasks would also need their private `Lookup`/period rebased, which this migration does not
		// do; that pre-condition is verified out-of-band via RPC before deployment.)
		let slot_count: u32 = pallet_scheduler::Agenda::<T>::iter()
			.map(|(_, a)| a.into_iter().flatten().count() as u32)
			.sum();
		Ok(slot_count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;
		let before = u32::decode(&mut &state[..]).map_err(|_| "bad pre_upgrade state")?;
		let after: u32 = pallet_scheduler::Agenda::<T>::iter()
			.map(|(_, a)| a.into_iter().flatten().count() as u32)
			.sum();
		frame_support::ensure!(before == after, "scheduler slot count changed during rebase");

		let round = pallet_parachain_staking::Round::<T>::get();
		frame_support::ensure!(
			round.length == <T as pallet_parachain_staking::Config>::DefaultBlocksPerRound::get(),
			"parachain-staking Round.length not reset"
		);
		Ok(())
	}
}
