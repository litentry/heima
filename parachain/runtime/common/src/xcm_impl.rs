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

use frame_support::traits::{tokens::fungibles::Mutate, ContainsPair, Get, PalletInfoAccess};
use pallet_balances::pallet::Pallet as RuntimeBalances;
use parachain_info::pallet::Pallet as ParachainInfo;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::{Convert, MaybeEquivalence, Zero};
use sp_std::{boxed::Box, cmp::Ordering, marker::PhantomData, prelude::*, sync::Arc};
use xcm::{
	latest::{
		prelude::{
			Asset as MultiAsset, Fungibility, Junction, Junctions, Location as MultiLocation,
			XcmError,
		},
		Weight, XcmContext,
	},
	v4::{AssetId as xcmAssetId, Junction::*, Junctions::*}
};
use xcm_builder::TakeRevenue;
use xcm_executor::traits::{ConvertLocation, MatchesFungibles, WeightTrader};

use crate::{BaseRuntimeRequirements, ParaRuntimeRequirements};
use core_primitives::{AccountId, AssetId};
use pallet_asset_manager::{AssetTypeGetter, Pallet as AssetManager, UnitsToWeightRatio};

use super::WEIGHT_REF_TIME_PER_SECOND;

// We need to know how to charge for incoming assets
// This takes the first fungible asset, and takes whatever UnitPerSecondGetter establishes
// UnitsToWeightRatio trait, which needs to be implemented by AssetIdInfoGetter
pub struct FirstAssetTrader<
	AssetType: From<MultiLocation> + Clone,
	AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
	R: TakeRevenue,
>(u64, Option<(MultiLocation, u128, u128)>, PhantomData<(AssetType, AssetIdInfoGetter, R)>);
impl<
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
		R: TakeRevenue,
	> WeightTrader for FirstAssetTrader<AssetType, AssetIdInfoGetter, R>
{
	fn new() -> Self {
		FirstAssetTrader(0, None, PhantomData)
	}
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::AssetsInHolding,
		_context: &XcmContext,
	) -> Result<xcm_executor::AssetsInHolding, XcmError> {
		let first_asset = payment.fungible_assets_iter().next().ok_or(XcmError::TooExpensive)?;

		// We are only going to check first asset for now. This should be sufficient for simple
		// token transfers. We will see later if we change this.
		match (first_asset.id, first_asset.fun) {
			(xcmAssetId(id), Fungibility::Fungible(_)) => {
				let asset_type: AssetType = id.clone().into();
				// Shortcut if we know the asset is not supported
				// This involves the same db read per block, mitigating any attack based on
				// non-supported assets
				if !AssetIdInfoGetter::payment_is_supported(asset_type.clone()) {
					return Err(XcmError::TooExpensive);
				}
				if let Some(units_per_second) = AssetIdInfoGetter::get_units_per_second(asset_type)
				{
					let amount = units_per_second.saturating_mul(weight.ref_time() as u128)
						/ (WEIGHT_REF_TIME_PER_SECOND as u128);

					// We dont need to proceed if the amount is 0
					// For cases (specially tests) where the asset is very cheap with respect
					// to the weight needed
					if amount.is_zero() {
						return Ok(payment);
					}

					let required = MultiAsset {
						fun: Fungibility::Fungible(amount),
						id: xcmAssetId(id.clone()),
					};
					let unused =
						payment.checked_sub(required).map_err(|_| XcmError::TooExpensive)?;
					self.0 = self.0.saturating_add(weight.ref_time() as u64);

					// In case the asset matches the one the trader already stored before, add
					// to later refund

					// Else we are always going to substract the weight if we can, but we latter do
					// not refund it

					// In short, we only refund on the asset the trader first succesfully was able
					// to pay for an execution
					let new_asset = match self.1.clone() {
						Some((prev_id, prev_amount, units_per_second)) => {
							if prev_id == id {
								Some((id, prev_amount.saturating_add(amount), units_per_second))
							} else {
								None
							}
						},
						None => Some((id, amount, units_per_second)),
					};

					// Due to the trait bound, we can only refund one asset.
					if let Some(new_asset) = new_asset {
						self.0 = self.0.saturating_add(weight.ref_time() as u64);
						self.1 = Some(new_asset);
					};
					Ok(unused)
				} else {
					Err(XcmError::TooExpensive)
				}
			},
			_ => Err(XcmError::TooExpensive),
		}
	}

	fn refund_weight(&mut self, weight: Weight, _context: &XcmContext) -> Option<MultiAsset> {
		if let Some((id, prev_amount, units_per_second)) = self.1.clone() {
			let ref_time = weight.ref_time().min(self.0);
			self.0 -= ref_time;
			let amount =
				units_per_second * (ref_time as u128) / (WEIGHT_REF_TIME_PER_SECOND as u128);
			self.1 = Some((id.clone(), prev_amount.saturating_sub(amount), units_per_second));
			Some(MultiAsset { fun: Fungibility::Fungible(amount), id: xcmAssetId(id) })
		} else {
			None
		}
	}
}

/// Deal with spent fees, deposit them as dictated by R
impl<
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
		R: TakeRevenue,
	> Drop for FirstAssetTrader<AssetType, AssetIdInfoGetter, R>
{
	fn drop(&mut self) {
		if let Some((id, amount, _)) = self.1.clone() {
			R::take_revenue((id, amount).into());
		}
	}
}

/// XCM fee depositor to which we implement the TakeRevenue trait
/// It receives a fungibles::Mutate implemented argument, a matcher to convert MultiAsset into
/// AssetId and amount, and the fee receiver account
pub struct XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>(
	PhantomData<(Assets, Matcher, AccountId, ReceiverAccount)>,
);
impl<
		Assets: Mutate<AccountId>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		AccountId: Clone + std::cmp::Eq,
		ReceiverAccount: Get<AccountId>,
	> TakeRevenue for XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>
{
	fn take_revenue(revenue: MultiAsset) {
		match Matcher::matches_fungibles(&revenue) {
			Ok((asset_id, amount)) => {
				if !amount.is_zero() {
					let ok = Assets::mint_into(asset_id, &ReceiverAccount::get(), amount).is_ok();
					debug_assert!(ok, "`mint_into` cannot generally fail; qed");
				}
			},
			Err(_) => log::debug!(
				target: "xcm",
				"take revenue failed matching fungible"
			),
		}
	}
}

pub trait Reserve {
	/// Returns assets reserve location.
	fn reserve(&self) -> Option<MultiLocation>;
}

// Takes the chain part of a MultiAsset
impl Reserve for MultiAsset {
	fn reserve(&self) -> Option<MultiLocation> {
		let xcmAssetId(location) = self.id.clone();
		let first_interior = location.first_interior();
		let parents = location.parent_count();
		match (parents, first_interior) {
			// The only case for non-relay chain will be the chain itself.
			(0, Some(Parachain(id))) => Some(MultiLocation::new(0, X1(Arc::new([Parachain(*id)])))),
			// Only Sibling parachain is recognized.
			(1, Some(Parachain(id))) => Some(MultiLocation::new(1, X1(Arc::new([Parachain(*id)])))),
			// The Relay chain.
			(1, _) => Some(MultiLocation::parent()),
			// No other case is allowed for now.
			_ => None,
		}
	}
}

/// A `FilterAssetLocation` implementation. Filters multi native assets whose
/// reserve is same with `origin`.
pub struct MultiNativeAsset;

impl ContainsPair<MultiAsset, MultiLocation> for MultiNativeAsset {
	fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		if let Some(ref reserve) = asset.reserve() {
			if reserve == origin {
				return true;
			}
		}
		false
	}
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId4Compare {
	#[codec(index = 0)]
	SelfReserve,
	#[codec(index = 1)]
	ParachainReserve(Box<MultiLocation>),
}

// Our currencyId. We distinguish for now between SelfReserve, and Others, defined by their Id.
#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub enum CurrencyId<R: BaseRuntimeRequirements> {
	// The only parachain native token: LIT
	SelfReserve(PhantomData<R>),

	// Any parachain based asset, including local native minted ones.
	ParachainReserve(Box<MultiLocation>),
}

fn convert_currency<R: BaseRuntimeRequirements>(s: &CurrencyId<R>) -> CurrencyId4Compare {
	match s {
		CurrencyId::<R>::SelfReserve(_) => CurrencyId4Compare::SelfReserve,
		CurrencyId::<R>::ParachainReserve(multi) => {
			CurrencyId4Compare::ParachainReserve(multi.clone())
		},
	}
}

impl<R: BaseRuntimeRequirements> Ord for CurrencyId<R> {
	fn cmp(&self, other: &Self) -> Ordering {
		convert_currency(self).cmp(&convert_currency(other))
	}
}

impl<R: BaseRuntimeRequirements> PartialOrd for CurrencyId<R> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<R: BaseRuntimeRequirements> Default for CurrencyId<R> {
	fn default() -> Self {
		CurrencyId::ParachainReserve(Box::new(MultiLocation::here()))
	}
}

/// Instructs how to convert a 32 byte accountId into a MultiLocation
pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		MultiLocation {
			parents: 0,
			interior: X1(Arc::new([Junction::AccountId32 { network: None, id: account.into() }])),
		}
	}
}

pub struct OldAnchoringSelfReserve<R>(PhantomData<R>);
impl<R: BaseRuntimeRequirements> OldAnchoringSelfReserve<R> {
	/// Returns the value of this parameter type.
	pub fn get() -> MultiLocation {
		MultiLocation {
			parents: 1,
			interior: Junctions::X2(Arc::new([
				Parachain(ParachainInfo::<R>::parachain_id().into()),
				Junction::PalletInstance(<RuntimeBalances<R> as PalletInfoAccess>::index() as u8),
			])),
		}
	}
}

impl<I: From<MultiLocation>, R: BaseRuntimeRequirements> Get<I> for OldAnchoringSelfReserve<R> {
	fn get() -> I {
		I::from(Self::get())
	}
}

pub struct NewAnchoringSelfReserve<R>(PhantomData<R>);

impl<R: BaseRuntimeRequirements> NewAnchoringSelfReserve<R> {
	/// Returns the value of this parameter type.
	pub fn get() -> MultiLocation {
		MultiLocation {
			parents: 0,
			interior: Junctions::X1(Arc::new([Junction::PalletInstance(
				<RuntimeBalances<R> as PalletInfoAccess>::index() as u8,
			)])),
		}
	}
}

impl<I: From<MultiLocation>, R: BaseRuntimeRequirements> Get<I> for NewAnchoringSelfReserve<R> {
	fn get() -> I {
		I::from(Self::get())
	}
}

impl<R: BaseRuntimeRequirements> From<MultiLocation> for CurrencyId<R> {
	fn from(location: MultiLocation) -> Self {
		match location {
			a if (a == (OldAnchoringSelfReserve::<R>::get()))
				| (a == (NewAnchoringSelfReserve::<R>::get())) =>
			{
				CurrencyId::<R>::SelfReserve(PhantomData)
			},
			_ => CurrencyId::<R>::ParachainReserve(Box::new(location)),
		}
	}
}

impl<R: BaseRuntimeRequirements> From<Option<MultiLocation>> for CurrencyId<R> {
	fn from(location: Option<MultiLocation>) -> Self {
		match location {
			Some(multi) => Self::from(multi),
			None => CurrencyId::ParachainReserve(Box::default()),
		}
	}
}

impl<R: BaseRuntimeRequirements> From<CurrencyId<R>> for Option<MultiLocation> {
	fn from(currency_id: CurrencyId<R>) -> Self {
		match currency_id {
			// For now and until Xtokens is adapted to handle 0.9.16 version we use
			// the old anchoring here
			// This is not a problem in either cases, since the view of the destination
			// chain does not change
			// TODO! change this to NewAnchoringSelfReserve once xtokens is adapted for it
			CurrencyId::<R>::SelfReserve(_) => {
				let multi: MultiLocation = OldAnchoringSelfReserve::<R>::get();
				Some(multi)
			},
			CurrencyId::<R>::ParachainReserve(multi) => Some(*multi),
		}
	}
}

// How to convert from CurrencyId to MultiLocation: for orml convert sp_runtime Convert
// trait
pub struct CurrencyIdMultiLocationConvert<R: BaseRuntimeRequirements>(PhantomData<R>);
impl<R: BaseRuntimeRequirements> Convert<CurrencyId<R>, Option<MultiLocation>>
	for CurrencyIdMultiLocationConvert<R>
{
	fn convert(currency: CurrencyId<R>) -> Option<MultiLocation> {
		currency.into()
	}
}

impl<R: BaseRuntimeRequirements> Convert<MultiLocation, Option<CurrencyId<R>>>
	for CurrencyIdMultiLocationConvert<R>
{
	fn convert(multi: MultiLocation) -> Option<CurrencyId<R>> {
		match multi {
			a if (a == OldAnchoringSelfReserve::<R>::get())
				| (a == NewAnchoringSelfReserve::<R>::get()) =>
			{
				Some(CurrencyId::<R>::SelfReserve(PhantomData))
			},
			_ => Some(CurrencyId::<R>::ParachainReserve(Box::new(multi))),
		}
	}
}

/// Converter struct implementing `AssetIdConversion` converting a numeric asset ID
/// (must be `TryFrom/TryInto<u128>`) into a MultiLocation Value and Viceversa through
/// an intermediate generic type AssetType.
/// The trait bounds enforce is that the AssetTypeGetter trait is also implemented
pub struct AssetIdMultiLocationConvert<R>(PhantomData<R>);

impl<R: ParaRuntimeRequirements> MaybeEquivalence<MultiLocation, AssetId>
	for AssetIdMultiLocationConvert<R>
where
	R: pallet_asset_manager::Config<ForeignAssetType = CurrencyId<R>>,
{
	fn convert(multi: &MultiLocation) -> Option<AssetId> {
		<Self as ConvertLocation<AssetId>>::convert_location(multi)
	}

	fn convert_back(id: &AssetId) -> Option<MultiLocation> {
		if let Some(currency_id) =
			<AssetManager<R> as AssetTypeGetter<AssetId, CurrencyId<R>>>::get_asset_type(*id)
		{
			<CurrencyIdMultiLocationConvert<R> as Convert<
			CurrencyId<R>,
			Option<MultiLocation>,
			>>::convert(currency_id)
		} else {
			None
		}
	}
}

impl<R: ParaRuntimeRequirements> ConvertLocation<AssetId> for AssetIdMultiLocationConvert<R>
where
	R: pallet_asset_manager::Config<ForeignAssetType = CurrencyId<R>>,
{
	fn convert_location(multi: &MultiLocation) -> Option<AssetId> {
		if let Some(currency_id) = <CurrencyIdMultiLocationConvert<R> as Convert<
			MultiLocation,
			Option<CurrencyId<R>>,
		>>::convert(multi.clone())
		{
			<AssetManager<R> as AssetTypeGetter<AssetId, CurrencyId<R>>>::get_asset_id(currency_id)
		} else {
			None
		}
	}
}
