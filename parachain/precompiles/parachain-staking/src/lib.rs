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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use fp_evm::{PrecompileFailure, PrecompileHandle};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	sp_runtime::Percent,
	traits::{Currency, Get},
};
use pallet_evm::AddressMapping;
use precompile_utils::{prelude::*, EvmResult};
use sp_core::{H256, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::marker::PhantomData;

type BalanceOf<Runtime> = <<Runtime as pallet_parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from parachain_staking.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
pub struct ParachainStakingPrecompile<Runtime>(PhantomData<Runtime>);

// TODO: Only part for delagator is implemented for minimal task purpose*/
#[precompile_utils::precompile]
impl<Runtime> ParachainStakingPrecompile<Runtime>
where
	Runtime: pallet_parachain_staking::Config + pallet_evm::Config,
	Runtime::AccountId: From<[u8; 32]>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_parachain_staking::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
{
	#[precompile::public("delegationRequestIsPending(bytes32,bytes32)")]
	#[precompile::view]
	fn delegation_request_is_pending(
		handle: &mut impl PrecompileHandle,
		delegator: H256,
		candidate: H256,
	) -> EvmResult<bool> {
		let delegator: [u8; 32] = delegator.into();
		let delegator = Runtime::AccountId::from(delegator);
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);

		// DelegationScheduledRequests:
		// Blake2128(16) + AccountId(32)
		// + Vec(
		// 	ScheduledRequest(32 + 4 + DelegationAction(18))
		// 	* (MaxTopDelegationsPerCandidate + MaxBottomDelegationsPerCandidate)
		// )
		handle.record_db_read::<Runtime>(
			48 + (
				54 * (<Runtime as pallet_parachain_staking::Config>::MaxTopDelegationsPerCandidate::get()
				+ <Runtime as pallet_parachain_staking::Config>::MaxBottomDelegationsPerCandidate::get())
				as usize),
		)?;

		// If we are not able to get delegator state, we return false
		// Users can call `is_delegator` to determine when this happens
		let pending = <pallet_parachain_staking::Pallet<Runtime>>::delegation_request_exists(
			&candidate, &delegator,
		);

		Ok(pending)
	}

	#[precompile::public("delegate(bytes32,uint256)")]
	fn delegate(handle: &mut impl PrecompileHandle, candidate: H256, amount: U256) -> EvmResult {
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::delegate { candidate, amount };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("delegateWithAutoCompound(bytes32,uint256,uint8)")]
	fn delegate_with_auto_compound(
		handle: &mut impl PrecompileHandle,
		candidate: H256,
		amount: U256,
		auto_compound: u8,
	) -> EvmResult {
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		if auto_compound > 100 {
			return Err(RevertReason::custom("Must be an integer between 0 and 100 included")
				.in_field("auto_compound")
				.into());
		}
		let auto_compound = Percent::from_percent(auto_compound);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::delegate_with_auto_compound {
			candidate,
			amount,
			auto_compound,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("scheduleRevokeDelegation(bytes32)")]
	fn schedule_revoke_delegation(
		handle: &mut impl PrecompileHandle,
		candidate: H256,
	) -> EvmResult {
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_revoke_delegation {
			collator: candidate,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("delegatorBondMore(bytes32,uint256)")]
	fn delegator_bond_more(
		handle: &mut impl PrecompileHandle,
		candidate: H256,
		more: U256,
	) -> EvmResult {
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);
		let more: BalanceOf<Runtime> = more.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::delegator_bond_more { candidate, more };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("scheduleDelegatorBondLess(bytes32,uint256)")]
	fn schedule_delegator_bond_less(
		handle: &mut impl PrecompileHandle,
		candidate: H256,
		less: U256,
	) -> EvmResult {
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);
		let less: BalanceOf<Runtime> = less.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_delegator_bond_less {
			candidate,
			less,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("executeDelegationRequest(bytes32,bytes32)")]
	fn execute_delegation_request(
		handle: &mut impl PrecompileHandle,
		delegator: H256,
		candidate: H256,
	) -> EvmResult {
		let delegator: [u8; 32] = delegator.into();
		let delegator = Runtime::AccountId::from(delegator);
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_delegation_request {
			delegator,
			candidate,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("cancelDelegationRequest(bytes32)")]
	fn cancel_delegation_request(handle: &mut impl PrecompileHandle, candidate: H256) -> EvmResult {
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::cancel_delegation_request { candidate };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("setAutoCompound(bytes32,uint8)")]
	fn set_auto_compound(
		handle: &mut impl PrecompileHandle,
		candidate: H256,
		value: u8,
	) -> EvmResult {
		let candidate: [u8; 32] = candidate.into();
		let candidate = Runtime::AccountId::from(candidate);

		if value > 100 {
			return Err(RevertReason::custom("Must be an integer between 0 and 100 included")
				.in_field("value")
				.into());
		}
		let value = Percent::from_percent(value);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::set_auto_compound { candidate, value };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}
}
