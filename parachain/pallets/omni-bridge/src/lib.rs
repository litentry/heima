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
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::AccountIdConversion;
use sp_std::{prelude::*, vec};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const MODULE_ID: PalletId = PalletId(*b"hm/ombrg");
const DEFAULT_RELAYER_THRESHOLD: u32 = 1;

// TODO: maybe using xcm Location is better
//       but we'd need enums for all foreign types, or use GeneralIndex
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ChainType {
	Heima,         // this chain
	Ethereum(u32), // with chain id
}

// We assume "chain + asset-id" can uniquely identify an asset that can
// be bridged out/in
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ChainAsset<AssetKind> {
	pub chain: ChainType,
	pub asset: AssetKind,
}

impl<AssetKind: Encode> ChainAsset<AssetKind> {
	pub fn to_resource_id(&self) -> ResourceId {
		self.using_encoded(blake2_256)
	}
}

pub type Nonce = u64;
pub type ResourceId = [u8; 32]; // to be compatible with chainsafe's contract
pub type PayOutRequestHash = H256;

// payin request by user (from user to foreign chain) - better name?
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PayInRequest<AssetKind, Balance> {
	pub asset: AssetKind,
	pub dest_chain: ChainType,
	pub dest_account: Vec<u8>,
	pub amount: Balance,
}

// payout request by relayer (from foreign chain to user) - better name?
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PayOutRequest<AccountId, Balance> {
	pub source_chain: ChainType,
	pub nonce: Nonce,
	pub resource_id: ResourceId,
	pub dest_account: AccountId,
	pub amount: Balance,
}

impl<AccountId: Encode, Balance: Encode> PayOutRequest<AccountId, Balance> {
	pub fn hash(&self) -> PayOutRequestHash {
		self.using_encoded(blake2_256).into()
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PayOutVote<AccountId> {
	pub ayes: Vec<AccountId>,
	pub nays: Vec<AccountId>,
	pub status: VoteStatus,
}

#[derive(PartialEq, Eq, Clone, Encode, Default, Decode, RuntimeDebug, TypeInfo)]
pub enum VoteStatus {
	#[default]
	Pending,
	Passed,
	Failed,
}

impl<AccountId: PartialEq> PayOutVote<AccountId> {
	/// Try to finalize the vote, return the updated status
	pub fn try_finalize(&mut self, threshold: u32, total: u32) -> VoteStatus {
		if self.ayes.len() >= threshold as usize {
			self.status = VoteStatus::Passed;
			VoteStatus::Passed
		} else if total >= threshold && self.nays.len() as u32 + threshold > total {
			self.status = VoteStatus::Failed;
			VoteStatus::Failed
		} else {
			VoteStatus::Pending
		}
	}

	pub fn is_finalized(&self) -> bool {
		self.status == VoteStatus::Passed || self.status == VoteStatus::Failed
	}

	pub fn has_voted(&self, who: &AccountId) -> bool {
		self.ayes.contains(who) || self.nays.contains(who)
	}
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
	#[pallet::getter(fn relayers)]
	pub type Relayers<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

	#[pallet::type_value]
	pub fn DefaultRelayerThresholdValue() -> u32 {
		DEFAULT_RELAYER_THRESHOLD
	}

	#[pallet::storage]
	#[pallet::getter(fn relayer_threshold)]
	pub type RelayerThreshold<T> = StorageValue<_, u32, ValueQuery, DefaultRelayerThresholdValue>;

	#[pallet::storage]
	#[pallet::getter(fn asset_symbol)]
	pub type ResourceIds<T: Config> =
		StorageMap<_, Twox64Concat, ResourceId, ChainAsset<T::AssetKind>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pay_in_pair)]
	pub type PayInPair<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AssetKind, ChainType), (), OptionQuery>;

	// TODO: later we can allow pay the fee with other assets
	#[pallet::storage]
	#[pallet::getter(fn pay_in_fee)]
	pub type PayInFee<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AssetKind,
		Blake2_128Concat,
		ChainType,
		T::Balance,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pay_in_nonce)]
	pub type PayInNonce<T: Config> = StorageMap<_, Twox64Concat, ChainType, Nonce, ValueQuery>;

	/// Finalized nonce for a given `ChainType` - we track this to avoid storing finalized
	/// requests in the chain storage indefinitely.
	///
	/// Payout request whose nonce is smaller or equal to this nonce, should be considered finalized
	/// and can thus be removed from storage.
	///
	/// This nonce should only be pumped if **all** previous requests are finalized too.
	#[pallet::storage]
	#[pallet::getter(fn finalized_pay_out_nonce)]
	pub type FinalizedPayOutNonce<T: Config> =
		StorageMap<_, Twox64Concat, ChainType, Nonce, ValueQuery>;

	/// Finalized (highest) nonce for a vote, it should always be greater than or equal to FinalizedPayOutNonce
	/// It doesn't mean all votes with smaller nonce are finalized, it just provides an upper bound
	/// when bumping FinalizedPayOutNonce
	#[pallet::storage]
	#[pallet::getter(fn finalized_vote_nonce)]
	pub type FinalizedVoteNonce<T: Config> =
		StorageMap<_, Twox64Concat, ChainType, Nonce, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pay_out_hashes)]
	pub type PayOutHashes<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ChainType,
		Twox64Concat,
		Nonce,
		Vec<PayOutRequestHash>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pay_out_votes)]
	pub type PayOutVotes<T: Config> =
		StorageMap<_, Blake2_128Concat, PayOutRequestHash, PayOutVote<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pay_out_requests)]
	pub type PayOutRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		PayOutRequestHash,
		PayOutRequest<T::AccountId, T::Balance>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Admins was set
		AdminSet { new_admin: Option<T::AccountId> },
		/// Relayer added
		RelayerAdded { who: T::AccountId },
		/// Relayer removed
		RelayerRemoved { who: T::AccountId },
		/// Relayer threshold set
		RelayerThresholdSet { threshold: u32 },
		/// Some pay in pair is added
		PayInPairAdded { asset: T::AssetKind, dest_chain: ChainType },
		/// Some pay in pair is removed
		PayInPairRemoved { asset: T::AssetKind, dest_chain: ChainType },
		/// Some resource id is set
		ResourceIdSet { resource_id: ResourceId, chain_asset: ChainAsset<T::AssetKind> },
		/// Some resource id is removed
		ResourceIdRemoved { resource_id: ResourceId },
		/// PayIn fee is set
		PayInFeeSet { asset: T::AssetKind, dest_chain: ChainType, fee: T::Balance },
		/// The payout nonce for global finalization is updated
		FinalizedPayOutNonceUpdated { source_chain: ChainType, nonce: Nonce },
		/// The finalized vote nonce is updated
		FinalizedVoteNonceUpdated { source_chain: ChainType, nonce: Nonce },
		/// Someone paid in tokens, they will be paid out on the other side of the bridge
		/// This event together with payout events don't have nested structure to:
		/// 1. have a clearer display
		/// 2. apply any required adjustments (e.g. amount)
		PaidIn {
			source_account: T::AccountId,
			nonce: Nonce,
			asset: T::AssetKind,
			resource_id: ResourceId,
			dest_chain: ChainType,
			dest_account: Vec<u8>,
			amount: T::Balance,
		},
		/// Some payout request is voted
		PayOutVoted {
			who: T::AccountId, // relayer
			source_chain: ChainType,
			nonce: Nonce,
			asset: T::AssetKind,
			dest_account: T::AccountId,
			amount: T::Balance,
			aye: bool,
		},
		/// Some payout request is rejected
		PayOutRejected {
			source_chain: ChainType,
			nonce: Nonce,
			asset: T::AssetKind,
			dest_account: T::AccountId,
			amount: T::Balance,
		},
		/// Some payout request is successfully executed
		PaidOut {
			source_chain: ChainType,
			nonce: Nonce,
			asset: T::AssetKind,
			dest_account: T::AccountId,
			amount: T::Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		RequireAdminOrRoot,
		RequireRelayer,
		ThresholdInvalid,
		ChainTypeInvalid,
		ResourceIdNotExist,
		PayInNonceOverflow,
		PayInPairNotAllowed,
		PayInPairNotExist,
		PayInFeeNotSet,
		PayInAmountTooLow,
		PayOutNonceFinalized,
		PayOutVoteFinalized,
		PayOutVoteCommitted,
		FinalizedPayOutNonceOverflow,
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
				Relayers::<T>::insert(r, ());
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
			ensure!(
				PayInPair::<T>::get((&req.asset, &req.dest_chain)).is_some(),
				Error::<T>::PayInPairNotAllowed
			);

			// Note the resource_id is always calculated from this chain
			let resource_id =
				ChainAsset { asset: req.asset.clone(), chain: ChainType::Heima }.to_resource_id();

			let fee = PayInFee::<T>::get(&req.asset, &req.dest_chain)
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
				source_account: who,
				nonce,
				asset: req.asset,
				resource_id,
				dest_chain: req.dest_chain,
				dest_account: req.dest_account,
				amount: burn_amount - fee,
			});
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn request_pay_out(
			origin: OriginFor<T>,
			req: PayOutRequest<T::AccountId, T::Balance>,
			aye: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_relayer(&who), Error::<T>::RequireRelayer);
			ensure!(req.source_chain != ChainType::Heima, Error::<T>::ChainTypeInvalid);
			ensure!(
				req.nonce > FinalizedPayOutNonce::<T>::get(&req.source_chain),
				Error::<T>::PayOutNonceFinalized
			);

			let hash = req.hash();
			let chain_asset =
				ResourceIds::<T>::get(req.resource_id).ok_or(Error::<T>::ResourceIdNotExist)?;
			ensure!(chain_asset.chain == ChainType::Heima, Error::<T>::ChainTypeInvalid);

			// vote
			let mut vote = match PayOutVotes::<T>::get(hash) {
				Some(v) => v,
				None => PayOutVote { ayes: vec![], nays: vec![], status: VoteStatus::Pending },
			};

			ensure!(!vote.is_finalized(), Error::<T>::PayOutVoteFinalized);
			ensure!(!vote.has_voted(&who), Error::<T>::PayOutVoteCommitted);

			if aye {
				vote.ayes.push(who.clone());
			} else {
				vote.nays.push(who.clone());
			}

			Self::deposit_event(Event::PayOutVoted {
				who: who.clone(),
				source_chain: req.source_chain.clone(),
				nonce: req.nonce,
				asset: chain_asset.asset.clone(),
				dest_account: req.dest_account.clone(),
				amount: req.amount,
				aye,
			});

			// try to finalise
			let threshold = Self::relayer_threshold();
			let total = Relayers::<T>::count();

			let status = vote.try_finalize(threshold, total);
			PayOutVotes::<T>::insert(hash, vote.clone());
			PayOutRequests::<T>::insert(hash, req.clone());
			let mut hashes = match PayOutHashes::<T>::get(&req.source_chain, req.nonce) {
				Some(hashes) => hashes,
				None => vec![hash],
			};

			if !hashes.contains(&hash) {
				hashes.push(hash);
			}
			PayOutHashes::<T>::insert(&req.source_chain, req.nonce, hashes);

			match status {
				VoteStatus::Passed => Self::do_pay_out(
					req.source_chain.clone(),
					req.nonce,
					chain_asset.asset.clone(),
					req.dest_account.clone(),
					req.amount,
				)?,
				VoteStatus::Failed => Self::deposit_event(Event::PayOutRejected {
					source_chain: req.source_chain.clone(),
					nonce: req.nonce,
					asset: chain_asset.asset,
					dest_account: req.dest_account.clone(),
					amount: req.amount,
				}),
				_ => (),
			}

			if vote.is_finalized() {
				Self::do_post_finalize(&req)?;
			}

			Ok(Pays::No.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn add_relayer(origin: OriginFor<T>, who: T::AccountId) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			Relayers::<T>::insert(&who, ());
			Self::deposit_event(Event::RelayerAdded { who });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn remove_relayer(
			origin: OriginFor<T>,
			who: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(Self::is_relayer(&who), Error::<T>::RequireRelayer);
			Relayers::<T>::remove(&who);
			Self::deposit_event(Event::RelayerRemoved { who });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn set_pay_in_fee(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			dest_chain: ChainType,
			fee: T::Balance,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			PayInFee::<T>::insert(&asset, &dest_chain, fee);
			Self::deposit_event(Event::PayInFeeSet { asset, dest_chain, fee });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn set_resource_id(
			origin: OriginFor<T>,
			resource_id: ResourceId,
			chain_asset: ChainAsset<T::AssetKind>,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin.clone())?;
			ResourceIds::<T>::insert(resource_id, chain_asset.clone());
			Self::deposit_event(Event::ResourceIdSet { resource_id, chain_asset });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn remove_resource_id(
			origin: OriginFor<T>,
			resource_id: ResourceId,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(ResourceIds::<T>::contains_key(resource_id), Error::<T>::ResourceIdNotExist);
			ResourceIds::<T>::remove(resource_id);
			Self::deposit_event(Event::ResourceIdRemoved { resource_id });
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

		#[pallet::call_index(9)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn add_pay_in_pair(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			dest_chain: ChainType,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			PayInPair::<T>::insert((&asset, &dest_chain), ());
			Self::deposit_event(Event::PayInPairAdded { asset, dest_chain });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(10)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn remove_pay_in_pair(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			dest_chain: ChainType,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(
				PayInPair::<T>::get((&asset, &dest_chain)).is_some(),
				Error::<T>::PayInPairNotExist
			);
			PayInPair::<T>::remove((&asset, &dest_chain));
			Self::deposit_event(Event::PayInPairRemoved { asset, dest_chain });
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
			Relayers::<T>::contains_key(who)
		}

		fn next_pay_in_nonce(chain: &ChainType) -> Result<Nonce, Error<T>> {
			let nonce = Self::pay_in_nonce(chain);
			let nonce = nonce.checked_add(1).ok_or(Error::<T>::PayInNonceOverflow)?;
			PayInNonce::<T>::insert(chain, nonce);
			Ok(nonce)
		}

		fn do_pay_out(
			source_chain: ChainType,
			nonce: Nonce,
			asset: T::AssetKind,
			dest_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			// do actual payout
			T::Assets::mint_into(asset.clone(), &dest_account, amount)?;
			Self::deposit_event(Event::PaidOut {
				source_chain,
				nonce,
				asset,
				dest_account,
				amount,
			});
			Ok(())
		}

		fn do_post_finalize(req: &PayOutRequest<T::AccountId, T::Balance>) -> DispatchResult {
			if req.nonce > Self::finalized_vote_nonce(&req.source_chain) {
				FinalizedVoteNonce::<T>::insert(&req.source_chain, req.nonce);
				Self::deposit_event(Event::FinalizedVoteNonceUpdated {
					source_chain: req.source_chain.clone(),
					nonce: req.nonce,
				});
			}
			// Try to bump FinalizedPayOutNonce until a non-finalized vote
			// TODO: weight-related: shall we set a max diff to avoid infinite/too many loops?
			let mut n = Self::finalized_pay_out_nonce(&req.source_chain);
			while n < Self::finalized_vote_nonce(&req.source_chain) {
				let next_n = n.checked_add(1).ok_or(Error::<T>::FinalizedPayOutNonceOverflow)?;
				// we are lenient that we don't require `vote` to exist - in practice, it would mean
				// some relayer didn't follow monotonically increasing pay out nonce (for some reason)
				if let Some(hashes) = PayOutHashes::<T>::get(&req.source_chain, next_n) {
					if hashes.iter().any(|h| {
						PayOutVotes::<T>::get(h).map_or_else(|| false, |v| v.is_finalized())
					}) {
						for h in hashes {
							PayOutVotes::<T>::remove(h);
							PayOutRequests::<T>::remove(h);
						}
						PayOutHashes::<T>::remove(&req.source_chain, next_n);
						n = next_n;
					} else {
						// nothing to do
						break;
					}
				} else {
					break;
				}
			}

			if n > Self::finalized_pay_out_nonce(&req.source_chain) {
				FinalizedPayOutNonce::<T>::insert(&req.source_chain, n);
				Self::deposit_event(Event::FinalizedPayOutNonceUpdated {
					source_chain: req.source_chain.clone(),
					nonce: n,
				});
			}
			Ok(())
		}
	}
}
