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

//! Inspired by:
//! - Moonbeam `pallet_parachain_staking`
//! implementations.

//! # Parachain Staking
//! Minimal staking pallet that implements collator selection by total backed stake.
//! The main difference between this pallet and `frame/pallet-staking` is that this pallet
//! uses direct delegation. Delegators choose exactly who they delegate and with what stake.
//! This is different from `frame/pallet-staking` where delegators approval vote and run Phragmen.
//!
//! ### Rules
//! There is a new round every `<Round<T>>::get().length` blocks.
//!
//! At the start of every round,
//! * issuance is calculated for collators (and their delegators) for block authoring
//! `T::RewardPaymentDelay` rounds ago
//! * a new set of collators is chosen from the candidates
//!
//! Immediately following a round change, payments are made once-per-block until all payments have
//! been made. In each such block, one collator is chosen for a rewards payment and is paid along
//! with each of its top `T::MaxTopDelegationsPerCandidate` delegators.
//!
//! To join the set of candidates, call `join_candidates` with `bond >= MinCandidateStk`.
//! To leave the set of candidates, call `schedule_leave_candidates`. If the call succeeds,
//! the collator is removed from the pool of candidates so they cannot be selected for future
//! collator sets, but they are not unbonded until their exit request is executed. Any signed
//! account may trigger the exit `T::LeaveCandidatesDelay` rounds after the round in which the
//! original request was made.
//!
//! To join the set of delegators, call `delegate` and pass in an account that is
//! already a collator candidate and `bond >= MinDelegatorStk`. Each delegator can delegate up to
//! `T::MaxDelegationsPerDelegator` collator candidates by calling `delegate`.
//!
//! To revoke a delegation, call `revoke_delegation` with the collator candidate's account.
//! To leave the set of delegators and revoke all delegations, call `leave_delegators`.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unit_arg)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::needless_borrow)]

mod auto_compound;
pub mod delegation_requests;
pub mod inflation;
pub mod traits;
pub mod types;
pub mod weights;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarking;
#[cfg(test)]
mod mock;
pub mod set;
#[cfg(test)]
mod tests;

pub use inflation::{InflationInfo, Range};
pub use weights::WeightInfo;

pub use auto_compound::{AutoCompoundConfig, AutoCompoundDelegations};
pub use delegation_requests::{CancelledScheduledRequest, DelegationAction, ScheduledRequest};
pub use pallet::*;
pub use traits::*;
pub use types::*;
pub use RoundIndex;

#[frame_support::pallet]
pub mod pallet {
	use crate::{
		delegation_requests::{CancelledScheduledRequest, DelegationAction, ScheduledRequest},
		set::OrderedSet,
		traits::*,
		types::*,
		AutoCompoundConfig, AutoCompoundDelegations, InflationInfo, Range, WeightInfo,
	};
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{
			Currency, EstimateNextSessionRotation, Get, Imbalance, LockableCurrency,
			ReservableCurrency,
		},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{Saturating, Zero},
		Perbill, Percent, Permill,
	};
	use sp_staking::SessionIndex;
	use sp_std::{collections::btree_map::BTreeMap, prelude::*};

	/// Pallet for parachain staking
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type RoundIndex = u32;
	type RewardPoint = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency type
		/// We make Currency Lockable on purpose
		/// So we can test its interaction with Locks
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;
		/// The origin for monetary governance
		type MonetaryGovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Minimum number of blocks per round
		#[pallet::constant]
		type MinBlocksPerRound: Get<u32>;
		/// Default number of blocks per round at genesis
		#[pallet::constant]
		type DefaultBlocksPerRound: Get<u32>;
		/// Number of rounds that candidates remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveCandidatesDelay: Get<RoundIndex>;
		/// Number of rounds candidate requests to decrease self-bond must wait to be executable
		#[pallet::constant]
		type CandidateBondLessDelay: Get<RoundIndex>;
		/// Number of rounds that delegators remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveDelegatorsDelay: Get<RoundIndex>;
		/// Number of rounds that delegations remain bonded before revocation request is executable
		#[pallet::constant]
		type RevokeDelegationDelay: Get<RoundIndex>;
		/// Number of rounds that delegation less requests must wait before executable
		#[pallet::constant]
		type DelegationBondLessDelay: Get<RoundIndex>;
		/// Number of rounds after which block authors are rewarded
		#[pallet::constant]
		type RewardPaymentDelay: Get<RoundIndex>;
		/// Minimum number of selected candidates every round
		#[pallet::constant]
		type MinSelectedCandidates: Get<u32>;
		/// Maximum top delegations counted per candidate
		#[pallet::constant]
		type MaxTopDelegationsPerCandidate: Get<u32>;
		/// Maximum bottom delegations (not counted) per candidate
		#[pallet::constant]
		type MaxBottomDelegationsPerCandidate: Get<u32>;
		/// Maximum delegations per delegator
		#[pallet::constant]
		type MaxDelegationsPerDelegator: Get<u32>;
		/// Default commission due to collators, is `CollatorCommission` storage value in genesis
		#[pallet::constant]
		type DefaultCollatorCommission: Get<Perbill>;
		/// Default percent of inflation set aside for parachain bond account
		#[pallet::constant]
		type DefaultParachainBondReservePercent: Get<Percent>;
		/// Minimum stake required for any candidate to be in `SelectedCandidates` for the round
		#[pallet::constant]
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be a collator candidate
		#[pallet::constant]
		type MinCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to delegate
		#[pallet::constant]
		type MinDelegation: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to be a delegator
		#[pallet::constant]
		type MinDelegatorStk: Get<BalanceOf<Self>>;
		/// Handler to notify the runtime when a collator is paid.
		/// If you don't need it, you can specify the type `()`.
		type OnCollatorPayout: OnCollatorPayout<Self::AccountId, BalanceOf<Self>>;
		/// Handler to notify the runtime when a new round begin.
		/// If you don't need it, you can specify the type `()`.
		type OnNewRound: OnNewRound;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
		/// The source for adjusted inflation base.
		type IssuanceAdapter: IssuanceAdapter<BalanceOf<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		DelegatorDNE,
		DelegatorDNEinTopNorBottom,
		DelegatorDNEInDelegatorSet,
		CandidateDNE,
		DelegationDNE,
		DelegatorExists,
		CandidateExists,
		CandidateBondBelowMin,
		InsufficientBalance,
		DelegatorBondBelowMin,
		DelegationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		DelegatorAlreadyLeaving,
		DelegatorNotLeaving,
		DelegatorCannotLeaveYet,
		CannotDelegateIfLeaving,
		CandidateAlreadyLeaving,
		CandidateNotLeaving,
		CandidateCannotLeaveYet,
		CannotGoOnlineIfLeaving,
		ExceedMaxDelegationsPerDelegator,
		AlreadyDelegatedCandidate,
		InvalidSchedule,
		CannotSetBelowMin,
		RoundLengthMustBeGreaterThanTotalSelectedCollators,
		NoWritingSameValue,
		TooLowCandidateCountWeightHintCancelLeaveCandidates,
		TooLowCandidateDelegationCountToLeaveCandidates,
		PendingCandidateRequestsDNE,
		PendingCandidateRequestAlreadyExists,
		PendingCandidateRequestNotDueYet,
		PendingDelegationRequestDNE,
		PendingDelegationRequestAlreadyExists,
		PendingDelegationRequestNotDueYet,
		CannotDelegateLessThanOrEqualToLowestBottomWhenFull,
		PendingDelegationRevoke,
		CandidateUnauthorized,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Started new round.
		NewRound {
			starting_block: T::BlockNumber,
			round: RoundIndex,
			selected_collators_number: u32,
			total_balance: BalanceOf<T>,
		},
		/// Account joined the set of collator candidates.
		JoinedCollatorCandidates {
			account: T::AccountId,
			amount_locked: BalanceOf<T>,
			new_total_amt_locked: BalanceOf<T>,
		},
		/// Candidate selected for collators. Total Exposed Amount includes all delegations.
		CollatorChosen {
			round: RoundIndex,
			collator_account: T::AccountId,
			total_exposed_amount: BalanceOf<T>,
		},
		/// Candidate requested to decrease a self bond.
		CandidateBondLessRequested {
			candidate: T::AccountId,
			amount_to_decrease: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		/// Candidate has increased a self bond.
		CandidateBondedMore {
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			new_total_bond: BalanceOf<T>,
		},
		/// Candidate has decreased a self bond.
		CandidateBondedLess {
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			new_bond: BalanceOf<T>,
		},
		/// Candidate temporarily leave the set of collator candidates without unbonding.
		CandidateWentOffline {
			candidate: T::AccountId,
		},
		/// Candidate rejoins the set of collator candidates.
		CandidateBackOnline {
			candidate: T::AccountId,
		},
		/// Candidate has requested to leave the set of candidates.
		CandidateScheduledExit {
			exit_allowed_round: RoundIndex,
			candidate: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Cancelled request to leave the set of candidates.
		CancelledCandidateExit {
			candidate: T::AccountId,
		},
		/// Cancelled request to decrease candidate's bond.
		CancelledCandidateBondLess {
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		/// Candidate has left the set of candidates.
		CandidateLeft {
			ex_candidate: T::AccountId,
			unlocked_amount: BalanceOf<T>,
			new_total_amt_locked: BalanceOf<T>,
		},
		/// Delegator requested to decrease a bond for the collator candidate.
		DelegationDecreaseScheduled {
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount_to_decrease: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		// Delegation increased.
		DelegationIncreased {
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			in_top: bool,
		},
		// Delegation decreased.
		DelegationDecreased {
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			in_top: bool,
		},
		/// Delegator requested to leave the set of delegators.
		DelegatorExitScheduled {
			round: RoundIndex,
			delegator: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Delegator requested to revoke delegation.
		DelegationRevocationScheduled {
			round: RoundIndex,
			delegator: T::AccountId,
			candidate: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Delegator has left the set of delegators.
		DelegatorLeft {
			delegator: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Delegation revoked.
		DelegationRevoked {
			delegator: T::AccountId,
			candidate: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Delegation kicked.
		DelegationKicked {
			delegator: T::AccountId,
			candidate: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Cancelled a pending request to exit the set of delegators.
		DelegatorExitCancelled {
			delegator: T::AccountId,
		},
		/// Cancelled request to change an existing delegation.
		CancelledDelegationRequest {
			delegator: T::AccountId,
			cancelled_request: CancelledScheduledRequest<BalanceOf<T>>,
			collator: T::AccountId,
		},
		/// New delegation (increase of the existing one).
		Delegation {
			delegator: T::AccountId,
			locked_amount: BalanceOf<T>,
			candidate: T::AccountId,
			delegator_position: DelegatorAdded<BalanceOf<T>>,
			auto_compound: Percent,
		},
		/// Delegation from candidate state has been remove.
		DelegatorLeftCandidate {
			delegator: T::AccountId,
			candidate: T::AccountId,
			unstaked_amount: BalanceOf<T>,
			total_candidate_staked: BalanceOf<T>,
		},
		/// Paid the account (delegator or collator) the balance as liquid rewards.
		Rewarded {
			account: T::AccountId,
			rewards: BalanceOf<T>,
		},
		/// Transferred to account which holds funds reserved for parachain bond.
		ReservedForParachainBond {
			account: T::AccountId,
			value: BalanceOf<T>,
		},
		/// Account (re)set for parachain bond treasury.
		ParachainBondAccountSet {
			old: T::AccountId,
			new: T::AccountId,
		},
		/// Percent of inflation reserved for parachain bond (re)set.
		ParachainBondReservePercentSet {
			old: Percent,
			new: Percent,
		},
		/// Annual inflation input (first 3) was used to derive new per-round inflation (last 3)
		InflationSet {
			annual_min: Perbill,
			annual_ideal: Perbill,
			annual_max: Perbill,
			round_min: Perbill,
			round_ideal: Perbill,
			round_max: Perbill,
		},
		/// Staking expectations set.
		StakeExpectationsSet {
			expect_min: BalanceOf<T>,
			expect_ideal: BalanceOf<T>,
			expect_max: BalanceOf<T>,
		},
		/// Set total selected candidates to this value.
		TotalSelectedSet {
			old: u32,
			new: u32,
		},
		/// Set collator commission to this value.
		CollatorCommissionSet {
			old: Perbill,
			new: Perbill,
		},
		/// Set blocks per round
		BlocksPerRoundSet {
			current_round: RoundIndex,
			first_block: T::BlockNumber,
			old: u32,
			new: u32,
			new_per_round_inflation_min: Perbill,
			new_per_round_inflation_ideal: Perbill,
			new_per_round_inflation_max: Perbill,
		},
		CandidateWhiteListAdded {
			candidate: T::AccountId,
		},
		CandidateWhiteListRemoved {
			candidate: T::AccountId,
		},
		/// Auto-compounding reward percent was set for a delegation.
		AutoCompoundSet {
			candidate: T::AccountId,
			delegator: T::AccountId,
			value: Percent,
		},
		/// Compounded a portion of rewards towards the delegation.
		Compounded {
			candidate: T::AccountId,
			delegator: T::AccountId,
			amount: BalanceOf<T>,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let mut weight = T::WeightInfo::base_on_initialize();

			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// mutate round
				round.update(n);
				// notify that new round begin
				weight = weight.saturating_add(T::OnNewRound::on_new_round(round.current));
				// pay all stakers for T::RewardPaymentDelay rounds ago
				Self::prepare_staking_payouts(round.current);
				// select top collator candidates for next round
				let (collator_count, delegation_count, total_staked) =
					Self::select_top_candidates(round.current);
				// start next round
				<Round<T>>::put(round);
				// snapshot total stake
				<Staked<T>>::insert(round.current, <Total<T>>::get());
				Self::deposit_event(Event::NewRound {
					starting_block: round.first,
					round: round.current,
					selected_collators_number: collator_count,
					total_balance: total_staked,
				});
				weight = weight.saturating_add(T::WeightInfo::round_transition_on_initialize(
					collator_count,
					delegation_count,
				));
			}

			weight = weight.saturating_add(Self::handle_delayed_payouts(round.current));

			weight
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn collator_commission)]
	/// Commission percent taken off of rewards for all collators
	type CollatorCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_selected)]
	/// The total candidates selected every round
	type TotalSelected<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn parachain_bond_info)]
	/// Parachain bond config info { account, percent_of_inflation }
	type ParachainBondInfo<T: Config> =
		StorageValue<_, ParachainBondConfig<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	/// Get delegator state associated with an account if account is delegating else None
	pub type DelegatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegator<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_info)]
	/// Get collator candidate info associated with an account if account is candidate else None
	pub type CandidateInfo<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, CandidateMetadata<BalanceOf<T>>, OptionQuery>;

	/// Stores outstanding delegation requests per collator.
	#[pallet::storage]
	#[pallet::getter(fn delegation_scheduled_requests)]
	pub type DelegationScheduledRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<ScheduledRequest<T::AccountId, BalanceOf<T>>>,
		ValueQuery,
	>;

	/// Stores auto-compounding configuration per collator.
	#[pallet::storage]
	#[pallet::getter(fn auto_compounding_delegations)]
	pub(crate) type AutoCompoundingDelegations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<AutoCompoundConfig<T::AccountId>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn top_delegations)]
	/// Top delegations for collator candidate
	pub type TopDelegations<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegations<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn bottom_delegations)]
	/// Bottom delegations for collator candidate
	pub type BottomDelegations<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegations<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn selected_candidates)]
	/// The collator candidates selected for the current round
	type SelectedCandidates<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	/// Total capital locked by this staking pallet
	pub type Total<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	/// The pool of collator candidates, each with their total backing stake
	pub type CandidatePool<T: Config> =
		StorageValue<_, OrderedSet<Bond<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of collator delegation stake at the start of the round
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		CollatorSnapshot<T::AccountId, BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn delayed_payouts)]
	/// Delayed payouts
	pub type DelayedPayouts<T: Config> =
		StorageMap<_, Twox64Concat, RoundIndex, DelayedPayout<BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn staked)]
	/// Total counted stake for selected candidates in the round
	pub type Staked<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn inflation_config)]
	/// Inflation configuration
	pub type InflationConfig<T: Config> = StorageValue<_, InflationInfo<BalanceOf<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn points)]
	/// Total points awarded to collators for block production in the round
	pub type Points<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, RewardPoint, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn awarded_pts)]
	/// Points for each collator per round
	pub type AwardedPts<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		RewardPoint,
		ValueQuery,
	>;

	/// The whitelist of collation candidates.
	/// This storage should be safe to delete after
	/// we release the restriction
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub candidates: Vec<(T::AccountId, BalanceOf<T>)>,
		/// Vec of tuples of the format (delegator AccountId, collator AccountId, delegation
		/// Amount)
		pub delegations: Vec<(T::AccountId, T::AccountId, BalanceOf<T>, Percent)>,
		pub inflation_config: InflationInfo<BalanceOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { candidates: vec![], delegations: vec![], inflation_config: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// Put the candiadates into the whitelist list
			// This should be safe to delete after removing whitelist
			let mut genesis_candidates = self.candidates.clone();
			genesis_candidates.sort();
			let candidates =
				genesis_candidates.into_iter().map(|x| x.0).collect::<Vec<T::AccountId>>();
			<Candidates<T>>::put(candidates);

			<InflationConfig<T>>::put(self.inflation_config.clone());
			let mut candidate_count = 0u32;
			// Initialize the candidates
			for &(ref candidate, balance) in &self.candidates {
				assert!(
					<Pallet<T>>::get_delegator_stakable_free_balance(candidate) >= balance,
					"Account does not have enough balance to bond as a candidate."
				);

				if let Err(error) = <Pallet<T>>::join_candidates(
					T::RuntimeOrigin::from(Some(candidate.clone()).into()),
					balance,
				) {
					log::warn!("Join candidates failed in genesis with error {:?}", error);
				} else {
					candidate_count = candidate_count.saturating_add(1u32);
				}
			}
			let mut col_delegator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			let mut del_delegation_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			// Initialize the delegations
			for &(ref delegator, ref target, balance, auto_compound) in &self.delegations {
				assert!(
					<Pallet<T>>::get_delegator_stakable_free_balance(delegator) >= balance,
					"Account does not have enough balance to place delegation."
				);
				if let Err(error) = <Pallet<T>>::delegate_with_auto_compound(
					T::RuntimeOrigin::from(Some(delegator.clone()).into()),
					target.clone(),
					balance,
					auto_compound,
				) {
					log::warn!("Delegate failed in genesis with error {:?}", error);
				} else {
					if let Some(x) = col_delegator_count.get_mut(target) {
						*x = x.saturating_add(1u32);
					} else {
						col_delegator_count.insert(target.clone(), 1u32);
					};
					if let Some(x) = del_delegation_count.get_mut(delegator) {
						*x = x.saturating_add(1u32);
					} else {
						del_delegation_count.insert(delegator.clone(), 1u32);
					};
				}
			}
			// Set collator commission to default config
			<CollatorCommission<T>>::put(T::DefaultCollatorCommission::get());
			// Set parachain bond config to default config
			<ParachainBondInfo<T>>::put(ParachainBondConfig {
				// must be set soon; if not => due inflation will be sent to collators/delegators
				account: T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
					.expect("infinite length input; no invalid inputs for type; qed"),
				percent: T::DefaultParachainBondReservePercent::get(),
			});
			// Set total selected candidates to minimum config
			<TotalSelected<T>>::put(T::MinSelectedCandidates::get());
			// Choose top TotalSelected collator candidates
			let (v_count, _, total_staked) = <Pallet<T>>::select_top_candidates(1u32);
			// Start Round 1 at Block 0
			let round: RoundInfo<T::BlockNumber> =
				RoundInfo::new(1u32, 0u32.into(), T::DefaultBlocksPerRound::get());
			<Round<T>>::put(round);
			// Snapshot total stake
			<Staked<T>>::insert(1u32, <Total<T>>::get());
			<Pallet<T>>::deposit_event(Event::NewRound {
				starting_block: T::BlockNumber::zero(),
				round: 1u32,
				selected_collators_number: v_count,
				total_balance: total_staked,
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(< T as Config >::WeightInfo::set_staking_expectations())]
		/// Set the expectations for total staked. These expectations determine the issuance for
		/// the round according to logic in `fn compute_issuance`
		pub fn set_staking_expectations(
			origin: OriginFor<T>,
			expectations: Range<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			ensure!(expectations.is_valid(), Error::<T>::InvalidSchedule);
			let mut config = <InflationConfig<T>>::get();
			ensure!(config.expect != expectations, Error::<T>::NoWritingSameValue);
			config.set_expectations(expectations);
			Self::deposit_event(Event::StakeExpectationsSet {
				expect_min: config.expect.min,
				expect_ideal: config.expect.ideal,
				expect_max: config.expect.max,
			});
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}
		#[pallet::call_index(1)]
		#[pallet::weight(< T as Config >::WeightInfo::set_inflation())]
		/// Set the annual inflation rate to derive per-round inflation
		pub fn set_inflation(
			origin: OriginFor<T>,
			schedule: Range<Perbill>,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			ensure!(schedule.is_valid(), Error::<T>::InvalidSchedule);
			let mut config = <InflationConfig<T>>::get();
			ensure!(config.annual != schedule, Error::<T>::NoWritingSameValue);
			config.annual = schedule;
			config.set_round_from_annual::<T>(schedule);
			Self::deposit_event(Event::InflationSet {
				annual_min: config.annual.min,
				annual_ideal: config.annual.ideal,
				annual_max: config.annual.max,
				round_min: config.round.min,
				round_ideal: config.round.ideal,
				round_max: config.round.max,
			});
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}
		#[pallet::call_index(2)]
		#[pallet::weight(< T as Config >::WeightInfo::set_parachain_bond_account())]
		/// Set the account that will hold funds set aside for parachain bond
		pub fn set_parachain_bond_account(
			origin: OriginFor<T>,
			new: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			let ParachainBondConfig { account: old, percent } = <ParachainBondInfo<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<ParachainBondInfo<T>>::put(ParachainBondConfig { account: new.clone(), percent });
			Self::deposit_event(Event::ParachainBondAccountSet { old, new });
			Ok(().into())
		}
		#[pallet::call_index(3)]
		#[pallet::weight(< T as Config >::WeightInfo::set_parachain_bond_reserve_percent())]
		/// Set the percent of inflation set aside for parachain bond
		pub fn set_parachain_bond_reserve_percent(
			origin: OriginFor<T>,
			new: Percent,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			let ParachainBondConfig { account, percent: old } = <ParachainBondInfo<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<ParachainBondInfo<T>>::put(ParachainBondConfig { account, percent: new });
			Self::deposit_event(Event::ParachainBondReservePercentSet { old, new });
			Ok(().into())
		}
		#[pallet::call_index(4)]
		#[pallet::weight(< T as Config >::WeightInfo::set_total_selected())]
		/// Set the total number of collator candidates selected per round
		/// - changes are not applied until the start of the next round
		pub fn set_total_selected(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinSelectedCandidates::get(), Error::<T>::CannotSetBelowMin);
			let old = <TotalSelected<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			ensure!(
				new < <Round<T>>::get().length,
				Error::<T>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
			);
			<TotalSelected<T>>::put(new);
			Self::deposit_event(Event::TotalSelectedSet { old, new });
			Ok(().into())
		}
		#[pallet::call_index(5)]
		#[pallet::weight(< T as Config >::WeightInfo::set_collator_commission())]
		/// Set the commission for all collators
		pub fn set_collator_commission(
			origin: OriginFor<T>,
			new: Perbill,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let old = <CollatorCommission<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<CollatorCommission<T>>::put(new);
			Self::deposit_event(Event::CollatorCommissionSet { old, new });
			Ok(().into())
		}
		#[pallet::call_index(6)]
		#[pallet::weight(< T as Config >::WeightInfo::set_blocks_per_round())]
		/// Set blocks per round
		/// - if called with `new` less than length of current round, will transition immediately
		/// in the next block
		/// - also updates per-round inflation config
		pub fn set_blocks_per_round(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(new >= T::MinBlocksPerRound::get(), Error::<T>::CannotSetBelowMin);
			let mut round = <Round<T>>::get();
			let (now, first, old) = (round.current, round.first, round.length);
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			ensure!(
				new > <TotalSelected<T>>::get(),
				Error::<T>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
			);
			round.length = new;
			// update per-round inflation given new rounds per year
			let mut inflation_config = <InflationConfig<T>>::get();
			inflation_config.reset_round(new);
			<Round<T>>::put(round);
			Self::deposit_event(Event::BlocksPerRoundSet {
				current_round: now,
				first_block: first,
				old,
				new,
				new_per_round_inflation_min: inflation_config.round.min,
				new_per_round_inflation_ideal: inflation_config.round.ideal,
				new_per_round_inflation_max: inflation_config.round.max,
			});
			<InflationConfig<T>>::put(inflation_config);
			Ok(().into())
		}
		#[pallet::call_index(7)]
		#[pallet::weight(< T as Config >::WeightInfo::add_candidates_whitelist(< Candidates < T >>::get().len() as u32))]
		/// add white list of candidates
		/// This function should be safe to delete after restriction removed
		pub fn add_candidates_whitelist(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let mut candidates = <Candidates<T>>::get();
			if let Err(index) = candidates.binary_search(&candidate) {
				candidates.insert(index, candidate.clone());
				Candidates::<T>::put(candidates);
				Self::deposit_event(Event::CandidateWhiteListAdded { candidate });
			}
			Ok(().into())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(< T as Config >::WeightInfo::remove_candidates_whitelist(< Candidates < T >>::get().len() as u32))]
		/// remove white list of candidates
		/// This function should be safe to delete after restriction removed
		pub fn remove_candidates_whitelist(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let mut candidates = <Candidates<T>>::get();
			if let Ok(index) = candidates.binary_search(&candidate) {
				candidates.remove(index);
				Candidates::<T>::put(candidates);
				Self::deposit_event(Event::CandidateWhiteListRemoved { candidate });
			}
			Ok(().into())
		}
		#[pallet::call_index(9)]
		#[pallet::weight(< T as Config >::WeightInfo::join_candidates(<CandidatePool <T>>::get().0.len() as u32))]
		/// Join the set of collator candidates
		pub fn join_candidates(
			origin: OriginFor<T>,
			bond: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(Self::is_white_list(&acc), Error::<T>::CandidateUnauthorized);
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_delegator(&acc), Error::<T>::DelegatorExists);
			ensure!(bond >= T::MinCandidateStk::get(), Error::<T>::CandidateBondBelowMin);
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond { owner: acc.clone(), amount: bond }),
				Error::<T>::CandidateExists
			);
			ensure!(
				<Pallet<T>>::get_delegator_stakable_free_balance(&acc) >= bond,
				Error::<T>::InsufficientBalance
			);
			T::Currency::reserve(&acc, bond)?;
			let candidate = CandidateMetadata::new(bond);
			<CandidateInfo<T>>::insert(&acc, candidate);
			let empty_delegations: Delegations<T::AccountId, BalanceOf<T>> = Default::default();
			// insert empty top delegations
			<TopDelegations<T>>::insert(&acc, empty_delegations.clone());
			// insert empty bottom delegations
			<BottomDelegations<T>>::insert(&acc, empty_delegations);
			<CandidatePool<T>>::put(candidates);
			let new_total = <Total<T>>::get().saturating_add(bond);
			<Total<T>>::put(new_total);
			Self::deposit_event(Event::JoinedCollatorCandidates {
				account: acc,
				amount_locked: bond,
				new_total_amt_locked: new_total,
			});
			Ok(().into())
		}
		#[pallet::call_index(10)]
		#[pallet::weight(< T as Config >::WeightInfo::schedule_leave_candidates(<CandidatePool < T >>::get().0.len() as u32))]
		/// Request to leave the set of candidates. If successful, the account is immediately
		/// removed from the candidate pool to prevent selection as a collator.
		pub fn schedule_leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (now, when) = state.schedule_leave::<T>()?;
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateScheduledExit {
				exit_allowed_round: now,
				candidate: collator,
				scheduled_exit: when,
			});
			Ok(().into())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(
		< T as Config >::WeightInfo::execute_leave_candidates(
			// Max CandidateDelegationCount
			<T as Config>::MaxTopDelegationsPerCandidate::get() + <T as Config>::MaxBottomDelegationsPerCandidate::get(),
		)
		)]
		/// Execute leave candidates request
		pub fn execute_leave_candidates(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let state = <CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			state.can_leave::<T>()?;
			let return_stake = |bond: Bond<T::AccountId, BalanceOf<T>>| -> DispatchResult {
				T::Currency::unreserve(&bond.owner, bond.amount);
				// remove delegation from delegator state
				let mut delegator = DelegatorState::<T>::get(&bond.owner).expect(
					"Collator state and delegator state are consistent.
						Collator state has a record of this delegation. Therefore,
						Delegator state also has a record. qed.",
				);

				if let Some(remaining) = delegator.rm_delegation(&candidate) {
					Self::delegation_remove_request_with_state(
						&candidate,
						&bond.owner,
						&mut delegator,
					);
					<AutoCompoundDelegations<T>>::remove_auto_compound(&candidate, &bond.owner);

					if remaining.is_zero() {
						// we do not remove the scheduled delegation requests from other collators
						// since it is assumed that they were removed incrementally before only the
						// last delegation was left.
						<DelegatorState<T>>::remove(&bond.owner);
					} else {
						<DelegatorState<T>>::insert(&bond.owner, delegator);
					}
				}
				Ok(())
			};
			// total backing stake is at least the candidate self bond
			let mut total_backing = state.bond;
			// return all top delegations
			let top_delegations =
				<TopDelegations<T>>::take(&candidate).expect("CandidateInfo existence checked");
			for bond in top_delegations.delegations {
				return_stake(bond)?;
			}
			total_backing = total_backing.saturating_add(top_delegations.total);
			// return all bottom delegations
			let bottom_delegations =
				<BottomDelegations<T>>::take(&candidate).expect("CandidateInfo existence checked");
			for bond in bottom_delegations.delegations {
				return_stake(bond)?;
			}
			total_backing = total_backing.saturating_add(bottom_delegations.total);
			// return stake to collator
			T::Currency::unreserve(&candidate, state.bond);
			<CandidateInfo<T>>::remove(&candidate);
			<DelegationScheduledRequests<T>>::remove(&candidate);
			<AutoCompoundingDelegations<T>>::remove(&candidate);
			<TopDelegations<T>>::remove(&candidate);
			<BottomDelegations<T>>::remove(&candidate);
			let new_total_staked = <Total<T>>::get().saturating_sub(total_backing);
			<Total<T>>::put(new_total_staked);
			Self::deposit_event(Event::CandidateLeft {
				ex_candidate: candidate,
				unlocked_amount: total_backing,
				new_total_amt_locked: new_total_staked,
			});
			let actual_weight =
				Some(T::WeightInfo::execute_leave_candidates(state.delegation_count as u32));
			Ok(actual_weight.into())
		}
		#[pallet::call_index(12)]
		#[pallet::weight(< T as Config >::WeightInfo::cancel_leave_candidates(< CandidatePool < T >>::get().0.len() as u32))]
		/// Cancel open request to leave candidates
		/// - only callable by collator account
		/// - result upon successful call is the candidate is active in the candidate pool
		pub fn cancel_leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_leaving(), Error::<T>::CandidateNotLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond { owner: collator.clone(), amount: state.total_counted }),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CancelledCandidateExit { candidate: collator });
			Ok(().into())
		}
		#[pallet::call_index(13)]
		#[pallet::weight(< T as Config >::WeightInfo::go_offline())]
		/// Temporarily leave the set of collator candidates without unbonding
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(), Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateWentOffline { candidate: collator });
			Ok(().into())
		}
		#[pallet::call_index(14)]
		#[pallet::weight(< T as Config >::WeightInfo::go_online())]
		/// Rejoin the set of collator candidates if previously had called `go_offline`
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(), Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(), Error::<T>::CannotGoOnlineIfLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond { owner: collator.clone(), amount: state.total_counted }),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBackOnline { candidate: collator });
			Ok(().into())
		}
		#[pallet::call_index(15)]
		#[pallet::weight(< T as Config >::WeightInfo::candidate_bond_more())]
		/// Increase collator candidate self bond by `more`
		pub fn candidate_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			state.bond_more::<T>(collator.clone(), more)?;
			let (is_active, total_counted) = (state.is_active(), state.total_counted);
			<CandidateInfo<T>>::insert(&collator, state);
			if is_active {
				Self::update_active(collator, total_counted);
			}
			Ok(().into())
		}
		#[pallet::call_index(16)]
		#[pallet::weight(< T as Config >::WeightInfo::schedule_candidate_bond_less())]
		/// Request by collator candidate to decrease self bond by `less`
		pub fn schedule_candidate_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let when = state.schedule_bond_less::<T>(less)?;
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBondLessRequested {
				candidate: collator,
				amount_to_decrease: less,
				execute_round: when,
			});
			Ok(().into())
		}
		#[pallet::call_index(17)]
		#[pallet::weight(< T as Config >::WeightInfo::execute_candidate_bond_less())]
		/// Execute pending request to adjust the collator candidate self bond
		pub fn execute_candidate_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward this if caller != candidate
			let mut state = <CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			state.execute_bond_less::<T>(candidate.clone())?;
			<CandidateInfo<T>>::insert(&candidate, state);
			Ok(().into())
		}
		#[pallet::call_index(18)]
		#[pallet::weight(< T as Config >::WeightInfo::cancel_candidate_bond_less())]
		/// Cancel pending request to adjust the collator candidate self bond
		pub fn cancel_candidate_bond_less(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			state.cancel_bond_less::<T>(collator.clone())?;
			<CandidateInfo<T>>::insert(&collator, state);
			Ok(().into())
		}
		#[pallet::call_index(19)]
		#[pallet::weight(
			< T as Config >::WeightInfo::delegate_with_auto_compound(
			// Max CandidateDelegationCount
			<T as Config>::MaxTopDelegationsPerCandidate::get() + <T as Config>::MaxBottomDelegationsPerCandidate::get(),
			0,
			// Max DelegationCount
			<T as Config>::MaxDelegationsPerDelegator::get(),
		))]
		/// If caller is not a delegator and not a collator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their delegation state
		pub fn delegate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::delegate_with_auto_compound(
				candidate,
				delegator,
				amount,
				Percent::zero(),
			)
		}
		/// If caller is not a delegator and not a collator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their delegation state
		/// Sets the auto-compound config for the delegation
		#[pallet::call_index(20)]
		#[pallet::weight(
			< T as Config >::WeightInfo::delegate_with_auto_compound(
			// Max CandidateDelegationCount
			<T as Config>::MaxTopDelegationsPerCandidate::get() + <T as Config>::MaxBottomDelegationsPerCandidate::get(),
			// Max CandidateAutoCompoundingDelegationCount
			<T as Config>::MaxDelegationsPerDelegator::get(),
			// Max DelegationCount
			<T as Config>::MaxDelegationsPerDelegator::get(),
		))]
		pub fn delegate_with_auto_compound(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			auto_compound: Percent,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::delegate_with_auto_compound(
				candidate,
				delegator,
				amount,
				auto_compound,
			)
		}
		#[pallet::call_index(21)]
		#[pallet::weight(< T as Config >::WeightInfo::schedule_leave_delegators())]
		/// Request to leave the set of delegators. If successful, the caller is scheduled to be
		/// allowed to exit via a [DelegationAction::Revoke] towards all existing delegations.
		/// Success forbids future delegation requests until the request is invoked or cancelled.
		pub fn schedule_leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegator_schedule_revoke_all(delegator)
		}
		#[pallet::call_index(22)]
		#[pallet::weight(< T as Config >::WeightInfo::execute_leave_delegators(
			// Max DelegationCount
			<T as Config>::MaxDelegationsPerDelegator::get(),
		))]
		/// Execute the right to exit the set of delegators and revoke all ongoing delegations.
		pub fn execute_leave_delegators(
			origin: OriginFor<T>,
			delegator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			Self::delegator_execute_scheduled_revoke_all(delegator)
		}
		#[pallet::call_index(23)]
		#[pallet::weight(< T as Config >::WeightInfo::cancel_leave_delegators())]
		/// Cancel a pending request to exit the set of delegators. Success clears the pending exit
		/// request (thereby resetting the delay upon another `leave_delegators` call).
		pub fn cancel_leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegator_cancel_scheduled_revoke_all(delegator)
		}

		#[pallet::call_index(24)]
		#[pallet::weight(< T as Config >::WeightInfo::schedule_revoke_delegation())]
		/// Request to revoke an existing delegation. If successful, the delegation is scheduled
		/// to be allowed to be revoked via the `execute_delegation_request` extrinsic.
		pub fn schedule_revoke_delegation(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_schedule_revoke(collator, delegator)
		}

		#[pallet::call_index(25)]
		#[pallet::weight(< T as Config >::WeightInfo::delegator_bond_more())]
		/// Bond more for delegators wrt a specific collator candidate.
		pub fn delegator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let in_top = Self::delegation_bond_more_without_event(
				delegator.clone(),
				candidate.clone(),
				more,
			)?;
			Pallet::<T>::deposit_event(Event::DelegationIncreased {
				delegator,
				candidate,
				amount: more,
				in_top,
			});

			Ok(().into())
		}

		#[pallet::call_index(26)]
		#[pallet::weight(< T as Config >::WeightInfo::schedule_delegator_bond_less())]
		/// Request bond less for delegators wrt a specific collator candidate.
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_schedule_bond_decrease(candidate, delegator, less)
		}

		#[pallet::call_index(27)]
		#[pallet::weight(< T as Config >::WeightInfo::execute_delegator_bond_less())]
		/// Execute pending request to change an existing delegation
		pub fn execute_delegation_request(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward caller if caller != delegator
			Self::delegation_execute_scheduled_request(candidate, delegator)
		}

		#[pallet::call_index(28)]
		#[pallet::weight(< T as Config >::WeightInfo::cancel_delegator_bond_less())]
		/// Cancel request to change an existing delegation.
		pub fn cancel_delegation_request(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_cancel_request(candidate, delegator)
		}

		/// Sets the auto-compounding reward percentage for a delegation.
		#[pallet::call_index(29)]
		#[pallet::weight(<T as Config>::WeightInfo::set_auto_compound(
			// Max CandidateAutoCompoundingDelegationCount
			<T as Config>::MaxDelegationsPerDelegator::get(),
			// Max DelegationCount
			<T as Config>::MaxDelegationsPerDelegator::get(),
		))]
		pub fn set_auto_compound(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			value: Percent,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::set_auto_compound(candidate, delegator, value)
		}
	}
	impl<T: Config> Pallet<T> {
		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}
		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CandidateInfo<T>>::get(acc).is_some()
		}

		pub fn is_white_list(acc: &T::AccountId) -> bool {
			if <Candidates<T>>::get().binary_search(acc).is_err() {
				return false
			}
			true
		}
		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}
		/// Returns an account's free balance which is not locked in delegation staking
		pub fn get_delegator_stakable_free_balance(acc: &T::AccountId) -> BalanceOf<T> {
			T::Currency::free_balance(acc)
		}
		/// Returns an account's free balance which is not locked in collator staking
		pub fn get_collator_stakable_free_balance(acc: &T::AccountId) -> BalanceOf<T> {
			T::Currency::free_balance(acc)
		}
		/// Returns a delegations auto-compound value.
		pub fn delegation_auto_compound(
			candidate: &T::AccountId,
			delegator: &T::AccountId,
		) -> Percent {
			<AutoCompoundDelegations<T>>::auto_compound(candidate, delegator)
		}
		///???
		/// Caller must ensure candidate is active before calling
		pub(crate) fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.remove(&Bond::from_owner(candidate.clone()));
			candidates.insert(Bond { owner: candidate, amount: total });
			<CandidatePool<T>>::put(candidates);
		}
		/// Compute round issuance based on total staked for the given round
		fn compute_issuance(staked: BalanceOf<T>) -> BalanceOf<T> {
			let config = <InflationConfig<T>>::get();
			let round_issuance = crate::inflation::round_issuance_range::<T>(config.round);
			// TODO: consider interpolation instead of bounded range
			if staked < config.expect.min {
				round_issuance.min
			} else if staked > config.expect.max {
				round_issuance.max
			} else {
				round_issuance.ideal
			}
		}
		/// Remove delegation from candidate state
		/// Amount input should be retrieved from delegator and it informs the storage lookups
		pub(crate) fn delegator_leaves_candidate(
			candidate: T::AccountId,
			delegator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let mut state = <CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			state.rm_delegation_if_exists::<T>(&candidate, delegator.clone(), amount)?;
			T::Currency::unreserve(&delegator, amount);
			let new_total_locked = <Total<T>>::get().saturating_sub(amount);
			<Total<T>>::put(new_total_locked);
			let new_total = state.total_counted;
			<CandidateInfo<T>>::insert(&candidate, state);
			Self::deposit_event(Event::DelegatorLeftCandidate {
				delegator,
				candidate,
				unstaked_amount: amount,
				total_candidate_staked: new_total,
			});
			Ok(())
		}
		fn prepare_staking_payouts(now: RoundIndex) {
			// payout is now - delay rounds ago => now - delay > 0 else return early
			let delay = T::RewardPaymentDelay::get();
			if now <= delay {
				return
			}
			let round_to_payout = now.saturating_sub(delay);
			let total_points = <Points<T>>::get(round_to_payout);
			if total_points.is_zero() {
				return
			}
			let total_staked = <Staked<T>>::take(round_to_payout);
			let total_issuance = Self::compute_issuance(total_staked);
			let mut left_issuance = total_issuance;
			// reserve portion of issuance for parachain bond account
			let bond_config = <ParachainBondInfo<T>>::get();
			let parachain_bond_reserve = bond_config.percent * total_issuance;
			if let Ok(imb) =
				T::Currency::deposit_into_existing(&bond_config.account, parachain_bond_reserve)
			{
				// update round issuance iff transfer succeeds
				left_issuance = left_issuance.saturating_sub(imb.peek());
				Self::deposit_event(Event::ReservedForParachainBond {
					account: bond_config.account,
					value: imb.peek(),
				});
			}

			let payout = DelayedPayout {
				round_issuance: total_issuance,
				total_staking_reward: left_issuance,
				collator_commission: <CollatorCommission<T>>::get(),
			};

			<DelayedPayouts<T>>::insert(round_to_payout, payout);
		}

		/// Wrapper around pay_one_collator_reward which handles the following logic:
		/// * whether or not a payout needs to be made
		/// * cleaning up when payouts are done
		/// * returns the weight consumed by pay_one_collator_reward if applicable
		fn handle_delayed_payouts(now: RoundIndex) -> Weight {
			let delay = T::RewardPaymentDelay::get();

			// don't underflow uint
			if now < delay {
				return Weight::zero()
			}

			let paid_for_round = now.saturating_sub(delay);

			if let Some(payout_info) = <DelayedPayouts<T>>::get(paid_for_round) {
				let result = Self::pay_one_collator_reward(paid_for_round, payout_info);
				if result.0.is_none() {
					// result.0 indicates whether or not a payout was made
					// clean up storage items that we no longer need
					<DelayedPayouts<T>>::remove(paid_for_round);
					<Points<T>>::remove(paid_for_round);
				}
				result.1 // weight consumed by pay_one_collator_reward
			} else {
				Weight::zero()
			}
		}

		/// Payout a single collator from the given round.
		///
		/// Returns an optional tuple of (Collator's AccountId, total paid)
		/// or None if there were no more payouts to be made for the round.
		pub(crate) fn pay_one_collator_reward(
			paid_for_round: RoundIndex,
			payout_info: DelayedPayout<BalanceOf<T>>,
		) -> (Option<(T::AccountId, BalanceOf<T>)>, Weight) {
			// 'early_weight' tracks weight used for reads/writes done early in this fn before its
			// early-exit codepaths.
			let mut early_weight = Weight::zero();

			// TODO: it would probably be optimal to roll Points into the DelayedPayouts storage
			// item so that we do fewer reads each block
			let total_points = <Points<T>>::get(paid_for_round);
			early_weight = early_weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
			if total_points.is_zero() {
				// TODO: this case is obnoxious... it's a value query, so it could mean one of two
				// different logic errors:
				// 1. we removed it before we should have
				// 2. we called pay_one_collator_reward when we were actually done with deferred
				//    payouts
				log::warn!("pay_one_collator_reward called with no <Points<T>> for the round!");
				return (None, early_weight)
			}

			let collator_fee = payout_info.collator_commission;
			let collator_issuance = collator_fee * payout_info.round_issuance;

			if let Some((collator, pts)) =
				<AwardedPts<T>>::iter_prefix(paid_for_round).drain().next()
			{
				let mut extra_weight = Weight::zero();
				let pct_due = Perbill::from_rational(pts, total_points);
				let total_paid = pct_due * payout_info.total_staking_reward;
				let mut amt_due = total_paid;
				// Take the snapshot of block author and delegations
				let state = <AtStake<T>>::take(paid_for_round, &collator);
				let num_delegators = state.delegations.len();
				if state.delegations.is_empty() {
					// solo collator with no delegators
					Self::mint(amt_due, collator.clone());
					extra_weight =
						extra_weight.saturating_add(T::OnCollatorPayout::on_collator_payout(
							paid_for_round,
							collator.clone(),
							amt_due,
						));
				} else {
					// pay collator first; commission + due_portion
					let collator_pct = Perbill::from_rational(state.bond, state.total);
					let commission = pct_due * collator_issuance;
					amt_due = amt_due.saturating_sub(commission);
					let collator_reward = (collator_pct * amt_due).saturating_add(commission);
					Self::mint(collator_reward, collator.clone());
					extra_weight =
						extra_weight.saturating_add(T::OnCollatorPayout::on_collator_payout(
							paid_for_round,
							collator.clone(),
							collator_reward,
						));
					// pay delegators due portion
					for BondWithAutoCompound { owner, amount, auto_compound } in state.delegations {
						let percent = Perbill::from_rational(amount, state.total);
						let due = percent * amt_due;
						if !due.is_zero() {
							Self::mint_and_compound(
								due,
								auto_compound,
								collator.clone(),
								owner.clone(),
							);
						}
					}
				}

				(
					Some((collator, total_paid)),
					T::WeightInfo::pay_one_collator_reward(num_delegators as u32)
						.saturating_add(extra_weight),
				)
			} else {
				// Note that we don't clean up storage here; it is cleaned up in
				// handle_delayed_payouts()
				(None, Weight::zero())
			}
		}

		/// Compute the top `TotalSelected` candidates in the CandidatePool and return
		/// a vec of their AccountIds (in the order of selection)
		pub fn compute_top_candidates() -> Vec<T::AccountId> {
			let mut candidates = <CandidatePool<T>>::get().0;
			// order candidates by stake (least to greatest so requires `rev()`)
			candidates.sort_by(|a, b| a.amount.cmp(&b.amount));
			let top_n = <TotalSelected<T>>::get() as usize;
			// choose the top TotalSelected qualified candidates, ordered by stake
			let mut collators = candidates
				.into_iter()
				.rev()
				.take(top_n)
				.filter(|x| x.amount >= T::MinCollatorStk::get())
				.map(|x| x.owner)
				.collect::<Vec<T::AccountId>>();
			collators.sort();
			collators
		}
		/// Best as in most cumulatively supported in terms of stake
		/// Returns [collator_count, delegation_count, total staked]
		fn select_top_candidates(now: RoundIndex) -> (u32, u32, BalanceOf<T>) {
			let (mut collator_count, mut delegation_count, mut total) =
				(0u32, 0u32, BalanceOf::<T>::zero());
			// choose the top TotalSelected qualified candidates, ordered by stake
			let collators = Self::compute_top_candidates();
			if collators.is_empty() {
				// SELECTION FAILED TO SELECT >=1 COLLATOR => select collators from previous round
				let last_round = now.saturating_sub(1u32);
				let mut total_per_candidate: BTreeMap<T::AccountId, BalanceOf<T>> = BTreeMap::new();
				// set this round AtStake to last round AtStake
				for (account, snapshot) in <AtStake<T>>::iter_prefix(last_round) {
					collator_count = collator_count.saturating_add(1u32);
					delegation_count =
						delegation_count.saturating_add(snapshot.delegations.len() as u32);
					total = total.saturating_add(snapshot.total);
					total_per_candidate.insert(account.clone(), snapshot.total);
					<AtStake<T>>::insert(now, account, snapshot);
				}
				// `SelectedCandidates` remains unchanged from last round
				// emit CollatorChosen event for tools that use this event
				for candidate in <SelectedCandidates<T>>::get() {
					let snapshot_total = total_per_candidate
						.get(&candidate)
						.expect("all selected candidates have snapshots");
					Self::deposit_event(Event::CollatorChosen {
						round: now,
						collator_account: candidate,
						total_exposed_amount: *snapshot_total,
					})
				}
				return (collator_count, delegation_count, total)
			}

			// snapshot exposure for round for weighting reward distribution
			for account in collators.iter() {
				let state = <CandidateInfo<T>>::get(account)
					.expect("all members of CandidateQ must be candidates");

				collator_count = collator_count.saturating_add(1u32);
				delegation_count = delegation_count.saturating_add(state.delegation_count);
				total = total.saturating_add(state.total_counted);
				let CountedDelegations { uncounted_stake, rewardable_delegations } =
					Self::get_rewardable_delegators(&account);
				let total_counted = state.total_counted.saturating_sub(uncounted_stake);

				let auto_compounding_delegations = <AutoCompoundingDelegations<T>>::get(&account)
					.into_iter()
					.map(|x| (x.delegator, x.value))
					.collect::<BTreeMap<_, _>>();
				let rewardable_delegations = rewardable_delegations
					.into_iter()
					.map(|d| BondWithAutoCompound {
						owner: d.owner.clone(),
						amount: d.amount,
						auto_compound: auto_compounding_delegations
							.get(&d.owner)
							.cloned()
							.unwrap_or_else(Percent::zero),
					})
					.collect();

				let snapshot = CollatorSnapshot {
					bond: state.bond,
					delegations: rewardable_delegations,
					total: total_counted,
				};
				<AtStake<T>>::insert(now, account, snapshot);
				Self::deposit_event(Event::CollatorChosen {
					round: now,
					collator_account: account.clone(),
					total_exposed_amount: state.total_counted,
				});
			}
			// insert canonical collator set
			<SelectedCandidates<T>>::put(collators);
			(collator_count, delegation_count, total)
		}

		/// Apply the delegator intent for revoke and decrease in order to build the
		/// effective list of delegators with their intended bond amount.
		///
		/// This will:
		/// - if [DelegationAction::Revoke] is outstanding, set the bond amount to 0.
		/// - if [DelegationAction::Decrease] is outstanding, subtract the bond by specified amount.
		/// - else, do nothing
		///
		/// The intended bond amounts will be used while calculating rewards.
		fn get_rewardable_delegators(collator: &T::AccountId) -> CountedDelegations<T> {
			let requests = <DelegationScheduledRequests<T>>::get(collator)
				.into_iter()
				.map(|x| (x.delegator, x.action))
				.collect::<BTreeMap<_, _>>();

			let mut uncounted_stake = BalanceOf::<T>::zero();
			let rewardable_delegations = <TopDelegations<T>>::get(collator)
				.expect("all members of CandidateQ must be candidates")
				.delegations
				.into_iter()
				.map(|mut bond| {
					bond.amount = match requests.get(&bond.owner) {
						None => bond.amount,
						Some(DelegationAction::Revoke(_)) => {
							uncounted_stake = uncounted_stake.saturating_add(bond.amount);
							BalanceOf::<T>::zero()
						},
						Some(DelegationAction::Decrease(amount)) => {
							uncounted_stake = uncounted_stake.saturating_add(*amount);
							bond.amount.saturating_sub(*amount)
						},
					};

					bond
				})
				.collect();
			CountedDelegations { uncounted_stake, rewardable_delegations }
		}

		/// This function exists as a helper to delegator_bond_more & auto_compound functionality.
		/// Any changes to this function must align with both user-initiated bond increases and
		/// auto-compounding bond increases.
		/// Any feature-specific preconditions should be validated before this function is invoked.
		/// Any feature-specific events must be emitted after this function is invoked.
		pub fn delegation_bond_more_without_event(
			delegator: T::AccountId,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> Result<bool, sp_runtime::DispatchError> {
			ensure!(
				!Self::delegation_request_revoke_exists(&candidate, &delegator),
				Error::<T>::PendingDelegationRevoke
			);
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			state.increase_delegation::<T>(candidate, more)
		}

		/// Mint a specified reward amount to the beneficiary account. Emits the [Rewarded] event.
		fn mint(amt: BalanceOf<T>, to: T::AccountId) {
			if let Ok(amount_transferred) = T::Currency::deposit_into_existing(&to, amt) {
				Self::deposit_event(Event::Rewarded {
					account: to.clone(),
					rewards: amount_transferred.peek(),
				});
			}
		}

		/// Mint and compound delegation rewards. The function mints the amount towards the
		/// delegator and tries to compound a specified percent of it back towards the delegation.
		/// If a scheduled delegation revoke exists, then the amount is only minted, and nothing is
		/// compounded. Emits the [Compounded] event.
		fn mint_and_compound(
			amt: BalanceOf<T>,
			compound_percent: Percent,
			candidate: T::AccountId,
			delegator: T::AccountId,
		) {
			if let Ok(amount_transferred) = T::Currency::deposit_into_existing(&delegator, amt) {
				Self::deposit_event(Event::Rewarded {
					account: delegator.clone(),
					rewards: amount_transferred.peek(),
				});

				let compound_amount = compound_percent.mul_ceil(amount_transferred.peek());
				if compound_amount.is_zero() {
					return
				}

				if let Err(err) = Self::delegation_bond_more_without_event(
					delegator.clone(),
					candidate.clone(),
					compound_amount,
				) {
					log::error!(
								"Error compounding staking reward towards candidate '{:?}' for delegator '{:?}': {:?}",
								candidate,
								delegator,
								err
							);
					return
				};

				Pallet::<T>::deposit_event(Event::Compounded {
					delegator,
					candidate,
					amount: compound_amount,
				});
			};
		}
	}

	/// Add reward points to block authors:
	/// * 20 points to the block producer for producing a block in the chain
	impl<T: Config> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get().current;
			let score_plus_20 = <AwardedPts<T>>::get(now, &author).saturating_add(20);
			<AwardedPts<T>>::insert(now, author, score_plus_20);
			<Points<T>>::mutate(now, |x| *x = x.saturating_add(20));
		}
	}

	// impl<T: Config> pallet_authorship::CanAuthor<T::AccountId> for Pallet<T> {
	// 	fn can_author(account: &T::AccountId, _slot: &u32) -> bool {
	// 		Self::is_selected_candidate(account)
	// 	}
	// }

	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			Self::selected_candidates()
		}
	}

	// Adpation for traditional substrate authorship system
	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log::debug!(
				"assembling new collators for new session {} at #{:?}",
				new_index,
				<frame_system::Pallet<T>>::block_number(),
			);

			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::DbWeight::get().reads(2),
				DispatchClass::Mandatory,
			);

			let collators = Self::compute_top_candidates();
			if collators.is_empty() {
				// we never want to pass an empty set of collators. This would brick the chain.
				log::error!("💥 keeping old session because of empty collator set!");
				None
			} else {
				Some(collators)
			}
		}

		fn end_session(_end_index: SessionIndex) {
			// we too are not caring.
		}

		fn start_session(_start_index: SessionIndex) {
			// we too are not caring.
		}
	}

	// Adpation for traditional substrate authorship system
	impl<T: Config> pallet_session::ShouldEndSession<T::BlockNumber> for Pallet<T> {
		fn should_end_session(now: T::BlockNumber) -> bool {
			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::DbWeight::get().reads(2),
				DispatchClass::Mandatory,
			);

			let round = <Round<T>>::get();
			round.should_update(now)
		}
	}

	impl<T: Config> EstimateNextSessionRotation<T::BlockNumber> for Pallet<T> {
		fn average_session_length() -> T::BlockNumber {
			<Round<T>>::get().length.into()
		}

		fn estimate_current_session_progress(now: T::BlockNumber) -> (Option<Permill>, Weight) {
			let round = <Round<T>>::get();
			let passed_blocks = now.saturating_sub(round.first);

			(
				Some(Permill::from_rational(passed_blocks, round.length.into())),
				// One read for the round info, blocknumber is read free
				T::DbWeight::get().reads(1),
			)
		}

		fn estimate_next_session_rotation(
			_now: T::BlockNumber,
		) -> (Option<T::BlockNumber>, Weight) {
			let round = <Round<T>>::get();

			(
				Some(round.first + round.length.into()),
				// One read for the round info, blocknumber is read free
				T::DbWeight::get().reads(1),
			)
		}
	}
}
