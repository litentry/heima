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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use frame_support::{
	pallet_prelude::*,
	traits::{
		fungibles::{Balanced, Create, Mutate, Refund},
		tokens::{Balance, Fortitude, Precision, Preservation},
		AccountTouch,
	},
	PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use sp_runtime::traits::AccountIdConversion;
use sp_std::{prelude::*, vec};

pub use pallet::*;

const MODULE_ID: PalletId = PalletId(*b"hm/ombrg");
const DEFAULT_RELAYER_THRESHOLD: u32 = 1;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ForeignChain {
	Ethereum(u32), // chain id
}

// We assume "chain + token_symbol" can uniquely identify a foreign asset
pub type ForeignAsset = (ForeignChain, Vec<u8>);
pub type Nonce = u64;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PayInRequest<AssetKind, Balance> {
	pub asset: AssetKind,
	pub dest_chain: ForeignChain,
	pub dest_address: Vec<u8>,
	pub amount: Balance,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PayOutRequest<AccountId, AssetKind, Balance> {
	pub to: AccountId,
	pub asset: AssetKind,
	pub amount: Balance,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PayOutProposal<AccountId, AssetKind, Balance> {
	pub req: PayOutRequest<AccountId, AssetKind, Balance>,
	pub ayes: Vec<AccountId>,
	pub nays: Vec<AccountId>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Treasury account to receive the bridging fee
		type TreasuryAccount: Get<Self::AccountId>;

		/// The scalar type of balance of some asset
		type Balance: Balance;

		type AssetKind: Parameter + MaxEncodedLen;

		type Assets: Create<Self::AccountId, AssetId = Self::AssetKind, Balance = Self::Balance>
			+ Mutate<Self::AccountId>
			+ AccountTouch<Self::AssetKind, Self::AccountId, Balance = Self::Balance>
			+ Balanced<Self::AccountId>
			+ Refund<Self::AccountId, AssetId = Self::AssetKind>;

		// origin to manage Relayer Admin
		type SetAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn relayer)]
	pub type Relayer<T: Config> = CountedStorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

	#[pallet::type_value]
	pub fn DefaultRelayerThresholdValue() -> u32 {
		DEFAULT_RELAYER_THRESHOLD
	}

	#[pallet::storage]
	#[pallet::getter(fn relayer_threshold)]
	pub type RelayerThreshold<T> = StorageValue<_, u32, ValueQuery, DefaultRelayerThresholdValue>;

	// A map from AssetKind to its metadata
	// It's a simplified version of the standard asset metadata
	//
	// TODO: shall we have an assset registry to store this?
	#[pallet::storage]
	#[pallet::getter(fn asset_symbol)]
	pub type AssetSymbol<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetKind, Vec<u8>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn supported_pay_in_pair)]
	pub type SupportedPayInPair<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetKind, Vec<ForeignAsset>, ValueQuery>;

	// TODO: later we can allow pay the fee with other assets
	#[pallet::storage]
	#[pallet::getter(fn pay_in_fee)]
	pub type PayInFee<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AssetKind,
		Blake2_128Concat,
		ForeignChain,
		T::Balance,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pay_in_nonce)]
	pub type PayInNonce<T: Config> = StorageMap<_, Twox64Concat, ForeignChain, Nonce, ValueQuery>;

	/// Per-relayer nonce for a given `ForeignChain`.
	///
	/// Payout request with smaller or equal nonce has been processed by this specific relayer already
	/// and thus should be ignored by this relayer.
	///
	/// This nonce mechanism relies on the monotonically increased nonce submitted from relayers, so
	/// relayers must submit payout requests in the same order as foreign chain emits the requests.
	/// Out of order may cause loss of payout requests.
	#[pallet::storage]
	#[pallet::getter(fn relayer_pay_out_nonce)]
	pub type RelayerPayOutNonce<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		ForeignChain,
		Nonce,
		ValueQuery,
	>;

	/// Finalized (global) nonce for a given `ForeignChain`.
	///
	/// Payout request with smaller or equal nonce has been finalized already and thus should be ignored
	/// by all relayers.
	#[pallet::storage]
	#[pallet::getter(fn finalized_pay_out_nonce)]
	pub type FinalizedPayOutNonce<T: Config> =
		StorageMap<_, Twox64Concat, ForeignChain, Nonce, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pay_out_proposals)]
	pub type PayOutProposals<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ForeignChain,
		Twox64Concat,
		Nonce,
		PayOutProposal<T::AccountId, T::AssetKind, T::Balance>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Admins was set
		AdminSet { new_admin: Option<T::AccountId> },
		/// Relayer added
		RelayerAdded { relayer: T::AccountId },
		/// Relayer removed
		RelayerRemoved { relayer: T::AccountId },
		/// Relayer threshold set
		RelayerThresholdSet { threshold: u32 },
		/// Some asset symbl is set
		AssetSymbolSet { asset: T::AssetKind, symbol: Vec<u8> },
		/// PayIn fee is set
		PayInFeeSet { asset: T::AssetKind, dest_chain: ForeignChain, fee: T::Balance },
		/// Account paid in tokens, they will be paid out on the other side of the bridge.
		PaidIn {
			from: T::AccountId,
			nonce: Nonce,
			asset: T::AssetKind,
			dest_asset: ForeignAsset,
			dest_address: Vec<u8>,
			amount: T::Balance,
		},
		/// Some payout request is proposed and awaits other relayers' votes
		PayOutProposed {
			who: T::AccountId, // relayer
			aye: bool,
			to: T::AccountId,
			nonce: Nonce,
			asset: T::AssetKind,
			source_asset: ForeignAsset, // to track bridging same type of token from different chains
			source_address: Vec<u8>,    // TODO: tracking purpose - might not be necessary
			amount: T::Balance,
		},
		/// Some payout request is rejected
		PayOutRejected {
			to: T::AccountId,
			nonce: Nonce,
			asset: T::AssetKind,
			source_asset: ForeignAsset, // to track bridging same type of token from different chains
			source_address: Vec<u8>,    // TODO: tracking purpose - might not be necessary
			amount: T::Balance,
		},
		/// Some payout request is successfully executed
		PaidOut {
			to: T::AccountId,
			nonce: Nonce,
			asset: T::AssetKind,
			source_asset: ForeignAsset, // to track bridging same type of token from different chains
			source_address: Vec<u8>,    // TODO: tracking purpose - might not be necessary
			amount: T::Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		RequireAdminOrRoot,
		RequireRelayer,
		ThresholdInvalid,
		AssetSymbolNotExist,
		AssetSymbolInvalid,
		PayInNonceOverflow,
		PayInPairNotSupported,
		PayInFeeNotSet,
		PayInAmountTooLow,
		PayOutNonceAlreadyProcessedByRelayer,
		PayOutNonceAlreadyFinalized,
		PayOutRequestMismatch,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub admin: Option<T::AccountId>,
		pub default_relayers: Vec<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { admin: None, default_relayers: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if let Some(ref admin) = self.admin {
				Admin::<T>::put(admin);
			}
			for r in &self.default_relayers {
				Relayer::<T>::insert(r, ());
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn set_admin(
			origin: OriginFor<T>,
			new_admin: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::SetAdminOrigin::ensure_origin(origin)?;
			Admin::<T>::put(new_admin.clone());
			Self::deposit_event(Event::AdminSet { new_admin: Some(new_admin) });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::Yes))]
		pub fn pay_in(
			origin: OriginFor<T>,
			req: PayInRequest<T::AssetKind, T::Balance>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let nonce = Self::next_pay_in_nonce(&req.dest_chain)?;
			let symbol =
				AssetSymbol::<T>::get(&req.asset).ok_or(Error::<T>::AssetSymbolNotExist)?;
			let foreign_asset: ForeignAsset = (req.dest_chain.clone(), symbol);
			ensure!(
				SupportedPayInPair::<T>::get(req.asset.clone()).contains(&foreign_asset),
				Error::<T>::PayInPairNotSupported
			);
			let fee = PayInFee::<T>::get(req.asset.clone(), req.dest_chain)
				.ok_or(Error::<T>::PayInFeeNotSet)?;
			let burn_amount = T::Assets::burn_from(
				req.asset.clone(),
				&who,
				req.amount,
				Preservation::Expendable,
				Precision::Exact,
				Fortitude::Polite,
			)?;
			ensure!(burn_amount > fee, Error::<T>::PayInAmountTooLow);

			// TODO: we could define a `OnChargeFee` trait and outsource the fee charging to runtime
			T::Assets::mint_into(req.asset.clone(), &T::TreasuryAccount::get(), fee)?;

			Self::deposit_event(Event::PaidIn {
				from: who,
				nonce,
				asset: req.asset,
				dest_asset: foreign_asset,
				dest_address: req.dest_address,
				amount: burn_amount - fee,
			});
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn propose_pay_out(
			origin: OriginFor<T>,
			source_asset: ForeignAsset,
			source_address: Vec<u8>,
			nonce: Nonce,
			to: T::AccountId,
			asset: T::AssetKind,
			amount: T::Balance,
			aye: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_relayer(&who), Error::<T>::RequireRelayer);
			let symbol = AssetSymbol::<T>::get(&asset).ok_or(Error::<T>::AssetSymbolNotExist)?;
			ensure!(symbol == source_asset.1, Error::<T>::AssetSymbolInvalid);
			// we can't require nonce == finalized_pay_out_nonce + 1, as a relayer could be way ahead
			// of finalized payout nonce
			ensure!(
				nonce > FinalizedPayOutNonce::<T>::get(&source_asset.0),
				Error::<T>::PayOutNonceAlreadyFinalized
			);
			ensure!(
				nonce > RelayerPayOutNonce::<T>::get(&who, &source_asset.0),
				Error::<T>::PayOutNonceAlreadyProcessedByRelayer
			);

			let threshold = Self::relayer_threshold();
			let total = Relayer::<T>::count();

			if threshold == 1 && aye {
				// update per-relayer payout nonce
				RelayerPayOutNonce::<T>::insert(&who, &source_asset.0, nonce);
				// immediately do the payout
				Self::do_pay_out(source_asset, source_address, nonce, to, asset, amount)?;
			} else {
				let req = PayOutRequest { to: to.clone(), asset: asset.clone(), amount };
				let mut prop = match PayOutProposals::<T>::get(&source_asset.0, nonce) {
					Some(p) => p,
					None => PayOutProposal { req: req.clone(), ayes: vec![], nays: vec![] },
				};

				// TODO: what if the faulty relayer creates the entry first?
				// righteous relayers won't have a chance to fix it
				ensure!(req == prop.req, Error::<T>::PayOutRequestMismatch);

				// TODO: what if this relayer voted already?
				if aye {
					prop.ayes.push(who.clone());
				} else {
					prop.nays.push(who.clone());
				}
				// update per-relayer payout nonce
				RelayerPayOutNonce::<T>::insert(&who, &source_asset.0, nonce);

				// Try to finalize
				if prop.ayes.len() >= threshold as usize {
					Self::do_pay_out(source_asset, source_address, nonce, to, asset, amount)?;
				} else if total >= threshold && prop.nays.len() as u32 + threshold > total {
					Self::deposit_event(Event::PayOutRejected {
						to,
						nonce,
						asset,
						source_asset,
						source_address,
						amount,
					});
				} else {
					Self::deposit_event(Event::PayOutProposed {
						who: who.clone(),
						aye,
						to,
						nonce,
						asset,
						source_asset,
						source_address,
						amount,
					});
				}
			}

			Ok(Pays::No.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn add_relayer(
			origin: OriginFor<T>,
			relayer: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			Relayer::<T>::insert(relayer.clone(), ());
			Self::deposit_event(Event::RelayerAdded { relayer });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn remove_relayer(
			origin: OriginFor<T>,
			relayer: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(Self::is_relayer(&relayer), Error::<T>::RequireRelayer);
			Relayer::<T>::remove(relayer.clone());
			Self::deposit_event(Event::RelayerRemoved { relayer });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn set_pay_in_fee(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			dest_chain: ForeignChain,
			fee: T::Balance,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			PayInFee::<T>::insert(asset.clone(), dest_chain.clone(), fee);
			Self::deposit_event(Event::PayInFeeSet { asset, dest_chain, fee });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn create_asset(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			is_sufficient: bool,
			min_balance: T::Balance,
			symbol: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin.clone())?;
			T::Assets::create(asset.clone(), Self::account_id(), is_sufficient, min_balance)?;
			Self::set_asset_symbol(origin, asset, symbol)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn set_asset_symbol(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			symbol: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			AssetSymbol::<T>::insert(&asset, symbol.clone());
			Self::deposit_event(Event::AssetSymbolSet { asset, symbol });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(8)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn set_relayer_threshold(
			origin: OriginFor<T>,
			threshold: u32,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(threshold > 0, Error::<T>::ThresholdInvalid);
			RelayerThreshold::<T>::put(threshold);
			Self::deposit_event(Event::RelayerThresholdSet { threshold });
			Ok(Pays::No.into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn ensure_admin_or_root(origin: OriginFor<T>) -> DispatchResult {
			ensure!(
				ensure_root(origin.clone()).is_ok()
					|| Some(ensure_signed(origin)?) == Self::admin(),
				Error::<T>::RequireAdminOrRoot
			);
			Ok(())
		}

		/// The derived AccountId for the pallet
		pub fn account_id() -> T::AccountId {
			MODULE_ID.into_account_truncating()
		}

		pub fn is_relayer(who: &T::AccountId) -> bool {
			Relayer::<T>::contains_key(who)
		}

		fn next_pay_in_nonce(chain: &ForeignChain) -> Result<Nonce, Error<T>> {
			let nonce = Self::pay_in_nonce(chain);
			let nonce = nonce.checked_add(1).ok_or(Error::<T>::PayInNonceOverflow)?;
			PayInNonce::<T>::insert(chain, nonce);
			Ok(nonce)
		}

		fn do_pay_out(
			source_asset: ForeignAsset,
			source_address: Vec<u8>,
			nonce: Nonce,
			to: T::AccountId,
			asset: T::AssetKind,
			amount: T::Balance,
		) -> DispatchResult {
			// update finalized payout nonce
			FinalizedPayOutNonce::<T>::insert(&source_asset.0, nonce);
			// do actual payout
			T::Assets::mint_into(asset.clone(), &to, amount)?;
			// remove it from proposal pool if exists
			if PayOutProposals::<T>::get(&source_asset.0, nonce).is_some() {
				PayOutProposals::<T>::remove(&source_asset.0, nonce)
			}
			Self::deposit_event(Event::PaidOut {
				to,
				nonce,
				asset,
				source_asset,
				source_address,
				amount,
			});
			Ok(())
		}
	}
}
