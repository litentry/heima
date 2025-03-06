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

use core_primitives::AssetId;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::fungible::NativeOrWithId,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_runtime::traits::Dispatchable;

use sp_core::U256;
use sp_std::{marker::PhantomData, vec::Vec};

use pallet_omni_bridge::{ChainType, PayInRequest};

pub struct OmniBridgePrecompile<Runtime>(PhantomData<Runtime>);

type BridgeBalanceOf<Runtime> = <Runtime as pallet_omni_bridge::Config>::Balance;

#[precompile_utils::precompile]
impl<Runtime> OmniBridgePrecompile<Runtime>
where
	Runtime: pallet_omni_bridge::Config<AssetKind = NativeOrWithId<AssetId>> + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_omni_bridge::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BridgeBalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	#[precompile::public("payIn(uint256,uint8,bool,uint256,bytes)")]
	fn pay_in(
		handle: &mut impl PrecompileHandle,
		amount: U256,
		dest_id: u8,
		native: bool,
		asset_id: U256,
		recipient: UnboundedBytes,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let amount: BridgeBalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;
		let recipient: Vec<u8> = recipient.into();
		let asset_id: AssetId = asset_id.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("asset id type"))
		})?;

		let pay_in_request: PayInRequest<NativeOrWithId<AssetId>, BridgeBalanceOf<Runtime>> =
			match native {
				true => PayInRequest {
					asset: NativeOrWithId::Native,
					// This is substrate parachain precompile
					// So always be non native chain
					dest_chain: ChainType::Ethereum(dest_id.into()),
					dest_account: recipient,
					amount,
				},
				false => PayInRequest {
					asset: NativeOrWithId::WithId(asset_id),
					// This is substrate parachain precompile
					// So always be non native chain
					dest_chain: ChainType::Ethereum(dest_id.into()),
					dest_account: recipient,
					amount,
				},
			};

		let call = pallet_omni_bridge::Call::<Runtime>::pay_in { req: pay_in_request };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}
}
