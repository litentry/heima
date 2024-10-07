#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::{PrecompileFailure, PrecompileHandle};

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_runtime::traits::Dispatchable;

use sp_core::{H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};

use pallet_aiusd_convertor::{AssetBalanceOf, AssetIdOf};

pub struct AIUSDConvertorPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> AIUSDConvertorPrecompile<Runtime>
where
	Runtime: pallet_aiusd_convertor::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_aiusd_convertor::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	AssetBalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
	AssetIdOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	#[precompile::public("mintAIUSD(uint256,uint256)")]
	fn mint_aiusd(handle: &mut impl PrecompileHandle, asset_id: U256, amount: U256) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let asset_id: AssetIdOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("asset id type"))
		})?;
		let amount: AssetBalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		let call = pallet_aiusd_convertor::Call::<Runtime>::mint_aiusd { asset_id, amount };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("burnAIUSD(uint256,uint256)")]
	fn burn_aiusd(handle: &mut impl PrecompileHandle, asset_id: U256, amount: U256) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let asset_id: AssetIdOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("asset id type"))
		})?;
		let amount: AssetBalanceOf<Runtime> = amount.try_into().map_err(|_| {
			Into::<PrecompileFailure>::into(RevertReason::value_is_too_large("balance type"))
		})?;

		let call = pallet_aiusd_convertor::Call::<Runtime>::burn_aiusd { asset_id, amount };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}