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
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_pool_proposal::{
	AssetBalanceOf, Bond as PalletBond, PoolProposalInfo as PalletPoolProposalInfo,
};
use parity_scale_codec::MaxEncodedLen;
use precompile_utils::prelude::*;
use sp_runtime::traits::Dispatchable;

use sp_core::{Get, H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};

use pallet_collab_ai_common::PoolProposalIndex;

pub struct PoolProposalPrecompile<Runtime>(PhantomData<Runtime>);

type BalanceOf<T> = <<T as pallet_pool_proposal::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[precompile_utils::precompile]
impl<Runtime> PoolProposalPrecompile<Runtime>
where
	Runtime: pallet_pool_proposal::Config + pallet_evm::Config,
	Runtime::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime as frame_system::Config>::RuntimeCall: From<pallet_pool_proposal::Call<Runtime>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	AssetBalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
	BlockNumberFor<Runtime>: TryFrom<U256> + Into<U256>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	#[precompile::public("proposeInvestingPool(uint256,uint256,uint256,uint256,bytes32)")]
	fn propose_investing_pool(
		handle: &mut impl PrecompileHandle,
		max_pool_size: U256,
		proposal_last_time: U256,
		pool_last_time: U256,
		estimated_pool_reward: U256,
		pool_info_hash: H256,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let max_pool_size: AssetBalanceOf<Runtime> = max_pool_size.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		let proposal_last_time: BlockNumberFor<Runtime> =
			proposal_last_time.try_into().map_err(|_| {
				Into::<PrecompileFailure>::into(RevertReason::value_is_too_large(
					"block number type",
				))
			})?;

		let pool_last_time: BlockNumberFor<Runtime> = pool_last_time.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("block number type"))
		})?;

		let estimated_pool_reward: AssetBalanceOf<Runtime> =
			estimated_pool_reward.try_into().map_err(|_| {
				Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
			})?;

		let call = pallet_pool_proposal::Call::<Runtime>::propose_investing_pool {
			max_pool_size,
			proposal_last_time,
			pool_last_time,
			estimated_pool_reward,
			pool_info_hash,
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("preStakeProposal(uint256,uint256)")]
	fn pre_stake_proposal(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: U256,
		amount: U256,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_proposal_index: u128 = pool_proposal_index.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;

		let amount: AssetBalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		let call = pallet_pool_proposal::Call::<Runtime>::pre_stake_proposal {
			pool_proposal_index,
			amount,
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("withdrawPreInvesting(uint256,uint256)")]
	fn withdraw_pre_investing(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: U256,
		amount: U256,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_proposal_index: u128 = pool_proposal_index.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;

		let amount: AssetBalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		let call = pallet_pool_proposal::Call::<Runtime>::withdraw_pre_investing {
			pool_proposal_index,
			amount,
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("guardianParticipateProposal(uint256)")]
	fn guardian_participate_proposal(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: U256,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_proposal_index: u128 = pool_proposal_index.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;

		let call = pallet_pool_proposal::Call::<Runtime>::guardian_participate_proposal {
			pool_proposal_index,
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("poolProposalCount()")]
	#[precompile::view]
	fn pool_proposal_count(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// Storage item: PoolProposalCount ->
		// 		PoolProposalIndex (16)
		handle.record_db_read::<Runtime>(16)?;

		let next_proposal_index: U256 =
			pallet_pool_proposal::Pallet::<Runtime>::pool_proposal_count().into();
		Ok(next_proposal_index)
	}

	#[precompile::public("poolProposalDepositOf(bytes32)")]
	#[precompile::view]
	fn pool_proposal_deposit_of(
		handle: &mut impl PrecompileHandle,
		curator: H256,
	) -> EvmResult<Vec<DepositBond>> {
		// Storage item: PoolProposalDepositOf ->
		// 		OrderedSet<Bond<PoolProposalIndex, BalanceOf<T>>, T::MaximumPoolProposed>
		handle.record_db_read::<Runtime>(
			PalletBond::<PoolProposalIndex, BalanceOf<Runtime>>::max_encoded_len()
				.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;

		let curator: [u8; 32] = curator.into();
		let curator = Runtime::AccountId::from(curator);

		if let Some(result) =
			pallet_pool_proposal::Pallet::<Runtime>::pool_proposal_deposit_of(curator)
		{
			Ok(result
				.0
				.into_iter()
				.enumerate()
				.map(|(_index, bond)| DepositBond {
					owner: bond.owner.into(),
					amount: bond.amount.into(),
				})
				.collect())
		} else {
			Ok(Vec::new())
		}
	}

	#[precompile::public("pendingPoolProposalStatus()")]
	#[precompile::view]
	fn pending_pool_proposal_status(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<Vec<PoolProposalStatus>> {
		// Storage item: PendingPoolProposalStatus ->
		// 		VecDeque<PoolProposalStatus<BlockNumberFor<T>>
		//      16 * max number
		handle.record_db_read::<Runtime>(
			16usize.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;

		let result = pallet_pool_proposal::Pallet::<Runtime>::pending_pool_proposal_status()
			.into_iter()
			.enumerate()
			.map(|(_index, status)| PoolProposalStatus {
				index: status.pool_proposal_index.into(),
				expiry_time: status.proposal_expire_time.into(),
			})
			.collect();

		Ok(result)
	}

	#[precompile::public("poolProposal(uint256)")]
	#[precompile::view]
	fn pool_proposal(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: U256,
	) -> EvmResult<PoolProposalInfo> {
		// Storage item: PoolProposal ->
		// 		PoolProposalInfo<InfoHash, AssetBalanceOf<T>, BlockNumberFor<T>, T::AccountId>
		handle.record_db_read::<Runtime>(PalletPoolProposalInfo::<
			H256,
			AssetBalanceOf<Runtime>,
			BlockNumberFor<Runtime>,
			Runtime::AccountId,
		>::max_encoded_len())?;

		let pool_proposal_index: u128 = pool_proposal_index.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;

		if let Some(info) =
			pallet_pool_proposal::Pallet::<Runtime>::pool_proposal(pool_proposal_index)
		{
			let proposer: [u8; 32] = info.proposer.into();
			let proposer = proposer.into();

			Ok(PoolProposalInfo {
				exist: true,
				proposer,
				info_hash: info.pool_info_hash,
				max_pool_size: info.max_pool_size.into(),
				pool_start_time: info.pool_start_time.into(),
				pool_end_time: info.pool_end_time.into(),
				estimated_pool_reward: info.estimated_pool_reward.into(),
				proposal_status_flags: info.proposal_status_flags.bits(),
			})
		} else {
			Ok(PoolProposalInfo {
				exist: false,
				proposer: Default::default(),
				info_hash: Default::default(),
				max_pool_size: Default::default(),
				pool_start_time: Default::default(),
				pool_end_time: Default::default(),
				estimated_pool_reward: Default::default(),
				proposal_status_flags: Default::default(),
			})
		}
	}

	#[precompile::public("batchPoolProposal(uint256,uint256)")]
	#[precompile::view]
	fn batch_pool_proposal(
		handle: &mut impl PrecompileHandle,
		start_id: U256,
		end_id: U256,
	) -> EvmResult<Vec<PoolProposalInfo>> {
		let start_id: u128 = start_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		let end_id: u128 = end_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;

		let length: u128 = end_id.checked_sub(start_id).ok_or(Into::<PrecompileFailure>::into(
			RevertReason::value_is_too_large("id overflow"),
		))?;
		// Storage item: PoolProposal ->
		// 		PoolProposalInfo<InfoHash, AssetBalanceOf<T>, BlockNumberFor<T>, T::AccountId>
		let length_usize: usize = length.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		handle.record_db_read::<Runtime>(
			PalletPoolProposalInfo::<
				H256,
				AssetBalanceOf<Runtime>,
				BlockNumberFor<Runtime>,
				Runtime::AccountId,
			>::max_encoded_len()
			.saturating_mul(length_usize),
		)?;

		let result = (start_id..end_id)
			.map(|i| {
				if let Some(info) = pallet_pool_proposal::Pallet::<Runtime>::pool_proposal(i) {
					let proposer: [u8; 32] = info.proposer.into();
					let proposer = proposer.into();

					PoolProposalInfo {
						exist: true,
						proposer,
						info_hash: info.pool_info_hash,
						max_pool_size: info.max_pool_size.into(),
						pool_start_time: info.pool_start_time.into(),
						pool_end_time: info.pool_end_time.into(),
						estimated_pool_reward: info.estimated_pool_reward.into(),
						proposal_status_flags: info.proposal_status_flags.bits(),
					}
				} else {
					PoolProposalInfo {
						exist: false,
						proposer: Default::default(),
						info_hash: Default::default(),
						max_pool_size: Default::default(),
						pool_start_time: Default::default(),
						pool_end_time: Default::default(),
						estimated_pool_reward: Default::default(),
						proposal_status_flags: Default::default(),
					}
				}
			})
			.collect();

		Ok(result)
	}

	#[precompile::public("poolPreInvestings(uint256,uint256)")]
	#[precompile::view]
	fn pool_pre_investings(
		handle: &mut impl PrecompileHandle,
		start_id: U256,
		end_id: U256,
	) -> EvmResult<Vec<StakingBond>> {
		let start_id: u128 = start_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		let end_id: u128 = end_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		// Storage item: PoolPreInvestings ->
		// 		PoolProposalPreInvesting<T::AccountId, AssetBalanceOf<T>, BlockNumberFor<T>, T::MaximumPoolProposed>
		handle.record_db_read::<Runtime>(
			PalletBond::<Runtime::AccountId, AssetBalanceOf<Runtime>>::max_encoded_len()
				.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;
		let mut bond_result = Vec::<StakingBond>::new();
		for n in start_id..end_id {
			if let Some(result) = pallet_pool_proposal::Pallet::<Runtime>::pool_pre_investings(n) {
				let bond_vec: Vec<StakingBond> = result
					.pre_investings
					.into_iter()
					.enumerate()
					.map(|(_index, bond)| {
						let owner: [u8; 32] = bond.owner.into();
						let owner = owner.into();
						StakingBond { pool_index: n.into(), owner, amount: bond.amount.into() }
					})
					.collect();

				bond_result.extend(bond_vec);
			}
		}

		Ok(bond_result)
	}

	#[precompile::public("poolPreInvestingsQueued(uint256,uint256)")]
	#[precompile::view]
	fn pool_pre_investings_queued(
		handle: &mut impl PrecompileHandle,
		start_id: U256,
		end_id: U256,
	) -> EvmResult<Vec<QueuedStakingBond>> {
		let start_id: u128 = start_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		let end_id: u128 = end_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		// Storage item: PoolPreInvestings ->
		// 		PoolProposalPreInvesting<T::AccountId, AssetBalanceOf<T>, BlockNumberFor<T>, T::MaximumPoolProposed>
		handle.record_db_read::<Runtime>(
			PalletBond::<Runtime::AccountId, AssetBalanceOf<Runtime>>::max_encoded_len()
				.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;

		let mut bond_result = Vec::<QueuedStakingBond>::new();
		for n in start_id..end_id {
			if let Some(result) = pallet_pool_proposal::Pallet::<Runtime>::pool_pre_investings(n) {
				let bond_vec: Vec<QueuedStakingBond> = result
					.queued_pre_investings
					.into_iter()
					.enumerate()
					.map(|(_index, bond)| {
						let owner: [u8; 32] = bond.0.owner.into();
						let owner = owner.into();
						QueuedStakingBond {
							pool_index: n.into(),
							owner,
							amount: bond.0.amount.into(),
							queued_time: bond.1.into(),
						}
					})
					.collect();

				bond_result.extend(bond_vec);
			}
		}

		Ok(bond_result)
	}

	#[precompile::public("poolGuardian(uint256)")]
	#[precompile::view]
	fn pool_guardian(
		handle: &mut impl PrecompileHandle,
		pool_proposal_index: U256,
	) -> EvmResult<Vec<H256>> {
		// Storage item: PoolGuardian ->
		// 		OrderedSet<T::AccountId, T::MaxGuardianPerProposal>
		handle.record_db_read::<Runtime>(
			32usize.saturating_mul(Runtime::MaxGuardianPerProposal::get() as usize),
		)?;

		let pool_proposal_index: u128 = pool_proposal_index.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;

		if let Some(result) =
			pallet_pool_proposal::Pallet::<Runtime>::pool_guardian(pool_proposal_index)
		{
			let guardian_vec = result
				.0
				.into_iter()
				.enumerate()
				.map(|(_index, guardian)| {
					let guardian: [u8; 32] = guardian.into();
					guardian.into()
				})
				.collect();

			Ok(guardian_vec)
		} else {
			Ok(Default::default())
		}
	}

	#[precompile::public("userPoolPreInvestings(bytes32)")]
	#[precompile::view]
	fn user_pool_pre_investings(
		handle: &mut impl PrecompileHandle,
		user_address: H256,
	) -> EvmResult<Vec<StakingBond>> {
		// Storage item: PendingPoolProposalStatus ->
		// 		VecDeque<PoolProposalStatus<BlockNumberFor<T>>
		//      16 * max number
		// Storage item: PoolPreInvestings ->
		// 		PoolProposalPreInvesting<T::AccountId, AssetBalanceOf<T>, BlockNumberFor<T>, T::MaximumPoolProposed>
		handle.record_db_read::<Runtime>(
			16usize.saturating_mul(Runtime::MaximumPoolProposed::get() as usize)
				+ PalletBond::<Runtime::AccountId, AssetBalanceOf<Runtime>>::max_encoded_len()
					.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;

		let user_address: [u8; 32] = user_address.into();
		let user_address = Runtime::AccountId::from(user_address);

		let mut bond_result = Vec::<StakingBond>::new();

		let pendings = pallet_pool_proposal::Pallet::<Runtime>::pending_pool_proposal_status();
		for pool_proposal_status in pendings.iter() {
			// get underlying investings
			if let Some(result) = pallet_pool_proposal::Pallet::<Runtime>::pool_pre_investings(
				pool_proposal_status.pool_proposal_index,
			) {
				if let Some(bond) =
					result.pre_investings.into_iter().find(|x| x.owner == user_address)
				{
					let owner: [u8; 32] = bond.owner.into();
					let owner = owner.into();
					bond_result.push(StakingBond {
						pool_index: pool_proposal_status.pool_proposal_index.into(),
						owner,
						amount: bond.amount.into(),
					});
				}
			}
		}

		Ok(bond_result)
	}

	#[precompile::public("userPoolPreInvestingsQueued(bytes32)")]
	#[precompile::view]
	fn user_pool_pre_investings_queued(
		handle: &mut impl PrecompileHandle,
		user_address: H256,
	) -> EvmResult<Vec<QueuedStakingBond>> {
		// Storage item: PendingPoolProposalStatus ->
		// 		VecDeque<PoolProposalStatus<BlockNumberFor<T>>
		//      16 * max number
		// Storage item: PoolPreInvestings ->
		// 		PoolProposalPreInvesting<T::AccountId, AssetBalanceOf<T>, BlockNumberFor<T>, T::MaximumPoolProposed>
		handle.record_db_read::<Runtime>(
			16usize.saturating_mul(Runtime::MaximumPoolProposed::get() as usize)
				+ PalletBond::<Runtime::AccountId, AssetBalanceOf<Runtime>>::max_encoded_len()
					.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;

		let user_address: [u8; 32] = user_address.into();
		let user_address = Runtime::AccountId::from(user_address);

		let mut bond_result = Vec::<QueuedStakingBond>::new();

		let pendings = pallet_pool_proposal::Pallet::<Runtime>::pending_pool_proposal_status();
		for pool_proposal_status in pendings.iter() {
			// get underlying investings
			if let Some(result) = pallet_pool_proposal::Pallet::<Runtime>::pool_pre_investings(
				pool_proposal_status.pool_proposal_index,
			) {
				if let Some(bond) =
					result.queued_pre_investings.into_iter().find(|x| x.0.owner == user_address)
				{
					let owner: [u8; 32] = bond.0.owner.into();
					let owner = owner.into();
					bond_result.push(QueuedStakingBond {
						pool_index: pool_proposal_status.pool_proposal_index.into(),
						owner,
						amount: bond.0.amount.into(),
						queued_time: bond.1.into(),
					});
				}
			}
		}

		Ok(bond_result)
	}

	#[precompile::public("poolPreInvestingsTotal(uint256,uint256)")]
	#[precompile::view]
	fn pool_pre_investings_total(
		handle: &mut impl PrecompileHandle,
		start_id: U256,
		end_id: U256,
	) -> EvmResult<Vec<U256>> {
		let start_id: u128 = start_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		let end_id: u128 = end_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		// Storage item: PoolPreInvestings ->
		// 		PoolProposalPreInvesting<T::AccountId, AssetBalanceOf<T>, BlockNumberFor<T>, T::MaximumPoolProposed>
		handle.record_db_read::<Runtime>(
			PalletBond::<Runtime::AccountId, AssetBalanceOf<Runtime>>::max_encoded_len()
				.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;

		let mut total_vec_result = Vec::<U256>::new();
		for n in start_id..end_id {
			if let Some(result) = pallet_pool_proposal::Pallet::<Runtime>::pool_pre_investings(n) {
				let total: U256 = result.total_pre_investing_amount.into();

				total_vec_result.push(total);
			}
		}

		Ok(total_vec_result)
	}

	#[precompile::public("poolPreInvestingsQueuedTotal(uint256,uint256)")]
	#[precompile::view]
	fn pool_pre_investings_queued_total(
		handle: &mut impl PrecompileHandle,
		start_id: U256,
		end_id: U256,
	) -> EvmResult<Vec<U256>> {
		let start_id: u128 = start_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		let end_id: u128 = end_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		// Storage item: PoolPreInvestings ->
		// 		PoolProposalPreInvesting<T::AccountId, AssetBalanceOf<T>, BlockNumberFor<T>, T::MaximumPoolProposed>
		handle.record_db_read::<Runtime>(
			PalletBond::<Runtime::AccountId, AssetBalanceOf<Runtime>>::max_encoded_len()
				.saturating_mul(Runtime::MaximumPoolProposed::get() as usize),
		)?;

		let mut total_vec_result = Vec::<U256>::new();
		for n in start_id..end_id {
			if let Some(result) = pallet_pool_proposal::Pallet::<Runtime>::pool_pre_investings(n) {
				let total: U256 = result.total_queued_amount.into();

				total_vec_result.push(total);
			}
		}

		Ok(total_vec_result)
	}
}

#[derive(Default, Debug, solidity::Codec)]
pub struct DepositBond {
	owner: U256,
	amount: U256,
}

#[derive(Default, Debug, solidity::Codec)]
pub struct StakingBond {
	pool_index: U256,
	owner: H256,
	amount: U256,
}

#[derive(Default, Debug, solidity::Codec)]
pub struct QueuedStakingBond {
	pool_index: U256,
	owner: H256,
	amount: U256,
	queued_time: U256,
}

#[derive(Default, Debug, solidity::Codec)]
pub struct PoolProposalStatus {
	index: U256,
	expiry_time: U256,
}

#[derive(Default, Debug, solidity::Codec)]
struct PoolProposalInfo {
	exist: bool,
	proposer: H256,
	info_hash: H256,
	max_pool_size: U256,
	pool_start_time: U256,
	pool_end_time: U256,
	estimated_pool_reward: U256,
	proposal_status_flags: u8,
}
