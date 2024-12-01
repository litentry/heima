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

use crate::RuntimeCall;
use core::marker::PhantomData;
use fp_evm::{ExitError, PrecompileFailure};
use frame_support::{
	dispatch::GetDispatchInfo,
	pallet_prelude::{DispatchClass, Pays},
	parameter_types,
	traits::Contains,
};
use pallet_evm_precompile_assets_erc20::Erc20AssetsPrecompileSet;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_bridge_transfer::BridgeTransferPrecompile;
use pallet_evm_precompile_dispatch::{Dispatch, DispatchValidateT};
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_parachain_staking::ParachainStakingPrecompile;
use pallet_evm_precompile_score_staking::ScoreStakingPrecompile;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use precompile_utils::precompile_set::*;
use sp_std::fmt::Debug;

// CollabAI
use pallet_evm_precompile_aiusd_convertor::AIUSDConvertorPrecompile;
use pallet_evm_precompile_curator::CuratorPrecompile;
use pallet_evm_precompile_guardian::GuardianPrecompile;
use pallet_evm_precompile_investing_pool::InvestingPoolPrecompile;
use pallet_evm_precompile_pool_proposal::PoolProposalPrecompile;

/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet
pub const ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];
parameter_types! {
	pub AssetPrefix: &'static [u8] = ASSET_PRECOMPILE_ADDRESS_PREFIX;
}

pub struct DispatchFilterValidate<RuntimeCall, Filter: Contains<RuntimeCall>>(
	PhantomData<(RuntimeCall, Filter)>,
);

impl<AccountId, RuntimeCall: GetDispatchInfo, Filter: Contains<RuntimeCall>>
	DispatchValidateT<AccountId, RuntimeCall> for DispatchFilterValidate<RuntimeCall, Filter>
{
	fn validate_before_dispatch(
		_origin: &AccountId,
		call: &RuntimeCall,
	) -> Option<PrecompileFailure> {
		let info = call.get_dispatch_info();
		let paid_normal_call = info.pays_fee == Pays::Yes && info.class == DispatchClass::Normal;
		if !paid_normal_call {
			return Some(PrecompileFailure::Error {
				exit_status: ExitError::Other("invalid call".into()),
			});
		}
		if Filter::contains(call) {
			None
		} else {
			Some(PrecompileFailure::Error {
				exit_status: ExitError::Other("call filtered out".into()),
			})
		}
	}
}

/// Precompile checks for ethereum spec precompiles
/// We allow DELEGATECALL to stay compliant with Ethereum behavior.
type EthereumPrecompilesChecks = (AcceptDelegateCall, CallableByContract, CallableByPrecompile);

/// Filter that only allows whitelisted runtime call to pass through dispatch precompile
pub struct WhitelistedCalls;

impl Contains<RuntimeCall> for WhitelistedCalls {
	fn contains(t: &RuntimeCall) -> bool {
		match t {
			RuntimeCall::Assets(pallet_assets::Call::transfer { .. }) => true,
			RuntimeCall::Utility(pallet_utility::Call::batch { calls })
			| RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) => {
				calls.iter().all(WhitelistedCalls::contains)
			},
			_ => false,
		}
	}
}
/// The PrecompileSet installed in the rococo runtime.
#[precompile_utils::precompile_name_from_address]
pub type PrecompilesSetAt<R> = (
	// Ethereum precompiles:
	// We allow DELEGATECALL to stay compliant with Ethereum behavior.
	PrecompileAt<AddressU64<1>, ECRecover, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<2>, Sha256, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<3>, Ripemd160, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<4>, Identity, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<5>, Modexp, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<6>, Bn128Add, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<7>, Bn128Mul, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<8>, Bn128Pairing, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<9>, Blake2F, EthereumPrecompilesChecks>,
	// Non-Litentry specific nor Ethereum precompiles :
	PrecompileAt<AddressU64<1024>, Sha3FIPS256, (CallableByContract, CallableByPrecompile)>,
	PrecompileAt<
		AddressU64<1025>,
		Dispatch<R, DispatchFilterValidate<RuntimeCall, WhitelistedCalls>>,
		// Not callable from smart contract nor precompiles, only EOA accounts
		// TODO: test this without the gensis hack for blacklisted
		(),
	>,
	PrecompileAt<AddressU64<1026>, ECRecoverPublicKey, (CallableByContract, CallableByPrecompile)>,
	PrecompileAt<AddressU64<1027>, Ed25519Verify, (CallableByContract, CallableByPrecompile)>,
	// Litentry precompiles (starts from 0x5000):
	// ParachainStaking: pallet_parachain_staking = 45 + 20480
	PrecompileAt<
		AddressU64<20525>,
		ParachainStakingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// BridgeTransfer: pallet_bridge_transfer = 61 + 20480
	PrecompileAt<
		AddressU64<20541>,
		BridgeTransferPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// ScoreStaking: pallet_score_staking = 75 + 20480
	PrecompileAt<
		AddressU64<20555>,
		ScoreStakingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Curator: pallet_curator = 150 + 20480
	PrecompileAt<
		AddressU64<20630>,
		CuratorPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Guardian: pallet_guardian = 151 + 20480
	PrecompileAt<
		AddressU64<20631>,
		GuardianPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// PoolProposal: pallet_pool_proposal = 152 + 20480
	PrecompileAt<
		AddressU64<20632>,
		PoolProposalPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// InvestingPool: pallet_investing_pool = 153 + 20480
	PrecompileAt<
		AddressU64<20633>,
		InvestingPoolPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// AIUSDConvertor: pallet_aiusd_convertor = 154 + 20480
	PrecompileAt<
		AddressU64<20634>,
		AIUSDConvertorPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
);

pub type RococoNetworkPrecompiles<R> = PrecompileSetBuilder<
	R,
	(
		// Skip precompiles if out of range.
		PrecompilesInRangeInclusive<
			// We take range as last precompile index, UPDATE this once new prcompile is added
			(AddressU64<1>, AddressU64<20635>),
			PrecompilesSetAt<R>,
		>,
		// Prefixed precompile sets (XC20)
		PrecompileSetStartingWith<AssetPrefix, Erc20AssetsPrecompileSet<R>, CallableByContract>,
	),
>;
