#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::{PrecompileFailure, PrecompileHandle};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;

use parity_scale_codec::MaxEncodedLen;
use precompile_utils::prelude::*;
use sp_runtime::traits::Dispatchable;

use sp_core::{Get, H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};

use pallet_collab_ai_common::PoolProposalIndex;

pub struct InvestingPoolPrecompile<Runtime>(PhantomData<Runtime>);



#[precompile_utils::precompile]
impl<Runtime> InvestingPoolPrecompile<Runtime>
where
	Runtime: pallet_investing_pool::Config + pallet_evm::Config,
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
