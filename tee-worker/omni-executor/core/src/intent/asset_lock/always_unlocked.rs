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

// Manages whether any asset is locked for particular account or not.

use super::{AmountType, AssetsLock};
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct AlwaysUnlockedAssetsLock {}

impl AssetsLock for AlwaysUnlockedAssetsLock {
	fn with_lock(_amount_to_lock: AmountType, _available_amount: AmountType) -> Result<Self, ()> {
		Ok(Self {})
	}

	fn lock(
		&mut self,
		_amount_to_lock: AmountType,
		_available_amount: AmountType,
	) -> Result<(), ()> {
		Ok(())
	}

	fn release(&mut self, _amount_to_release: AmountType) -> Result<(), ()> {
		Ok(())
	}

	fn get(&self) -> AmountType {
		AmountType::from(0)
	}
}
