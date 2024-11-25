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

/// Adapter for [`Contains`] trait to match [`VersionedLocatableAsset`] type converted to the latest
/// version of itself where it's location matched by `L` and it's asset id by `A` parameter types.

use frame_support::traits::{Contains, ContainsPair};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::{traits::TryConvert, RuntimeDebug};
use xcm::{latest::{Junction, Junctions}, VersionedLocation};
use xcm_builder::LocatableAssetId;
use cumulus_primitives_core::{AssetId, Location};
use sp_std::sync::Arc;

/// Versioned locatable asset type which contains both an XCM `location` and `asset_id` to identify
/// an asset which exists on some chain.
#[derive(
	Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, scale_info::TypeInfo, MaxEncodedLen,
)]
pub enum VersionedLocatableAsset {
	#[codec(index = 3)]
	V3 { location: xcm::v3::Location, asset_id: xcm::v3::AssetId },
	#[codec(index = 4)]
	V4 { location: xcm::v4::Location, asset_id: xcm::v4::AssetId },
}

/// Converts the [`VersionedLocation`] to the [`xcm::latest::Location`].
pub struct VersionedLocationConverter;
impl TryConvert<&VersionedLocation, xcm::latest::Location> for VersionedLocationConverter {
	fn try_convert(
		location: &VersionedLocation,
	) -> Result<xcm::latest::Location, &VersionedLocation> {
		let latest = match location.clone() {
			VersionedLocation::V2(l) => {
				let v3: xcm::v3::Location = l.try_into().map_err(|_| location)?;
				v3.try_into().map_err(|_| location)?
			},
			VersionedLocation::V3(l) => l.try_into().map_err(|_| location)?,
			VersionedLocation::V4(l) => l,
		};
		Ok(latest)
	}
}

pub struct ContainsParts<C>(core::marker::PhantomData<C>);
impl<C> Contains<VersionedLocatableAsset> for ContainsParts<C>
where
	C: ContainsPair<xcm::latest::Location, xcm::latest::Location>,
{
	fn contains(asset: &VersionedLocatableAsset) -> bool {
		use VersionedLocatableAsset::*;
		let (location, asset_id) = match asset.clone() {
			V3 { location, asset_id } => match (location.try_into(), asset_id.try_into()) {
				(Ok(l), Ok(a)) => (l, a),
				_ => return false,
			},
			V4 { location, asset_id } => (location, asset_id),
		};
		C::contains(&location, &asset_id.0)
	}
}

/// Converts the [`VersionedLocatableAsset`] to the [`LocatableAssetId`].
pub struct LocatableAssetConverter;
impl TryConvert<VersionedLocatableAsset, LocatableAssetId>
	for LocatableAssetConverter
{
	fn try_convert(
		asset: VersionedLocatableAsset,
	) -> Result<LocatableAssetId, VersionedLocatableAsset> {
		match asset {
			VersionedLocatableAsset::V3 { location, asset_id } =>
				Ok(LocatableAssetId {
					location: location.try_into().map_err(|_| asset.clone())?,
					asset_id: asset_id.try_into().map_err(|_| asset.clone())?,
				}),
			VersionedLocatableAsset::V4 { location, asset_id } =>
				Ok(LocatableAssetId { location, asset_id }),
		}
	}
}

impl sp_runtime::traits::TryConvert<u32, LocatableAssetId> for LocatableAssetConverter {
    fn try_convert(value: u32) -> Result<LocatableAssetId, u32> {
		// let location = Location::new(1, Junctions::X1(Arc::new([Junction::Parachain(value)])));
        // Ok(LocatableAssetId {
        //     asset_id: AssetId::from(location.clone()),
        //     location,
        // })
		Ok(LocatableAssetId {
            asset_id: AssetId::from(Location::default()),
            location: Location::default(),
        })

    }
}

impl<'a> TryConvert<&'a sp_runtime::AccountId32, Location> for VersionedLocationConverter {
    fn try_convert(value: &'a sp_runtime::AccountId32) -> Result<Location, &'a sp_runtime::AccountId32> {
        Ok(Location::default())
    }
}
