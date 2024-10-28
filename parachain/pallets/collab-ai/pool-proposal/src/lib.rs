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
//
//! # Pool Proposal Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The Pool Proposal handles the administration of proposed investing pool and pre-investing.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]
pub mod types;

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		tokens::{
			fungibles::{Inspect as FsInspect, Mutate as FsMutate},
			Preservation,
		},
		Currency, EnsureOrigin, Get, LockableCurrency, ReservableCurrency,
	},
	transactional,
	weights::Weight,
	PalletId,
};
use frame_system::{
	ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
	RawOrigin,
};
use orml_utilities::OrderedSet;
pub use pallet::*;
use pallet_collab_ai_common::*;
use sp_runtime::{
	traits::{AccountIdConversion, CheckedAdd, CheckedSub},
	ArithmeticError,
};
use sp_std::collections::vec_deque::VecDeque;

pub use types::*;

pub(crate) type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub(crate) type InspectFungibles<T> = pallet_assets::Pallet<T>;
/// Balance type alias for balances of assets that implement the `fungibles` trait.
pub type AssetBalanceOf<T> =
	<InspectFungibles<T> as FsInspect<<T as frame_system::Config>::AccountId>>::Balance;
/// Type alias for Asset IDs.
pub type AssetIdOf<T> =
	<InspectFungibles<T> as FsInspect<<T as frame_system::Config>::AccountId>>::AssetId;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::OptionQuery};

	use super::*;

	/// CollabAI investing pool proposal
	const MODULE_ID: PalletId = PalletId(*b"cbai/ipp");
	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Make pool mature.
		// type Scheduler: ScheduleNamed<BlockNumberFor<Self>, CallOf<Self>, Self::PalletsOrigin>;

		/// Currency type for this pallet.
		type Currency: ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = BlockNumberFor<Self>>;

		// Declare the asset id of AIUSD
		type AIUSDAssetId: Get<AssetIdOf<Self>>;

		/// Period of time between proposal ended and pool start
		#[pallet::constant]
		type OfficialGapPeriod: Get<BlockNumberFor<Self>>;

		/// Minimum period of time for proposal voting end/expired
		#[pallet::constant]
		type MinimumProposalLastTime: Get<BlockNumberFor<Self>>;

		/// The minimum amount to be used as a deposit for creating a pool curator
		#[pallet::constant]
		type MinimumPoolDeposit: Get<BalanceOf<Self>>;

		/// The maximum amount of allowed pool proposed by a single curator
		#[pallet::constant]
		type MaximumPoolProposed: Get<u32>;

		/// Origin who can make a pool proposal
		type ProposalOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// Origin who can make a pool proposal pass public vote check
		type PublicVotingOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Guardian vote resource
		type GuardianVoteResource: GuardianQuery<Self::AccountId>;

		/// The maximum amount of guardian allowed for a proposal
		#[pallet::constant]
		type MaxGuardianPerProposal: Get<u32>;

		/// System Account holding pre-investing assets
		type PreInvestingPool: Get<Self::AccountId>;
	}

	/// The next free Pool Proposal index, aka the number of pool proposed so far.
	#[pallet::storage]
	#[pallet::getter(fn pool_proposal_count)]
	pub type PoolProposalCount<T> = StorageValue<_, PoolProposalIndex, ValueQuery>;

	/// Those who have a reserve for his pool proposal.
	#[pallet::storage]
	#[pallet::getter(fn pool_proposal_deposit_of)]
	pub type PoolProposalDepositOf<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		OrderedSet<Bond<PoolProposalIndex, BalanceOf<T>>, T::MaximumPoolProposed>,
		OptionQuery,
	>;

	// Pending pool proposal status of investing pools
	// Ordered by expired time
	#[pallet::storage]
	#[pallet::getter(fn pending_pool_proposal_status)]
	pub type PendingPoolProposalStatus<T: Config> =
		StorageValue<_, VecDeque<PoolProposalStatus<BlockNumberFor<T>>>, ValueQuery>;

	// Pool proposal content
	// This storage is not allowed to update once any ProposalStatusFlags passed
	// Yet root is allowed to do that
	#[pallet::storage]
	#[pallet::getter(fn pool_proposal)]
	pub type PoolProposal<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PoolProposalIndex,
		PoolProposalInfo<InfoHash, AssetBalanceOf<T>, BlockNumberFor<T>, T::AccountId>,
		OptionQuery,
	>;

	// Preinvesting of pool proposal
	// This storage will be modified/delete correspondingly when solving pending pool
	#[pallet::storage]
	#[pallet::getter(fn pool_pre_investings)]
	pub type PoolPreInvestings<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PoolProposalIndex,
		PoolProposalPreInvesting<
			T::AccountId,
			AssetBalanceOf<T>,
			BlockNumberFor<T>,
			T::MaximumPoolProposed,
		>,
		OptionQuery,
	>;

	// Guardian willingness of proposal
	#[pallet::storage]
	#[pallet::getter(fn pool_guardian)]
	pub type PoolGuardian<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PoolProposalIndex,
		OrderedSet<T::AccountId, T::MaxGuardianPerProposal>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A motion has been proposed by a public account.
		PoolProposed {
			proposer: T::AccountId,
			pool_proposal_index: PoolProposalIndex,
			info_hash: InfoHash,
		},
		/// A pre investing becomes valid
		PoolPreInvested {
			user: T::AccountId,
			pool_proposal_index: PoolProposalIndex,
			amount: AssetBalanceOf<T>,
		},
		/// A pre investing queued
		PoolPreStakeQueued {
			user: T::AccountId,
			pool_proposal_index: PoolProposalIndex,
			amount: AssetBalanceOf<T>,
		},
		/// A queued pre investing becomes a valid pre investing
		PoolQueuedInvested {
			user: T::AccountId,
			pool_proposal_index: PoolProposalIndex,
			amount: AssetBalanceOf<T>,
		},
		/// Some amount of pre investing regardless of queue or pre invested, withdrawed (Withdraw queue ones first)
		PoolWithdrawed {
			user: T::AccountId,
			pool_proposal_index: PoolProposalIndex,
			amount: AssetBalanceOf<T>,
		},
		/// A public vote result of proposal get passed
		ProposalPublicVoted { pool_proposal_index: PoolProposalIndex, vote_result: bool },
	}

	#[pallet::error]
	pub enum Error<T> {
		ProposalDepositDuplicatedOrOversized,
		ProposalExpired,
		ProposalPreInvestingLocked,
		ProposalPublicTimeTooShort,
		ProposalNotExist,
		InvestingPoolOversized,
		InsufficientPreInvesting,
		GuardianDuplicatedOrOversized,
		GuardianInvalid,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			// Check proposal expire by order

			// curator must be verified by this time
			// check epoch number not too large so asset id will not overflow
			// Mature the pool by proposal if qualified, refund/transfer all money based on investing pool logic

			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Curator propose a investing pool
		///
		/// max_pool_size: At most this amount of raised money curator/investing pool willing to take
		/// proposal_last_time: How long does the proposal lasts for voting/preinvesting.
		///                     All ProposalStatusFlags must be satisfied after this period passed, which is also
		/// 					the approximate date when pool begins.
		/// pool_last_time: How long does the investing pool last if passed
		/// estimated_epoch_reward: This number is only for displaying purpose without any techinical meaning
		/// pool_info_hash: Hash of pool info for including pool details
		#[pallet::call_index(0)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn propose_investing_pool(
			origin: OriginFor<T>,
			max_pool_size: AssetBalanceOf<T>,
			proposal_last_time: BlockNumberFor<T>,
			pool_last_time: BlockNumberFor<T>,
			estimated_epoch_reward: AssetBalanceOf<T>,
			pool_info_hash: InfoHash,
		) -> DispatchResult {
			let who = T::ProposalOrigin::ensure_origin(origin)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(
				proposal_last_time >= T::MinimumProposalLastTime::get(),
				Error::<T>::ProposalPublicTimeTooShort
			);

			let proposal_end_time = current_block
				.checked_add(&proposal_last_time)
				.ok_or(ArithmeticError::Overflow)?;

			let pool_start_time = proposal_end_time
				.checked_add(&T::OfficialGapPeriod::get())
				.ok_or(ArithmeticError::Overflow)?;

			let new_proposal_info = PoolProposalInfo {
				proposer: who.clone(),
				pool_info_hash,
				max_pool_size,
				pool_start_time,
				pool_end_time: pool_start_time
					.checked_add(&pool_last_time)
					.ok_or(ArithmeticError::Overflow)?,
				estimated_epoch_reward,
				proposal_status_flags: ProposalStatusFlags::empty(),
			};

			let next_proposal_index = PoolProposalCount::<T>::get();
			PoolProposal::<T>::insert(next_proposal_index, new_proposal_info);
			PoolProposalDepositOf::<T>::try_mutate_exists(
				&who,
				|maybe_ordered_set| -> Result<(), DispatchError> {
					let reserved_amount = T::MinimumPoolDeposit::get();
					<T as pallet::Config>::Currency::reserve(&who, reserved_amount)?;
					// We should not care about duplicating since the proposal index is auto-increment
					match maybe_ordered_set.as_mut() {
						Some(ordered_set) => {
							ensure!(
								ordered_set.insert(Bond {
									owner: next_proposal_index,
									amount: reserved_amount,
								}),
								Error::<T>::ProposalDepositDuplicatedOrOversized
							);
							Ok(())
						},
						None => {
							let mut new_ordered_set = OrderedSet::new();

							ensure!(
								new_ordered_set.insert(Bond {
									owner: next_proposal_index,
									amount: reserved_amount,
								}),
								Error::<T>::ProposalDepositDuplicatedOrOversized
							);
							*maybe_ordered_set = Some(new_ordered_set);
							Ok(())
						},
					}
				},
			)?;
			<PendingPoolProposalStatus<T>>::mutate(|pending_porposals| {
				let new_proposal_status = PoolProposalStatus {
					pool_proposal_index: next_proposal_index,
					proposal_expire_time: proposal_end_time,
				};
				pending_porposals.push_back(new_proposal_status);
				// Make sure the first element has earlies effective time
				pending_porposals
					.make_contiguous()
					.sort_by(|a, b| a.proposal_expire_time.cmp(&b.proposal_expire_time));
			});
			PoolProposalCount::<T>::put(
				next_proposal_index.checked_add(1u32.into()).ok_or(ArithmeticError::Overflow)?,
			);
			Self::deposit_event(Event::PoolProposed {
				proposer: who,
				pool_proposal_index: next_proposal_index,
				info_hash: pool_info_hash,
			});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn pre_stake_proposal(
			origin: OriginFor<T>,
			pool_proposal_index: PoolProposalIndex,
			amount: AssetBalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let asset_actual_transfer_amount: AssetBalanceOf<T> = <InspectFungibles<T> as FsMutate<<T as frame_system::Config>::AccountId>>::transfer(
				T::AIUSDAssetId::get(),
				&who,
				&T::PreInvestingPool::get(),
				amount,
				Preservation::Expendable,
			)?;

			let mut pool_proposal_pre_investing =
				<PoolPreInvestings<T>>::take(pool_proposal_index).unwrap_or_default();

			// Check pool maximum size limit and make pool size limit flag change accordingly
			let mut pool_proposal =
				<PoolProposal<T>>::get(pool_proposal_index).ok_or(Error::<T>::ProposalNotExist)?;
			// Proposal not expired
			ensure!(
				!pool_proposal
					.proposal_status_flags
					.contains(ProposalStatusFlags::PROPOSAL_EXPIRED),
				Error::<T>::ProposalExpired
			);
			// If proposal is fully pre-Investing or partial oversized after this stake

			// Check BoundedVec limit
			ensure!(
				!pool_proposal_pre_investing.pre_investings.is_full()
					&& !pool_proposal_pre_investing.queued_pre_investings.is_full(),
				Error::<T>::InvestingPoolOversized
			);

			let target_pre_investing_amount = pool_proposal_pre_investing
				.total_pre_investing_amount
				.checked_add(&asset_actual_transfer_amount)
				.ok_or(ArithmeticError::Overflow)?;
			if target_pre_investing_amount <= pool_proposal.max_pool_size {
				// take all pre-investing into valid pre-investing line
				pool_proposal_pre_investing
					.add_pre_investing::<T>(who.clone(), asset_actual_transfer_amount)?;

				// Emit event only
				Self::deposit_event(Event::PoolPreInvested {
					user: who.clone(),
					pool_proposal_index,
					amount: asset_actual_transfer_amount,
				});
				// Flag proposal status if pool is just fully Investing
				if target_pre_investing_amount == pool_proposal.max_pool_size {
					pool_proposal.proposal_status_flags |= ProposalStatusFlags::STAKE_AMOUNT_PASSED;
					<PoolProposal<T>>::insert(pool_proposal_index, pool_proposal);
				}
			} else {
				// Partially
				let queued_pre_investing_amount = target_pre_investing_amount
					.checked_sub(&pool_proposal.max_pool_size)
					.ok_or(ArithmeticError::Overflow)?;
				pool_proposal_pre_investing.add_queued_investing::<T>(
					who.clone(),
					queued_pre_investing_amount,
					frame_system::Pallet::<T>::block_number(),
				)?;

				// If pool not already full, flag proposal status
				if asset_actual_transfer_amount > queued_pre_investing_amount {
					let actual_pre_investing_amount = asset_actual_transfer_amount
						.checked_sub(&queued_pre_investing_amount)
						.ok_or(ArithmeticError::Overflow)?;
					pool_proposal_pre_investing
						.add_pre_investing::<T>(who.clone(), actual_pre_investing_amount)?;

					Self::deposit_event(Event::PoolPreInvested {
						user: who.clone(),
						pool_proposal_index,
						amount: actual_pre_investing_amount,
					});

					pool_proposal.proposal_status_flags |= ProposalStatusFlags::STAKE_AMOUNT_PASSED;
					<PoolProposal<T>>::insert(pool_proposal_index, pool_proposal);
				}

				// Emit events
				Self::deposit_event(Event::PoolPreStakeQueued {
					user: who,
					pool_proposal_index,
					amount: queued_pre_investing_amount,
				});
			}

			<PoolPreInvestings<T>>::insert(pool_proposal_index, pool_proposal_pre_investing);
			Ok(())
		}

		// Withdraw is not allowed when proposal has STAKE_AMOUNT_PASSED flag
		// unless there is queued amount pending
		#[pallet::call_index(2)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn withdraw_pre_investing(
			origin: OriginFor<T>,
			pool_proposal_index: PoolProposalIndex,
			amount: AssetBalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut pool_proposal_pre_investing =
				<PoolPreInvestings<T>>::take(pool_proposal_index).unwrap_or_default();

			let pool_proposal =
				<PoolProposal<T>>::get(pool_proposal_index).ok_or(Error::<T>::ProposalNotExist)?;
			// Either investing pool has not locked yet,
			// Or queued amount is enough to replace the withdrawal
			ensure!(
				!pool_proposal
					.proposal_status_flags
					.contains(ProposalStatusFlags::STAKE_AMOUNT_PASSED)
					|| (pool_proposal_pre_investing.total_queued_amount >= amount),
				Error::<T>::ProposalPreInvestingLocked
			);

			pool_proposal_pre_investing.withdraw::<T>(who.clone(), amount)?;
			Self::deposit_event(Event::PoolWithdrawed {
				user: who.clone(),
				pool_proposal_index,
				amount,
			});

			// Make queued amount fill the missing Investing amount if pool Investing flag ever reached
			if (pool_proposal_pre_investing.total_pre_investing_amount
				< pool_proposal.max_pool_size)
				&& (pool_proposal
					.proposal_status_flags
					.contains(ProposalStatusFlags::STAKE_AMOUNT_PASSED))
			{
				let moved_bonds = pool_proposal_pre_investing
					.move_queued_to_pre_investing_until::<T>(pool_proposal.max_pool_size)?;
				for i in moved_bonds.iter() {
					// Emit events
					Self::deposit_event(Event::PoolQueuedInvested {
						user: i.owner.clone(),
						pool_proposal_index,
						amount: i.amount,
					});
				}
			}

			// Return funds
			let _asset_actual_transfer_amount: AssetBalanceOf<T> = <InspectFungibles<T> as FsMutate<<T as frame_system::Config>::AccountId>>::transfer(
				T::AIUSDAssetId::get(),
				&T::PreInvestingPool::get(),
				&who,
				amount,
				Preservation::Expendable,
			)?;

			<PoolPreInvestings<T>>::insert(pool_proposal_index, pool_proposal_pre_investing);

			Ok(())
		}

		// This is democracy/committe passing check for investing pool proposal
		// TODO: Related logic with "pallet-conviction-voting"
		#[pallet::call_index(3)]
		#[pallet::weight({195_000_000})]
		pub fn public_vote_proposal(
			origin: OriginFor<T>,
			pool_proposal_index: PoolProposalIndex,
			vote: bool,
		) -> DispatchResult {
			T::PublicVotingOrigin::ensure_origin(origin)?;
			let mut pool_proposal =
				<PoolProposal<T>>::get(pool_proposal_index).ok_or(Error::<T>::ProposalNotExist)?;

			if vote {
				pool_proposal.proposal_status_flags |= ProposalStatusFlags::PUBLIC_VOTE_PASSED;
			} else {
				pool_proposal.proposal_status_flags &= !ProposalStatusFlags::PUBLIC_VOTE_PASSED;
			}
			<PoolProposal<T>>::insert(pool_proposal_index, pool_proposal);

			Self::deposit_event(Event::ProposalPublicVoted {
				pool_proposal_index,
				vote_result: vote,
			});
			Ok(())
		}

		// A guardian has decided to participate the investing pool
		// When proposal expired, the guardian must have everything ready
		// Including KYC. Otherwise he will be ignored no matter how much vote he collects
		#[pallet::call_index(4)]
		#[pallet::weight({195_000_000})]
		pub fn guardian_participate_proposal(
			origin: OriginFor<T>,
			pool_proposal_index: PoolProposalIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Ensure guardian exists when participate, will double check if verified when mature the proposal)
			ensure!(T::GuardianVoteResource::is_guardian(who.clone()), Error::<T>::GuardianInvalid);
			PoolGuardian::<T>::try_mutate_exists(
				pool_proposal_index,
				|maybe_ordered_set| -> Result<(), DispatchError> {
					match maybe_ordered_set.as_mut() {
						Some(ordered_set) => {
							ensure!(
								ordered_set.insert(who),
								Error::<T>::GuardianDuplicatedOrOversized
							);
							Ok(())
						},
						None => {
							let mut new_ordered_set = OrderedSet::new();

							ensure!(
								new_ordered_set.insert(who),
								Error::<T>::GuardianDuplicatedOrOversized
							);
							*maybe_ordered_set = Some(new_ordered_set);
							Ok(())
						},
					}
				},
			)
		}
	}

	/// Simple ensure origin from pallet pool proposal
	pub struct EnsurePoolProposal<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> EnsureOrigin<T::RuntimeOrigin> for EnsurePoolProposal<T> {
		type Success = T::AccountId;
		fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
			let sync_account_id = MODULE_ID.into_account_truncating();
			o.into().and_then(|o| match o {
				RawOrigin::Signed(who) if who == sync_account_id => Ok(sync_account_id),
				r => Err(T::RuntimeOrigin::from(r)),
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<T::RuntimeOrigin, ()> {
			let sync_account_id = MODULE_ID.into_account_truncating();
			Ok(T::RuntimeOrigin::from(RawOrigin::Signed(sync_account_id)))
		}
	}

	/// Some sort of check on the origin is from proposer.
	impl<T: Config> ProposerQuery<T::AccountId> for Pallet<T> {
		fn is_proposer(account: T::AccountId, proposal_index: PoolProposalIndex) -> bool {
			if let Some(info) = Self::pool_proposal(proposal_index) {
				info.proposer == account
			} else {
				false
			}
		}
	}
}
