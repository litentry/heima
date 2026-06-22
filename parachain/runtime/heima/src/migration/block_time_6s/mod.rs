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

//! Migrations for the 12s -> 6s block-time change.
//!
//! Two pieces, wired into the runtime in different places:
//! * [`onepass::OnePassRescale`] — a single-pass `OnRuntimeUpgrade` for the bounded state
//!   (parachain-staking round, score-staking round config, scheduler agenda). Register it in the
//!   executive `Migrations` tuple in `lib.rs`.
//! * [`vesting::VestingRescaleMigration`] — a `SteppedMigration` (multi-block) for the large
//!   `pallet_vesting` map. Register it in `pallet_migrations::Config::Migrations` in `lib.rs`.
//!
//! Both are one-shot: remove them in the release *after* the one that ships spec_version 9262, once
//! the upgrade has been enacted and finalized on every network.

pub mod onepass;
pub mod vesting;

pub use onepass::OnePassRescale;
pub use vesting::VestingRescaleMigration;
