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

/// Money matters.
pub mod currency {
	use crate::Balance;
	use runtime_common::currency::{DOLLARS, MILLICENTS};

	// Linear ratio of transaction fee distribution
	// It is recommended to set sum of ratio to 100, yet only decimal loss is concerned.
	pub const TREASURY_PROPORTION: u32 = 40u32;
	pub const AUTHOR_PROPORTION: u32 = 0u32;
	pub const BURNED_PROPORTION: u32 = 60u32;

	/// Function used in some fee configurations
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
	}
}
