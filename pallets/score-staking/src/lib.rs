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

//! A pallet served as a staking pool separated from `pallet-parachain-staking`
//!
//! Once started, the reward is calculated every `RoundInterval` blocks, the
//! snapshot of user state at that time will be used to calculate rewards.
//!
//! The yearly total issuance of this pool = YearlyIssuance * YearlyInflation,
//! based on it and `RoundInterval`, the reward per round can be calculated.
//!
//! The scores come from external origin (e.g. IDHub), upon updating the scores
//! the staked amount in pallet-parachain-staking is checked: users without any
//! staking will **NOT** be recorded.
//!
//! Then the round reward for a specific user is calculated by:
//!
//! total_round_rewards *
//!     (score_coefficient * S(i) + stake_coefficient * T(i))
//!  /  (score_coefficient * S(a) + stake_coefficient * T(a))
//!
//! , where
//! S(i): the score of this user
//! T(i): the staked amount of this user
//! S(a): the total scores of all accounts in `Scores` storage
//! T(a): the total staked amount of all users, not only those in `Scores` storage

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use core_primitives::{DAYS, YEARS};
use frame_support::traits::Currency;
use pallet_parachain_staking as ParaStaking;
use sp_core::crypto::AccountId32;
use sp_runtime::{traits::CheckedSub, Perbill};
use sp_std::marker::PhantomData;

pub use pallet::*;

mod types;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// Need this to convert hardcoded `AccountId32` to T::AccountId
pub trait AccountIdConvert<T: Config> {
	fn convert(account: AccountId32) -> T::AccountId;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use core_primitives::Identity;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{Imbalance, LockableCurrency, ReservableCurrency, StorageVersion},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Zero;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + ParaStaking::Config {
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type YearlyIssuance: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type YearlyInflation: Get<Perbill>;
		/// The origin who manages this pallet
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// AccountId converter
		type AccountIdConvert: AccountIdConvert<Self>;
	}

	#[pallet::error]
	pub enum Error<T> {
		// unthorized origin
		UnauthorizedOrigin,
		// the user account doesn't have an entry in parachain-staking
		UserNotStaked,
		// the user account has an entry but the total staked amount is (somehow) zero
		UserStakedAmountZero,
		// the user account doesn't exist in the registry
		UserNotExist,
		// convert `Identity` to substrate account failed
		ConvertIdentityFailed,
		// pool is not in running state
		PoolNotRun,
		// pool is already in running state
		PoolAlreadyRunning,
		// round index overflow
		RoundIndexOverflow,
		// the user claims more than what he has
		InsufficientBalance,
		// balance underflow
		BalanceUnderflow,
		// block number can't be converted to u32
		BlockNumberConvertError,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolStarted { start_block: BlockNumberFor<T> },
		PoolStopped {},
		ScoreFeederSet { new_score_feeder: Option<T::AccountId> },
		RoundConfigSet { new_config: RoundSetting },
		ScoreUpdated { who: Identity, new_score: Score },
		ScoreRemoved { who: Identity },
		ScoreCleared {},
		RewardCalculated { total: BalanceOf<T>, distributed: BalanceOf<T> },
		RewardClaimed { who: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::storage]
	#[pallet::getter(fn score_feeder)]
	pub type ScoreFeeder<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	pub type Round<T: Config> = StorageValue<_, RoundInfo<BlockNumberFor<T>>, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultRoundSetting<T: Config>() -> RoundSetting {
		RoundSetting {
			interval: DEFAULT_ROUND_INTERVAL,
			score_coefficient: DEFAULT_SCORE_COEFFICIENT,
			stake_coefficient: DEFAULT_STAKE_COEFFICIENT,
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn round_config)]
	pub type RoundConfig<T: Config> =
		StorageValue<_, RoundSetting, ValueQuery, DefaultRoundSetting<T>>;

	// use `Twox64Concat` and `T::AccountId` for faster and shorter storage
	// we might have tens of thousands of entries
	#[pallet::storage]
	#[pallet::getter(fn scores)]
	pub type Scores<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, ScorePayment<BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn state)]
	pub type State<T: Config> = StorageValue<_, PoolState, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub state: PoolState,
		pub marker: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { state: PoolState::Stopped, marker: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			State::<T>::put(self.state);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 0); // Self::state()

			if Self::state() == PoolState::Stopped {
				return weight
			}

			let mut r = Round::<T>::get();
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

			if !is_modulo(now - r.start_block, Self::round_config().interval.into()) {
				// nothing to do there
				return weight
			}

			// We are about to start a new round
			// 1. update round info
			r.index = r.index.saturating_add(1);
			r.start_block = now;
			Round::<T>::put(r);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

			// 2. calculate payout
			let round_reward: BalanceOf<T> = (T::YearlyInflation::get() * T::YearlyIssuance::get() /
				YEARS.into()) * Self::round_config().interval.into();

			let total_stake = ParaStaking::Pallet::<T>::total();
			let total_score =
				Scores::<T>::iter_values().fold(0u32, |a, b| a.checked_add(b.score).unwrap_or(a));
			let score_coef = Self::round_config().score_coefficient;
			let stake_coef = Self::round_config().stake_coefficient;

			let mut total_user_reward = BalanceOf::<T>::zero();

			for (a, mut p) in Scores::<T>::iter() {
				let user_stake = ParaStaking::Pallet::<T>::delegator_state(&a)
					.map(|s| s.total)
					.unwrap_or_default();
				let user_reward: BalanceOf<T> = Perbill::from_rational(
					stake_coef * user_stake +
						score_coef * <u32 as Into<ParaStaking::BalanceOf<T>>>::into(p.score),
					stake_coef * total_stake +
						score_coef * <u32 as Into<ParaStaking::BalanceOf<T>>>::into(total_score),
				) * round_reward;

				p.unpaid += user_reward;
				total_user_reward += user_reward;
				Scores::<T>::insert(&a, p);
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 1));
			}

			Self::deposit_event(Event::<T>::RewardCalculated {
				total: round_reward,
				distributed: total_user_reward,
			});

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal))]
		pub fn set_score_feeder(
			origin: OriginFor<T>,
			new_score_feeder: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::AdminOrigin::ensure_origin(origin)?;
			ScoreFeeder::<T>::put(new_score_feeder.clone());
			Self::deposit_event(Event::ScoreFeederSet { new_score_feeder: Some(new_score_feeder) });
			Ok(Pays::No.into())
		}

		/// Start (or restart) a currently stopped pool
		///
		/// It:
		/// - sets the RoundInfo.start_block to the current block number
		/// - advances the round index
		#[pallet::call_index(1)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn start_pool(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			T::AdminOrigin::ensure_origin(origin)?;
			ensure!(Self::state() == PoolState::Stopped, Error::<T>::PoolAlreadyRunning);
			State::<T>::put(PoolState::Running);
			let start_block = frame_system::Pallet::<T>::block_number();
			let mut r = Round::<T>::take();
			r.index = r.index.checked_add(1).ok_or(Error::<T>::RoundIndexOverflow)?;
			r.start_block = start_block;
			Round::<T>::put(r);
			Self::deposit_event(Event::PoolStarted { start_block });
			Ok(Pays::No.into())
		}

		/// Stop a currently running pool, should be called as caution
		#[pallet::call_index(2)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn stop_pool(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			T::AdminOrigin::ensure_origin(origin)?;
			ensure!(Self::state() == PoolState::Running, Error::<T>::PoolNotRun);
			State::<T>::put(PoolState::Stopped);
			Self::deposit_event(Event::PoolStopped {});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn set_round_config(
			origin: OriginFor<T>,
			config: RoundSetting,
		) -> DispatchResultWithPostInfo {
			T::AdminOrigin::ensure_origin(origin)?;
			RoundConfig::<T>::put(config);
			Self::deposit_event(Event::RoundConfigSet { new_config: config });
			Ok(Pays::No.into())
		}

		// Intentionally use `Identity` type to lower the hurdle of mapping to the
		// desired substrate account as it's handled on-chain instead of by client.
		//
		// Subject to requirement change though
		#[pallet::call_index(4)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn update_score(
			origin: OriginFor<T>,
			user: Identity,
			score: Score,
		) -> DispatchResultWithPostInfo {
			ensure!(
				Some(ensure_signed(origin)?) == Self::score_feeder(),
				Error::<T>::UnauthorizedOrigin
			);
			let account = T::AccountIdConvert::convert(
				user.to_account_id().ok_or(Error::<T>::ConvertIdentityFailed)?,
			);
			Scores::<T>::try_mutate(&account, |payment| {
				let state = ParaStaking::Pallet::<T>::delegator_state(&account)
					.ok_or(Error::<T>::UserNotStaked)?;
				ensure!(state.total > 0u32.into(), Error::<T>::UserStakedAmountZero);

				match payment {
					Some(s) => {
						s.score = score;
						*payment = Some(*s);
					},
					None => *payment = Some(ScorePayment { score, unpaid: 0u32.into() }),
				}
				Ok::<(), Error<T>>(())
			})?;
			Self::deposit_event(Event::ScoreUpdated { who: user, new_score: score });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn remove_score(origin: OriginFor<T>, user: Identity) -> DispatchResultWithPostInfo {
			// only admin can remove entries in `Scores`
			T::AdminOrigin::ensure_origin(origin)?;
			let account = T::AccountIdConvert::convert(
				user.to_account_id().ok_or(Error::<T>::ConvertIdentityFailed)?,
			);
			ensure!(Scores::<T>::contains_key(&account), Error::<T>::UserNotExist);
			Scores::<T>::remove(&account);
			Self::deposit_event(Event::ScoreRemoved { who: user });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn clear_score(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// only admin can clear all entries in `Scores`
			T::AdminOrigin::ensure_origin(origin)?;
			let _ = Scores::<T>::clear(u32::MAX, None);
			Self::deposit_event(Event::ScoreCleared {});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn claim(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			Scores::<T>::try_mutate(&account, |payment| {
				let mut p = payment.take().ok_or(Error::<T>::UserNotExist)?;
				ensure!(amount <= p.unpaid, Error::<T>::InsufficientBalance);
				let rewarded =
					<T as pallet::Config>::Currency::deposit_into_existing(&account, amount)?
						.peek();
				p.unpaid = p.unpaid.checked_sub(&rewarded).ok_or(Error::<T>::BalanceUnderflow)?;
				*payment = Some(p);
				Self::deposit_event(Event::RewardClaimed {
					who: account.clone(),
					amount: rewarded,
				});
				Ok(().into())
			})
		}

		#[pallet::call_index(8)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn claim_all(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin.clone())?;
			let payment = Scores::<T>::get(&account).ok_or(Error::<T>::UserNotExist)?;
			Self::claim(origin, payment.unpaid)
		}
	}
}

fn is_modulo<BlockNumber: PartialEq + From<u32> + sp_std::ops::Rem<Output = BlockNumber>>(
	dividend: BlockNumber,
	divisor: BlockNumber,
) -> bool {
	dividend % divisor == BlockNumber::from(0u32)
}
