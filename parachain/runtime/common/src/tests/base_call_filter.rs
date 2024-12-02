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

use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok, pallet_prelude::Weight, traits::VestingSchedule};
use frame_system::RawOrigin;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::Dispatchable;

use core_primitives::AccountId;

use crate::{
	currency::UNIT,
	tests::setup::{alice, bob, charlie, para_ext},
	BaseRuntimeRequirements,
};

type ExtrinsicFilter<R> = pallet_extrinsic_filter::Pallet<R>;
type System<R> = frame_system::Pallet<R>;
type Balances<R> = pallet_balances::Pallet<R>;
type Vesting<R> = pallet_vesting::Pallet<R>;
type Multisig<R> = pallet_multisig::Pallet<R>;

pub fn default_mode<R: BaseRuntimeRequirements>() {
	para_ext(1).execute_with(|| {
		assert_eq!(ExtrinsicFilter::<R>::mode(), pallet_extrinsic_filter::OperationalMode::Normal);
	});
}

pub fn multisig_enabled<
	R: BaseRuntimeRequirements + pallet_multisig::Config<RuntimeCall = Call>,
	Origin: frame_support::traits::OriginTrait<AccountId = AccountId> + From<RawOrigin<AccountId>>,
	Call: Clone
		+ Dispatchable<RuntimeOrigin = Origin>
		+ From<pallet_multisig::Call<R>>
		+ From<frame_system::Call<R>>
		+ Encode,
>()
where
	<R as frame_system::Config>::RuntimeCall: Decode,
	<<R as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source:
		From<sp_runtime::AccountId32>,
	<Call as Dispatchable>::PostInfo: sp_std::fmt::Debug + Default,
{
	para_ext(1).execute_with(|| {
		let _ = Balances::<R>::deposit_creating(&bob(), 10 * UNIT);
		let _ = Balances::<R>::deposit_creating(&charlie(), 10 * UNIT);
		let _ = Multisig::<R>::multi_account_id(&[alice(), bob(), charlie()][..], 2);
		let remark_call = frame_system::Call::remark { remark: vec![] }.into();
		let call = Box::new(remark_call);
		let multisig_call: Call = pallet_multisig::Call::as_multi {
			threshold: 2,
			other_signatories: vec![bob(), charlie()],
			maybe_timepoint: None,
			call,
			max_weight: Weight::zero(),
		}
		.into();
		assert_ok!(multisig_call.dispatch(Origin::signed(alice())));
	})
}

pub fn balance_transfer_works<
	R: BaseRuntimeRequirements,
	Origin: frame_support::traits::OriginTrait<AccountId = AccountId> + From<RawOrigin<AccountId>>,
	Call: Clone + Dispatchable<RuntimeOrigin = Origin> + From<pallet_balances::Call<R>> + Encode,
>()
where
	<<R as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source:
		From<sp_runtime::AccountId32>,
	<Call as Dispatchable>::PostInfo: sp_std::fmt::Debug + Default,
{
	para_ext(1).execute_with(|| {
		let call: Call =
			pallet_balances::Call::transfer_keep_alive { dest: bob().into(), value: UNIT }.into();
		assert_ok!(call.dispatch(Origin::signed(alice())));
		assert_eq!(Balances::<R>::free_balance(&bob()), UNIT);
	})
}

pub fn balance_transfer_disabled<
	R: BaseRuntimeRequirements,
	Origin: frame_support::traits::OriginTrait<AccountId = AccountId> + From<RawOrigin<AccountId>>,
	Call: Clone + Dispatchable<RuntimeOrigin = Origin> + From<pallet_balances::Call<R>> + Encode,
>()
where
	<<R as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source:
		From<sp_runtime::AccountId32>,
	<Call as Dispatchable>::PostInfo: sp_std::fmt::Debug + Default,
{
	para_ext(1).execute_with(|| {
		let call: Call =
			pallet_balances::Call::transfer_keep_alive { dest: bob().into(), value: UNIT }.into();
		assert_noop!(
			call.dispatch(Origin::signed(alice())),
			frame_system::Error::<R>::CallFiltered
		);
	})
}

pub fn balance_transfer_with_sudo_works<
	R: BaseRuntimeRequirements,
	Origin: frame_support::traits::OriginTrait<AccountId = AccountId> + From<RawOrigin<AccountId>>,
	Call: Clone + Dispatchable<RuntimeOrigin = Origin> + From<pallet_balances::Call<R>> + Encode,
>()
where
	<<R as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source:
		From<sp_runtime::AccountId32>,
	<Call as Dispatchable>::PostInfo: sp_std::fmt::Debug + Default,
{
	para_ext(1).execute_with(|| {
		let call: Call = pallet_balances::Call::force_transfer {
			source: alice().into(),
			dest: bob().into(),
			value: UNIT,
		}
		.into();
		assert_ok!(call.dispatch(Origin::root()),);
		assert_eq!(Balances::<R>::free_balance(&alice()), 9 * UNIT);
		assert_eq!(Balances::<R>::free_balance(&bob()), UNIT);
	})
}

pub fn block_core_call_has_no_effect<
	R: BaseRuntimeRequirements + frame_system::Config<RuntimeOrigin = Origin>,
	Origin: frame_support::traits::OriginTrait<AccountId = AccountId> + From<RawOrigin<AccountId>>,
	Call: Clone + Dispatchable<RuntimeOrigin = Origin> + From<frame_system::Call<R>> + Encode,
>()
where
	<<R as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source:
		From<sp_runtime::AccountId32>,
	<Call as Dispatchable>::PostInfo: sp_std::fmt::Debug + Default,
{
	para_ext(1).execute_with(|| {
		let call: Call = frame_system::Call::remark { remark: vec![] }.into();
		assert_ok!(call.clone().dispatch(Origin::signed(alice())));

		// try to block System call, which is a core call
		assert_ok!(ExtrinsicFilter::<R>::block_extrinsics(
			Origin::root(),
			b"System".to_vec(),
			None
		)); // it's stored in the storage
		assert_eq!(
			ExtrinsicFilter::<R>::blocked_extrinsics((b"System".to_vec(), Vec::<u8>::default())),
			Some(())
		);
		// ...however, no effect in the actual call dispatching
		assert_ok!(call.dispatch(Origin::signed(alice())));
	})
}

pub fn block_non_core_call_works<
	R: BaseRuntimeRequirements
		+ frame_system::Config<RuntimeOrigin = Origin>
		+ pallet_vesting::Config<Currency = Balances<R>>,
	Origin: frame_support::traits::OriginTrait<AccountId = AccountId> + From<RawOrigin<AccountId>>,
	Call: Clone + Dispatchable<RuntimeOrigin = Origin> + From<pallet_vesting::Call<R>>,
>()
where
	<<R as frame_system::Config>::Lookup as sp_runtime::traits::StaticLookup>::Source:
		From<sp_runtime::AccountId32>,
	<Call as Dispatchable>::PostInfo: sp_std::fmt::Debug + Default,
{
	para_ext(1).execute_with(|| {
		assert_ok!(Vesting::<R>::vested_transfer(
			Origin::signed(alice()),
			bob().into(),
			pallet_vesting::VestingInfo::new(10 * UNIT, UNIT, 0u32.into()),
		));
		let call: Call = pallet_vesting::Call::vest {}.into();
		assert_ok!(call.clone().dispatch(Origin::signed(bob())));
		assert_eq!(Balances::<R>::free_balance(&bob()), 10 * UNIT);
		assert_eq!(Balances::<R>::usable_balance(&bob()), UNIT);

		System::<R>::set_block_number(2u32.into());
		assert_eq!(Vesting::<R>::vesting_balance(&bob()), Some(8 * UNIT));

		// try to block Vesting call, which is a non-core call
		assert_ok!(ExtrinsicFilter::<R>::block_extrinsics(
			Origin::root(),
			b"Vesting".to_vec(),
			None
		));
		// it's stored in the storage
		assert_eq!(
			ExtrinsicFilter::<R>::blocked_extrinsics((b"Vesting".to_vec(), Vec::<u8>::default())),
			Some(())
		);
		// ...and it will take effect
		assert_noop!(call.dispatch(Origin::signed(bob())), frame_system::Error::<R>::CallFiltered);
		// usable balance is unchanged
		assert_eq!(Balances::<R>::usable_balance(&bob()), UNIT);
	})
}

#[macro_export]
macro_rules! run_call_filter_tests {
	() => {
		use runtime_common::tests::base_call_filter;

		#[test]
		fn default_mode() {
			base_call_filter::default_mode::<Runtime>();
		}

		#[test]
		fn multisig_enabled() {
			base_call_filter::multisig_enabled::<Runtime, RuntimeOrigin, RuntimeCall>();
		}

		#[test]
		fn balance_transfer_with_sudo_works() {
			base_call_filter::balance_transfer_with_sudo_works::<Runtime, RuntimeOrigin, RuntimeCall>(
			);
		}

		#[test]
		fn block_core_call_has_no_effect() {
			base_call_filter::block_core_call_has_no_effect::<Runtime, RuntimeOrigin, RuntimeCall>(
			);
		}

		#[test]
		fn block_non_core_call_works() {
			base_call_filter::block_non_core_call_works::<Runtime, RuntimeOrigin, RuntimeCall>();
		}
	};
}
