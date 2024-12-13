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

/// Traits for pallet-halving-mint
use frame_support::pallet_prelude::Weight;

pub trait OnTokenMinted<AssetId, AccountId, Balance> {
	fn token_minted(asset_id: AssetId, beneficiary: AccountId, amount: Balance) -> Weight;
}

impl<AssetId, AccountId, Balance> OnTokenMinted<AssetId, AccountId, Balance> for () {
	fn token_minted(_asset_id: AssetId, _beneficiary: AccountId, _amount: Balance) -> Weight {
		Weight::zero()
	}
}
