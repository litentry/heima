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
use precompile_utils::prelude::*;
use sp_runtime::traits::Dispatchable;

use sp_core::{H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};

use pallet_collab_ai_common::{CandidateStatus, GuardianVote};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub struct GuardianPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> GuardianPrecompile<Runtime>
where
	Runtime: pallet_guardian::Config + pallet_evm::Config,
	Runtime::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_guardian::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BlockNumberFor<Runtime>: TryFrom<U256> + Into<U256>,
{
	#[precompile::public("registGuardian(bytes32)")]
	fn regist_guardian(handle: &mut impl PrecompileHandle, info_hash: H256) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = pallet_guardian::Call::<Runtime>::regist_guardian { info_hash };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("updateGuardian(bytes32)")]
	fn update_guardian(handle: &mut impl PrecompileHandle, info_hash: H256) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = pallet_guardian::Call::<Runtime>::update_guardian { info_hash };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cleanGuardian()")]
	fn clean_guardian(handle: &mut impl PrecompileHandle) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = pallet_guardian::Call::<Runtime>::clean_guardian {};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("vote(bytes32,uint8,uint256)")]
	fn vote(
		handle: &mut impl PrecompileHandle,
		guardian: H256,
		status: u8,
		potential_proposal_index: U256,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let guardian: [u8; 32] = guardian.into();
		let guardian = Runtime::AccountId::from(guardian);
		let guardian_vote: GuardianVote =
			Self::to_guardian_vote(status, potential_proposal_index).in_field("guardianVote")?;
		let call = pallet_guardian::Call::<Runtime>::vote { guardian, status: Some(guardian_vote) };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	fn to_guardian_vote(status: u8, potential_proposal_index: U256) -> MayRevert<GuardianVote> {
		match status {
			0u8 => Ok(GuardianVote::Neutral),
			1u8 => Ok(GuardianVote::Aye),
			2u8 => Ok(GuardianVote::Nay),
			3u8 => {
				Ok(GuardianVote::Specific(potential_proposal_index.try_into().map_err(|_| {
					<RevertReason as Into<Revert>>::into(RevertReason::value_is_too_large(
						"proposal index type",
					))
				})?))
			},
			_ => Err(RevertReason::custom("Out of potential status result").into()),
		}
	}

	fn guardian_vote_to(guardian_vote: Option<GuardianVote>) -> MayRevert<(u8, U256)> {
		match guardian_vote {
			None => Ok((0u8, 0.into())),
			Some(GuardianVote::Neutral) => Ok((0u8, 0.into())),
			Some(GuardianVote::Aye) => Ok((1u8, 0.into())),
			Some(GuardianVote::Nay) => Ok((2u8, 0.into())),
			Some(GuardianVote::Specific(i)) => Ok((3u8, i.into())),
		}
	}

	#[precompile::public("removeAllVotes()")]
	fn remove_all_votes(handle: &mut impl PrecompileHandle) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = pallet_guardian::Call::<Runtime>::remove_all_votes {};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("publicGuardianCount()")]
	#[precompile::view]
	fn public_guardian_count(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// Storage item: GuardianIndex u128:
		// 16
		handle.record_db_read::<Runtime>(16)?;

		Ok(pallet_guardian::Pallet::<Runtime>::public_guardian_count().into())
	}

	#[precompile::public("publicGuardianToIndex(bytes32)")]
	#[precompile::view]
	fn public_guardian_to_index(
		handle: &mut impl PrecompileHandle,
		guardian: H256,
	) -> EvmResult<(bool, U256)> {
		// Storage item: GuardianIndex u128:
		// Twox64Concat(8) + T::AccountId(32) + GuardianIndex(16)
		handle.record_db_read::<Runtime>(56)?;

		let guardian = Runtime::AccountId::from(guardian.into());

		if let Some(result) = pallet_guardian::Pallet::<Runtime>::public_guardian_to_index(guardian)
		{
			Ok((true, result.into()))
		} else {
			Ok((false, Default::default()))
		}
	}

	fn candidate_status_to_u8(status: CandidateStatus) -> MayRevert<u8> {
		match status {
			CandidateStatus::Unverified => Ok(0u8),
			CandidateStatus::Verified => Ok(1u8),
			CandidateStatus::Banned => Ok(2u8),
		}
	}

	#[precompile::public("guardianIndexToInfo(uint256)")]
	#[precompile::view]
	fn guardian_index_to_info(
		handle: &mut impl PrecompileHandle,
		index: U256,
	) -> EvmResult<GuardianQueryResult> {
		// Storage item: GuardianIndex u128:
		// Twox64Concat(8) + GuardianIndex(16) + InfoHash(32) + BlockNumber(4) + T::AccountId(32) + CandidateStatus(1)
		handle.record_db_read::<Runtime>(93)?;

		let index: u128 = index.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		if let Some((info_hash, update_block, guardian, status)) =
			pallet_guardian::Pallet::<Runtime>::guardian_index_to_info(index)
		{
			let update_block: U256 = update_block.into();

			let guardian: [u8; 32] = guardian.into();
			let guardian: H256 = guardian.into();

			let status = Self::candidate_status_to_u8(status).in_field("candidateStatus")?;

			Ok(GuardianQueryResult { exist: true, info_hash, update_block, guardian, status })
		} else {
			Ok(GuardianQueryResult {
				exist: false,
				info_hash: Default::default(),
				update_block: Default::default(),
				guardian: Default::default(),
				status: Default::default(),
			})
		}
	}

	#[precompile::public("batchGuardianIndexToInfo(uint256,uint256)")]
	#[precompile::view]
	fn batch_guardian_index_to_info(
		handle: &mut impl PrecompileHandle,
		start_id: U256,
		end_id: U256,
	) -> EvmResult<Vec<GuardianQueryResult>> {
		let start_id: u128 = start_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		let end_id: u128 = end_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;

		let length: u128 = end_id.checked_sub(start_id).ok_or(Into::<PrecompileFailure>::into(
			RevertReason::value_is_too_large("id overflow"),
		))?;
		// Storage item: GuardianIndex u128:
		// Twox64Concat(8) + GuardianIndex(16) + InfoHash(32) + BlockNumber(4) + T::AccountId(32) + CandidateStatus(1)
		let length_usize: usize = length.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		handle.record_db_read::<Runtime>(length_usize.saturating_mul(93_usize))?;

		let result = (start_id..end_id)
			.map(|i| {
				if let Some((info_hash, update_block, guardian, status)) =
					pallet_guardian::Pallet::<Runtime>::guardian_index_to_info(i)
				{
					let update_block: U256 = update_block.into();

					let guardian: [u8; 32] = guardian.into();
					let guardian: H256 = guardian.into();

					let status: u8 = Self::candidate_status_to_u8(status).unwrap_or_default();

					GuardianQueryResult { exist: true, info_hash, update_block, guardian, status }
				} else {
					GuardianQueryResult {
						exist: false,
						info_hash: Default::default(),
						update_block: Default::default(),
						guardian: Default::default(),
						status: Default::default(),
					}
				}
			})
			.collect();
		Ok(result)
	}

	#[precompile::public("guardianVotes(bytes32,uint256)")]
	#[precompile::view]
	fn guardian_votes(
		handle: &mut impl PrecompileHandle,
		voter: H256,
		guardian_index: U256,
	) -> EvmResult<(u8, U256)> {
		// Storage item: GuardianIndex u128:
		// 2 * Twox64Concat(8) + GuardianIndex(16) + T::AccountId(32) + GuardianVote(1) + ProposalIndex(16)
		handle.record_db_read::<Runtime>(81)?;

		let voter = Runtime::AccountId::from(voter.into());

		let guardian_index: u128 = guardian_index.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("index type"))
		})?;
		let result = Self::guardian_vote_to(pallet_guardian::Pallet::<Runtime>::guardian_votes(
			voter,
			guardian_index,
		))
		.in_field("GuardianVote")?;

		Ok(result)
	}
}

#[derive(Default, Debug, solidity::Codec)]
struct GuardianQueryResult {
	exist: bool,
	info_hash: H256,
	update_block: U256,
	guardian: H256,
	status: u8,
}
