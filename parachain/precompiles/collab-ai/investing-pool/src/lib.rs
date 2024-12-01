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

use fp_evm::{PrecompileFailure, PrecompileHandle};
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_investing_pool::{AssetIdOf, BalanceOf};

use parity_scale_codec::MaxEncodedLen;
use precompile_utils::prelude::*;
use sp_runtime::traits::Dispatchable;

use sp_core::{H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};

use pallet_collab_ai_common::{PoolProposalIndex, PoolSetting as InvestingPoolSetting};

pub struct InvestingPoolPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> InvestingPoolPrecompile<Runtime>
where
	Runtime: pallet_investing_pool::Config + pallet_evm::Config,
	Runtime::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime as frame_system::Config>::RuntimeCall: From<pallet_investing_pool::Call<Runtime>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	AssetIdOf<Runtime>: TryFrom<U256> + Into<U256>,
	BlockNumberFor<Runtime>: TryFrom<U256> + Into<U256>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	#[precompile::public("updateReward(uint256,uint256,uint256)")]
	fn updateReward(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: U256,
		epoch: U256,
		reward: U256,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_proposal_index: PoolProposalIndex =
			pool_proposal_index.try_into().map_err(|_| {
				Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
			})?;

		let epoch: u128 = epoch.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("epoch type"))
		})?;

		let reward: BalanceOf<Runtime> = reward.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		let call = pallet_investing_pool::Call::<Runtime>::update_reward {
			pool_id: pool_proposal_index,
			epoch,
			reward,
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("claim(uint256,uint256)")]
	fn claim(handle: &mut impl PrecompileHandle, asset_id: U256, amount: U256) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let asset_id: u128 = asset_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("asset type"))
		})?;

		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		let call = pallet_investing_pool::Call::<Runtime>::claim { asset_id, amount };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("investingPoolSetting(uint256[])")]
	#[precompile::view]
	fn investing_pool_setting(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: Vec<U256>,
	) -> EvmResult<Vec<PoolSetting>> {
		// Storage item: PoolSetting
		let length_usize: usize = pool_proposal_index.len().try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		handle.record_db_read::<Runtime>(
			InvestingPoolSetting::<Runtime::AccountId, BlockNumberFor<Runtime>, BalanceOf<Runtime>>::max_encoded_len(
			)
			.saturating_mul(length_usize),
		)?;

		let mut setting_result = Vec::<PoolSetting>::new();

		for index in pool_proposal_index.iter() {
			let index_u128: PoolProposalIndex = index.clone().try_into().map_err(|_| {
				Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
			})?;
			// get underlying investings
			if let Some(result) =
				pallet_investing_pool::Pallet::<Runtime>::investing_pool_setting(index_u128)
			{
				let admin: [u8; 32] = result.admin.into();
				let admin = admin.into();
				setting_result.push(PoolSetting {
					start_time: result.start_time.into(),
					epoch: result.epoch.into(),
					epoch_range: result.epoch_range.into(),
					pool_cap: result.pool_cap.into(),
					admin,
				});
			} else {
				setting_result.push(Default::default());
			}
		}

		Ok(setting_result)
	}

	#[precompile::public("stableInvestingPoolEpochReward(uint256)")]
	#[precompile::view]
	fn stable_investing_pool_epoch_reward(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: U256,
	) -> EvmResult<Vec<EpochReward>> {
		// Storage item: PoolSetting
		handle.record_db_read::<Runtime>(InvestingPoolSetting::<
			Runtime::AccountId,
			BlockNumberFor<Runtime>,
			BalanceOf<Runtime>,
		>::max_encoded_len())?;
		let mut reward_result = Vec::<EpochReward>::new();
		let mut epoch: u128;

		let pool_proposal_index: PoolProposalIndex =
			pool_proposal_index.try_into().map_err(|_| {
				Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
			})?;

		if let Some(result) =
			pallet_investing_pool::Pallet::<Runtime>::investing_pool_setting(pool_proposal_index)
		{
			epoch = result.epoch.into();
			let length_usize: usize = epoch.try_into().map_err(|_| {
				Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
			})?;
			// Storage item: (BalanceOf<T>, BalanceOf<T>)
			handle.record_db_read::<Runtime>(32 * length_usize)?;
		} else {
			return Ok(reward_result);
		}

		for i in 1..(epoch + 1) {
			if let Some(result) =
				pallet_investing_pool::Pallet::<Runtime>::stable_investing_pool_epoch_reward(
					pool_proposal_index,
					i,
				) {
				reward_result.push(EpochReward {
					total_reward: result.0.into(),
					claimed_reward: result.1.into(),
				});
			} else {
				// Until the first pending epoch
				break;
			}
		}
		Ok(reward_result)
	}
}

#[derive(Default, Debug, solidity::Codec)]
struct PoolSetting {
	start_time: U256,
	epoch: U256,
	epoch_range: U256,
	pool_cap: U256,
	admin: H256,
}

#[derive(Default, Debug, solidity::Codec)]
struct EpochReward {
	total_reward: U256,
	claimed_reward: U256,
}
