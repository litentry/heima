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
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::{
			fungible::{Inspect as FInspect, Mutate as FMutate},
			fungibles::{Create as FsCreate, Inspect as FsInspect, Mutate as FsMutate},
			Fortitude, Precision, Preservation,
		},
		StorageVersion,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub,
		One, Zero,
	},
	ArithmeticError, Perquintill, Saturating,
};
use sp_std::{collections::vec_deque::VecDeque, fmt::Debug, prelude::*};

use pallet_collab_ai_common::*;

#[derive(PartialEq, Eq, Clone, Encode, Debug, Decode, TypeInfo)]
pub struct InvestingWeightInfo<BlockNumber, Balance> {
	// For a single position or
	// Synthetic overall average effective_time weighted by staked amount
	pub effective_time: BlockNumber,
	// Staked amount
	pub amount: Balance,
	// This is recorded for not allowing weight calculation when time < some of history effective
	// time
	pub last_add_time: BlockNumber,
}

impl<BlockNumber, Balance> InvestingWeightInfo<BlockNumber, Balance>
where
	Balance: AtLeast32BitUnsigned + Copy,
	BlockNumber: AtLeast32BitUnsigned + Copy,
{
	// Mixing a new added investing position, replace the checkpoint with Synthetic new one
	// Notice: The logic will be wrong if weight calculated time is before any single added
	// effective_time
	// None means TypeIncompatible Or Overflow Or Division Zero
	fn add(&mut self, effective_time: BlockNumber, amount: Balance) -> Option<()> {
		// If last_add_time always > effective_time, only new added effective time can effect
		// last_add_time
		self.last_add_time = self.last_add_time.max(effective_time);

		// We try force all types into u128, then convert it back
		let e: u128 = effective_time.try_into().ok()?;
		let s: u128 = amount.try_into().ok()?;

		let oe: u128 = self.effective_time.try_into().ok()?;
		let os: u128 = self.amount.try_into().ok()?;

		let new_amount: u128 = os.checked_add(s)?;
		// (oe * os + e * s) / (os + s)
		let new_effective_time: u128 =
			(oe.checked_mul(os)?.checked_add(e.checked_mul(s)?)?).checked_div(new_amount)?;
		self.amount = new_amount.try_into().ok()?;
		self.effective_time = new_effective_time.try_into().ok()?;
		Some(())
	}

	// Mixing a new investing position removed, replace the checkpoint with Synthetic new one
	// Notice: The logic will be wrong if weight calculated time is before any single added
	// effective_time
	// None means TypeIncompatible Or Overflow Or Division Zero
	fn remove(&mut self, effective_time: BlockNumber, amount: Balance) -> Option<()> {
		// We try force all types into u128, then convert it back
		let e: u128 = effective_time.try_into().ok()?;
		let s: u128 = amount.try_into().ok()?;

		let oe: u128 = self.effective_time.try_into().ok()?;
		let os: u128 = self.amount.try_into().ok()?;

		let new_amount: u128 = os.checked_sub(s)?;
		// (oe * os - e * s) / (os - s)
		let new_effective_time: u128 =
			(oe.checked_mul(os)?.checked_sub(e.checked_mul(s)?)?).checked_div(new_amount)?;
		self.amount = new_amount.try_into().ok()?;
		self.effective_time = new_effective_time.try_into().ok()?;
		Some(())
	}

	// Claim/Update weighted info based on target until-block and return the consumed weight
	// None means TypeIncompatible Or Overflow
	fn claim(&mut self, n: BlockNumber) -> Option<u128> {
		// Claim time before last_add_time is not allowed, since weight can not be calculated
		let weight = self.weight(n)?;
		self.effective_time = n;

		Some(weight)
	}

	// consume corresponding weight, change effective time without changing staked amount, return
	// the changed effective time
	// This function is mostly used for Synthetic checkpoint change
	// None means TypeIncompatible Or Division Zero
	fn claim_based_on_weight(&mut self, weight: u128) -> Option<BlockNumber> {
		let oe: u128 = self.effective_time.try_into().ok()?;
		let os: u128 = self.amount.try_into().ok()?;

		let delta_e: u128 = weight.checked_div(os)?;
		let new_effective_time: BlockNumber = (oe + delta_e).try_into().ok()?;
		self.effective_time = new_effective_time;

		Some(new_effective_time)
	}

	// Withdraw investing amount and return the amount after withdrawal
	// None means underflow
	fn withdraw(&mut self, v: Balance) -> Option<Balance> {
		self.amount = self.amount.checked_sub(&v)?;

		Some(self.amount)
	}

	// You should never use n < any single effective_time
	// it only works for n > all effective_time
	// None means TypeIncompatible Or Overflow
	fn weight(&self, n: BlockNumber) -> Option<u128> {
		// Estimate weight before last_add_time can be biased so not allowed
		if self.last_add_time > n {
			return None;
		}

		let e: u128 = n.checked_sub(&self.effective_time)?.try_into().ok()?;
		let s: u128 = self.amount.try_into().ok()?;
		e.checked_mul(s)
	}

	// Force estimate weight regardless
	// None means TypeIncompatible Or Overflow
	fn weight_force(&self, n: BlockNumber) -> Option<u128> {
		let e: u128 = n.checked_sub(&self.effective_time)?.try_into().ok()?;
		let s: u128 = self.amount.try_into().ok()?;
		e.checked_mul(s)
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Debug, Decode, TypeInfo)]
pub struct PoolSetting<AccountId, BlockNumber, Balance> {
	// The start time of investing pool
	pub start_time: BlockNumber,
	// How many epoch will investing pool last, n > 0, valid epoch index :[0..n)
	pub epoch: u128,
	// How many blocks each epoch consist
	pub epoch_range: BlockNumber,
	// Max staked amount of pool
	pub pool_cap: Balance,
	// Curator
	pub admin: AccountId,
}

impl<AccountId, BlockNumber, Balance> PoolSetting<AccountId, BlockNumber, Balance>
where
	Balance: AtLeast32BitUnsigned + Copy,
	BlockNumber: AtLeast32BitUnsigned + Copy,
{
	// None means TypeIncompatible Or Overflow
	fn end_time(&self) -> Option<BlockNumber> {
		let er: u128 = self.epoch_range.try_into().ok()?;
		let st: u128 = self.start_time.try_into().ok()?;
		let result = st.checked_add(er.checked_mul(self.epoch)?)?;
		result.try_into().ok()
	}
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::transactional;

	use super::*;

	pub type BalanceOf<T> =
		<<T as Config>::Fungibles as FsInspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub type AssetIdOf<T> =
		<<T as Config>::Fungibles as FsInspect<<T as frame_system::Config>::AccountId>>::AssetId;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Pool proposal pallet origin used to start an investing pool
		type PoolProposalPalletOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin used to update epoch reward for investing pool
		type RewardUpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin used to administer the investing pool
		type InvestingPoolAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type Fungibles: FsMutate<Self::AccountId> + FsCreate<Self::AccountId>;

		/// The beneficiary PalletId, used fro deriving its sovereign account to hold assets of reward
		#[pallet::constant]
		type StableTokenBeneficiaryId: Get<PalletId>;

		/// The beneficiary PalletId, used for deriving its sovereign AccountId for providing native
		/// token reward
		#[pallet::constant]
		type CANBeneficiaryId: Get<PalletId>;
	}

	// Setting of investing pools
	#[pallet::storage]
	#[pallet::getter(fn investing_pool_setting)]
	pub type InvestingPoolSetting<T: Config> = StorageMap<
		_,
		Twox64Concat,
		InvestingPoolIndex,
		PoolSetting<T::AccountId, BlockNumberFor<T>, BalanceOf<T>>,
		OptionQuery,
	>;

	// investing pools' stable token reward waiting claiming
	// Pool id, epcoh index => epoch reward
	#[pallet::storage]
	#[pallet::getter(fn stable_investing_pool_epoch_reward)]
	pub type StableInvestingPoolEpochReward<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		InvestingPoolIndex,
		Twox64Concat,
		u128,
		BalanceOf<T>,
		OptionQuery,
	>;

	// Checkpoint of single stable staking pool
	// For stable token reward distribution
	#[pallet::storage]
	#[pallet::getter(fn stable_investing_pool_checkpoint)]
	pub type StableInvestingPoolCheckpoint<T: Config> = StorageMap<
		_,
		Twox64Concat,
		InvestingPoolIndex,
		InvestingWeightInfo<BlockNumberFor<T>, BalanceOf<T>>,
		OptionQuery,
	>;

	// Checkpoint of overall investing condition synthetic by tracking all investing pools
	// For CAN token reward distribution
	#[pallet::storage]
	#[pallet::getter(fn can_checkpoint)]
	pub type CANCheckpoint<T: Config> =
		StorageValue<_, InvestingWeightInfo<BlockNumberFor<T>, BalanceOf<T>>, OptionQuery>;

	// Asset id of AIUSD
	#[pallet::storage]
	#[pallet::getter(fn aiusd_asset_id)]
	pub type AIUSDAssetId<T: Config> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

	// Asset id of CAN
	#[pallet::storage]
	#[pallet::getter(fn can_asset_id)]
	pub type CANAssetId<T: Config> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InvestingPoolCreated {
			pool_id: InvestingPoolIndex,
			admin: T::AccountId,
			start_time: BlockNumberFor<T>,
			epoch: u128,
			epoch_range: BlockNumberFor<T>,
			pool_cap: BalanceOf<T>,
		},
		/// New metadata has been set for a investing pool.
		MetadataSet {
			pool_id: InvestingPoolIndex,
			name: Vec<u8>,
			description: Vec<u8>,
		},
		/// Metadata has been removed for a investing pool.
		MetadataRemoved {
			pool_id: InvestingPoolIndex,
		},
		/// Reward updated
		RewardUpdated {
			pool_id: InvestingPoolIndex,
			epoch: u128,
			amount: BalanceOf<T>,
		},
		PendingInvestingSolved {
			who: T::AccountId,
			pool_id: InvestingPoolIndex,
			effective_time: BlockNumberFor<T>,
			amount: BalanceOf<T>,
		},
		Staked {
			who: T::AccountId,
			pool_id: InvestingPoolIndex,
			target_effective_time: BlockNumberFor<T>,
			amount: BalanceOf<T>,
		},
		NativeRewardClaimed {
			who: T::AccountId,
			until_time: BlockNumberFor<T>,
			reward_amount: BalanceOf<T>,
		},
		StableRewardClaimed {
			who: T::AccountId,
			pool_id: InvestingPoolIndex,
			until_time: BlockNumberFor<T>,
			reward_amount: BalanceOf<T>,
		},
		Withdraw {
			who: T::AccountId,
			pool_id: InvestingPoolIndex,
			time: BlockNumberFor<T>,
			amount: BalanceOf<T>,
		},
		AIUSDRegisted {
			asset_id: <T::Fungibles as FsInspect<T::AccountId>>::AssetId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		RewardAlreadyExisted,
		PoolAlreadyStarted,
		PoolAlreadyEnded,
		PoolAlreadyExisted,
		PoolCapLimit,
		PoolNotEnded,
		PoolNotExisted,
		PoolNotStarted,
		BadMetadata,
		EpochAlreadyEnded,
		EpochRewardNotUpdated,
		EpochNotExist,
		NoAssetId,
		TypeIncompatibleOrArithmeticError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a investing pool
		/// Admin should be guardian multisig
		#[pallet::call_index(0)]
		#[pallet::weight({1000})]
		#[transactional]
		pub fn create_investing_pool(
			origin: OriginFor<T>,
			pool_id: InvestingPoolIndex,
			setting: PoolSetting<T::AccountId, BlockNumberFor<T>, BalanceOf<T>>,
			admin: T::AccountId,
		) -> DispatchResult {
			T::PoolProposalPalletOrigin::ensure_origin(origin)?;

			// Create all asset token categories
			let asset_id_vec =
				InvestingPoolAssetIdGenerator::get_all_pool_token(pool_id, setting.epoch)
					.ok_or(ArithmeticError::Overflow)?;
			for i in asset_id_vec.iter() {
				<T::Fungibles as FsCreate<<T as frame_system::Config>::AccountId>>::create(
					i.clone(),
					admin,
					true,
					One::one(),
				);
			}

			ensure!(
				frame_system::Pallet::<T>::block_number() <= setting.start_time,
				Error::<T>::PoolAlreadyStarted
			);
			ensure!(
				!InvestingPoolSetting::<T>::contains_key(&pool_id),
				Error::<T>::PoolAlreadyExisted
			);
			<InvestingPoolSetting<T>>::insert(pool_id.clone(), setting.clone());
			Self::deposit_event(Event::InvestingPoolCreated {
				pool_id,
				admin: setting.admin,
				start_time: setting.start_time,
				epoch: setting.epoch,
				epoch_range: setting.epoch_range,
				pool_cap: setting.pool_cap,
			});
			Ok(())
		}

		/// Update a reward for an investing pool of specific epoch
		/// Each epoch can be only updated once
		#[pallet::call_index(1)]
		#[pallet::weight({1000})]
		#[transactional]
		pub fn update_reward(
			origin: OriginFor<T>,
			pool_id: InvestingPoolIndex,
			epoch: u128,
			reward: BalanceOf<T>,
		) -> DispatchResult {
			T::RewardUpdateOrigin::ensure_origin(origin)?;

			let setting = <InvestingPoolSetting<T>>::get(pool_id.clone())
				.ok_or(Error::<T>::PoolNotExisted)?;
			ensure!(0 < epoch && epoch <= setting.epoch, Error::<T>::EpochNotExist);

			<StableInvestingPoolEpochReward<T>>::try_mutate(
				&pool_id,
				&epoch,
				|maybe_reward| -> DispatchResult {
					ensure!(maybe_reward.is_none(), Error::<T>::RewardAlreadyExisted);

					*maybe_reward = Some(reward);
					Self::deposit_event(Event::<T>::RewardUpdated {
						pool_id: pool_id.clone(),
						epoch,
						amount: reward,
					});
					Ok(())
				},
			)?;

			// Mint AIUSD into reward pool
			let aiusd_asset_id = <AIUSDAssetId<T>>::get().ok_or(Error::<T>::NoAssetId)?;
			let beneficiary_account: T::AccountId = Self::stable_token_beneficiary_account();
			let _ = T::Fungibles::mint_into(aiusd_asset_id, &beneficiary_account, reward)?;

			Ok(())
		}

		// Claim CAN and stable token reward, destroy/create corresponding pool token category
		#[pallet::call_index(2)]
		#[pallet::weight({1000})]
		#[transactional]
		pub fn claim(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			let pool_id = InvestingPoolAssetIdGenerator::get_token_pool_index(asset_id);
			// Epoch reward may update before epoch ends, which is also fine to claim early
			let mut claimed_until_epoch =
				Self::get_epoch_index_with_reward_updated_before(pool_id, current_block);
			let token_start_epoch = InvestingPoolAssetIdGenerator::get_token_start_epoch(asset_id);
			let token_end_epoch = InvestingPoolAssetIdGenerator::get_token_end_epoch(asset_id);

			// Technically speaking, start_epoch <= end_epoch
			if token_start_epoch > claimed_until_epoch {
				// Nothing to claim
				return Ok(());
			}

			// Burn old category token
			T::Fungibles::burn_from(
				asset_id,
				&source,
				amount,
				Precision::Exact,
				// Seem to be no effect
				Fortitude::Polite,
			)?;
			// Whether this claim leads to termination of investing procedure
			let mut terminated: bool = false;
			if token_end_epoch <= claimed_until_epoch {
				// We simply destroy the category token without minting new
				claimed_until_epoch = token_end_epoch;
				terminated = true;
			} else {
				// Mint new category token
				let new_asset_id = InvestingPoolAssetIdGenerator::get_intermediate_epoch_token(
					pool_id,
					claimed_until_epoch + 1,
					token_end_epoch,
				)
				.ok_or(ArithmeticError::Overflow)?;
				T::Fungibles::mint_into(new_asset_id, &source, amount)?;
			}
			Self::do_can_claim(
				source,
				pool_id,
				amount,
				Self::get_epoch_start_time(pool_id, token_start_epoch)?,
				Self::get_epoch_end_time(pool_id, claimed_until_epoch)?,
				terminated,
			)?;
			Self::do_stable_claim(source, pool_id, amount, token_start_epoch, claimed_until_epoch)?
		}

		// Registing AIUSD asset id
		#[pallet::call_index(3)]
		#[pallet::weight({1000})]
		#[transactional]
		pub fn regist_aiusd(origin: OriginFor<T>, asset_id: AssetIdOf<T>) -> DispatchResult {
			T::InvestingPoolAdminOrigin::ensure_origin(origin)?;
			<AIUSDAssetId<T>>::put(asset_id.clone());
			Self::deposit_event(Event::<T>::AIUSDRegisted { asset_id });
			Ok(())
		}

		// Registing CAN asset id
		#[pallet::call_index(4)]
		#[pallet::weight({1000})]
		#[transactional]
		pub fn regist_can(origin: OriginFor<T>, asset_id: AssetIdOf<T>) -> DispatchResult {
			T::InvestingPoolAdminOrigin::ensure_origin(origin)?;
			<AIUSDAssetId<T>>::put(asset_id.clone());
			Self::deposit_event(Event::<T>::AIUSDRegisted { asset_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Epoch starting from 1, epoch 0 means not start
		// The undergoing epoch index if at "time"
		// return setting.epoch if time >= pool end_time
		fn get_epoch_index(
			pool_id: InvestingPoolIndex,
			time: BlockNumberFor<T>,
		) -> Result<u128, sp_runtime::DispatchError> {
			let setting =
				<InvestingPoolSetting<T>>::get(pool_id).ok_or(Error::<T>::PoolNotExisted)?;
			// If start_time > time, means epoch 0
			if setting.start_time > time {
				return Ok(0);
			}
			let index_bn = time
				.saturating_sub(setting.start_time)
				.checked_div(&setting.epoch_range)
				.ok_or(ArithmeticError::DivisionByZero)?;
			let index: u128 =
				index_bn.try_into().or(Err(Error::<T>::TypeIncompatibleOrArithmeticError))?;
			if index >= setting.epoch {
				return Ok(setting.epoch);
			} else {
				return index.checked_add(1u128).ok_or(ArithmeticError::Overflow)?;
			}
		}

		// Epoch starting from 1
		// The largest epoch index with reward updated before "time" (including epoch during that time)
		// return setting.epoch if all epoch reward updated and time >= pool end_time
		fn get_epoch_index_with_reward_updated_before(
			pool_id: InvestingPoolIndex,
			time: BlockNumberFor<T>,
		) -> Result<u128, sp_runtime::DispatchError> {
			let epoch_index: u128 = Self::get_epoch_index(pool_id, time)?;

			for i in 1u128..(epoch_index + 1u128) {
				if <StableInvestingPoolEpochReward<T>>::get(pool_id, i).is_none() {
					return Ok(i);
				}
			}
			Ok(epoch_index)
		}

		// return pool ending time if epoch > setting.epoch
		// Epoch starting from 1
		fn get_epoch_start_time(
			pool_id: InvestingPoolIndex,
			epoch: u128,
		) -> Result<BlockNumberFor<T>, sp_runtime::DispatchError> {
			let setting =
				<InvestingPoolSetting<T>>::get(pool_id).ok_or(Error::<T>::PoolNotExisted)?;
			// If epoch larger than setting
			if epoch > setting.epoch {
				return Ok(setting
					.end_time()
					.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?);
			}
			let epoch_bn: BlockNumberFor<T> = epoch
				.checked_sub(1u128)
				.ok_or(ArithmeticError::Overflow)?
				.try_into()
				.or(Err(Error::<T>::TypeIncompatibleOrArithmeticError))?;
			let result = setting
				.start_time
				.checked_add(
					&setting.epoch_range.checked_mul(&epoch_bn).ok_or(ArithmeticError::Overflow)?,
				)
				.ok_or(ArithmeticError::Overflow)?;
			return Ok(result);
		}

		// return pool ending time if epoch >= setting.epoch
		// Epoch starting from 1
		fn get_epoch_end_time(
			pool_id: InvestingPoolIndex,
			epoch: u128,
		) -> Result<BlockNumberFor<T>, sp_runtime::DispatchError> {
			let setting =
				<InvestingPoolSetting<T>>::get(pool_id).ok_or(Error::<T>::PoolNotExisted)?;
			// If epoch larger than setting
			if epoch >= setting.epoch {
				return Ok(setting
					.end_time()
					.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?);
			}
			let epoch_bn: BlockNumberFor<T> =
				epoch.try_into().or(Err(Error::<T>::TypeIncompatibleOrArithmeticError))?;
			let result = setting
				.start_time
				.checked_add(
					&setting.epoch_range.checked_mul(&epoch_bn).ok_or(ArithmeticError::Overflow)?,
				)
				.ok_or(ArithmeticError::Overflow)?;
			return Ok(result);
		}

		// For can_investing
		fn do_can_add(amount: BalanceOf<T>, effective_time: BlockNumberFor<T>) -> DispatchResult {
			<CANCheckpoint<T>>::try_mutate(|maybe_checkpoint| {
				if let Some(checkpoint) = maybe_checkpoint {
					checkpoint
						.add(effective_time, amount)
						.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?;
				} else {
					*maybe_checkpoint = Some(InvestingWeightInfo {
						effective_time,
						amount,
						last_add_time: effective_time,
					});
				}
				Ok::<(), DispatchError>(())
			})?;
		}

		// For stable_investing
		fn do_stable_add(
			pool_id: InvestingPoolIndex,
			amount: BalanceOf<T>,
			effective_time: BlockNumberFor<T>,
		) -> DispatchResult {
			<StableInvestingPoolCheckpoint<T>>::try_mutate(&pool_id, |maybe_checkpoint| {
				if let Some(checkpoint) = maybe_checkpoint {
					checkpoint
						.add(effective_time, amount)
						.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?;
				} else {
					*maybe_checkpoint = Some(InvestingWeightInfo {
						effective_time,
						amount,
						last_add_time: effective_time,
					});
				}
				Ok::<(), DispatchError>(())
			})?;
			Ok(())
		}

		// Distribute can reward
		// No category token destroyed/created
		fn do_can_claim(
			who: T::AccountId,
			pool_id: InvestingPoolIndex,
			amount: BalanceOf<T>,
			start_time: BlockNumberFor<T>,
			end_time: BlockNumberFor<T>,
			terminated: bool,
		) -> DispatchResult {
			let beneficiary_account: T::AccountId = Self::can_token_beneficiary_account();
			let can_asset_id = <CANAssetId<T>>::get().ok_or(Error::<T>::NoAssetId)?;
			// BalanceOf
			let reward_pool = T::Fungibles::balance(can_asset_id, &beneficiary_account);
			let current_block = frame_system::Pallet::<T>::block_number();

			if start_time > end_time {
				// Nothing to claim
				// Do nothing
				return Ok(());
			} else {
				if let Some(mut ncp) = <CANCheckpoint<T>>::get() {
					let mut claim_duration: u128;
					if terminated {
						// This means the effective investing duration is beyond the pool lifespan
						// i.e. users who do not claim reward after the pool end are still considering as in-pool contributing their weights
						claim_duration = (current_block - start_time)
							.try_into()
							.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?;
					} else {
						// Only counting the investing weight during the epoch
						// Claim from start_time until the end_time
						claim_duration = (end_time - start_time)
							.try_into()
							.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?;
					}

					let claim_weight: u128 = claim_duration
						.checked_mul(
							&amount
								.try_into()
								.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?,
						)
						.ok_or(ArithmeticError::Overflow)?;
					let proportion = Perquintill::from_rational(
						claim_weight,
						ncp.weight_force(current_block)
							.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?,
					);

					let reward_pool_u128: u128 = reward_pool
						.try_into()
						.or(Err(Error::<T>::TypeIncompatibleOrArithmeticError))?;
					let distributed_reward_u128: u128 = proportion * reward_pool_u128;
					let distributed_reward: BalanceOf<T> = distributed_reward_u128
						.try_into()
						.or(Err(Error::<T>::TypeIncompatibleOrArithmeticError))?;
					// Transfer CAN reward
					T::Fungibles::transfer(
						can_asset_id,
						&beneficiary_account,
						&who,
						distributed_reward,
						Preservation::Expendable,
					)?;

					// Update gloabl investing status
					if terminated {
						let _ = ncp
							.remove(start_time, amount)
							.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?;
					} else {
						// Do not care what new Synthetic effective_time of investing pool
						let _ = ncp
							.claim_based_on_weight(claim_weight)
							.ok_or(Error::<T>::TypeIncompatibleOrArithmeticError)?;
					}

					// Adjust checkpoint
					<CANCheckpoint<T>>::put(ncp);
					Self::deposit_event(Event::<T>::CANRewardClaimed {
						who,
						claim_duration,
						amount,
						reward_amount: distributed_reward,
					});
				}
			}
			Ok(())
		}

		// Distribute stable reward
		// No category token destroyed/created
		// Claim epoch between start_epoch - end_epoch (included)
		fn do_stable_claim(
			who: T::AccountId,
			pool_id: InvestingPoolIndex,
			amount: BalanceOf<T>,
			start_epoch: u128,
			end_epoch: u128,
		) -> DispatchResult {
			let current_block = frame_system::Pallet::<T>::block_number();
			let beneficiary_account: T::AccountId = Self::stable_token_beneficiary_account();
			let aiusd_asset_id = <AIUSDAssetId<T>>::get().ok_or(Error::<T>::NoAssetId)?;

			let total_distributed_reward: BalanceOf<T> = Zero::zero();

			if start_epoch > end_epoch {
				// Nothing to claim
				// Do nothing
				return Ok(());
			} else {
				if let Some(mut scp) = <StableInvestingPoolCheckpoint<T>>::get(pool_id) {
					// Must exist
					let total_investing = scp.amount;
					// Claim until the claimed_until_epoch
					// loop through each epoch
					for i in start_epoch..(end_epoch + 1) {
						let reward_pool = <StableInvestingPoolEpochReward<T>>::get(pool_id, i)
							.ok_or(Error::<T>::EpochRewardNotUpdated)?;

						let proportion = Perquintill::from_rational(amount, total_investing);

						let reward_pool_u128: u128 = reward_pool
							.try_into()
							.or(Err(Error::<T>::TypeIncompatibleOrArithmeticError))?;
						let distributed_reward_u128: u128 = proportion * reward_pool_u128;
						let distributed_reward: BalanceOf<T> =
							distributed_reward_u128
								.try_into()
								.or(Err(Error::<T>::TypeIncompatibleOrArithmeticError))?;
						total_distributed_reward = total_distributed_reward
							.checked_add(distributed_reward)
							.ok_or(ArithmeticError::Overflow)?;

						Self::deposit_event(Event::<T>::StableRewardClaimed {
							who,
							pool_id,
							i,
							reward_amount: distributed_reward,
						});
					}
				}
			}

			// Stable token reward
			// Will fail if insufficient balance
			T::Fungibles::transfer(
				aiusd_asset_id,
				&beneficiary_account,
				&who,
				total_distributed_reward,
				Preservation::Expendable,
			)?;

			Ok(())
		}

		pub fn can_token_beneficiary_account() -> T::AccountId {
			T::CANBeneficiaryId::get().into_account_truncating()
		}

		pub fn stable_token_beneficiary_account() -> T::AccountId {
			T::StableTokenBeneficiaryId::get().into_account_truncating()
		}

		// Mint category token to user, record can token checkpoint accordingly
		pub fn inject_investment(
			pool_id: InvestingPoolIndex,
			investments: Vec<(T::AccountId, BalanceOf<T>)>,
		) -> DispatchResult {
			let setting =
				<InvestingPoolSetting<T>>::get(pool_id).ok_or(Error::<T>::PoolNotExisted)?;
			let effective_time = Self::get_epoch_start_time(pool_id, One::one());

			let debt_asset_id =
				InvestingPoolAssetIdGenerator::get_debt_token(pool_id, setting.epoch)
					.ok_or(ArithmeticError::Overflow)?;
			let initial_epoch_asset_id =
				InvestingPoolAssetIdGenerator::get_initial_epoch_token(pool_id, setting.epoch)
					.ok_or(ArithmeticError::Overflow)?;
			for i in investments.iter() {
				// Mint certification token to user
				let _ = T::Fungibles::mint_into(debt_asset_id, &i.0, i.1)?;

				let _ = T::Fungibles::mint_into(initial_epoch_asset_id, &i.0, i.1)?;

				// Add CAN token global checkpoint
				Self::do_can_add(i.1, effective_time)?;
				Self::do_stable_add(pool_id, i.1, effective_time)
			}
			Ok(())
		}
	}

	impl<T: Config> InvestmentInjector<T::AccountId, BalanceOf<T>> for Pallet<T> {
		fn inject_investment(
			pool_id: InvestingPoolIndex,
			investments: Vec<(T::AccountId, BalanceOf<T>)>,
		) -> DispatchResult {
			Self::inject_investment(pool_id, investments)
		}
	}
}
