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
#![allow(clippy::useless_conversion)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use frame_system::{self as system, pallet_prelude::BlockNumberFor};
pub use heima_primitives::{
	Identity, Intent, MemberAccount, OmniAccountAuthType, OmniAccountConverter,
	TransferNative, TransferEthereum, CallEthereum, TransferSolana,
};
pub use pallet::*;

use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::{IsSubType, UnfilteredDispatchable},
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::Dispatchable;
use sp_std::{boxed::Box, vec, vec::Vec};

// Customized origin for this pallet, to:
// 1. to decouple `TEECallOrigin` and extrinsic that should be sent from `OmniAccount` origin only
// 2. allow other pallets to specify ensure_origin using this origin
// 3. leave room for more delicate control over OmniAccount in the future (e.g. multisig-like control)
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub enum RawOrigin<AccountId> {
	// dispatched from OmniAccount T::AccountId
	OmniAccount(AccountId),
}

#[frame_support::pallet]
pub mod pallet {
	use heima_primitives::{ChainAsset, IntentId};

	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The runtime origin type
		type RuntimeOrigin: From<RawOrigin<Self::AccountId>>
			+ From<frame_system::RawOrigin<Self::AccountId>>;

		/// The overarching call type
		type RuntimeCall: Parameter
			+ Dispatchable<
				RuntimeOrigin = <Self as Config>::RuntimeOrigin,
				PostInfo = PostDispatchInfo,
			> + GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ UnfilteredDispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>
			+ IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// The event type of this pallet
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The origin that represents the off-chain worker
		type TEECallOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		/// The origin that represents the customised OmniAccount type
		type OmniAccountOrigin: EnsureOrigin<
			<Self as frame_system::Config>::RuntimeOrigin,
			Success = Self::AccountId,
		>;

		/// Convert an `Identity` to OmniAccount type
		type OmniAccountConverter: OmniAccountConverter<OmniAccount = Self::AccountId>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {}
	}

	#[pallet::origin]
	pub type Origin<T> = RawOrigin<<T as frame_system::Config>::AccountId>;

	// For now we keep all intents online for easy query, it's concerning if it would bloat the data space
	#[pallet::storage]
	#[pallet::getter(fn intents)]
	pub type Intents<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		IntentId,
		Intent,
		OptionQuery,
	>;

	/// The hightest intent_id that has been accepted for a given AccountId
	#[pallet::storage]
	#[pallet::getter(fn accepted_intent_ids)]
	pub type AcceptedIntentIds<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, IntentId, ValueQuery>;

	/// The hightest intent_id that has been completed for a given AccountId
	#[pallet::storage]
	#[pallet::getter(fn completed_intent_ids)]
	pub type CompletedIntentIds<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, IntentId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Some call is dispatched as omni-account origin
		DispatchedAsOmniAccount {
			who: T::AccountId,
			auth_type: Option<OmniAccountAuthType>,
			result: DispatchResult,
		},
		/// Some call is dispatched as signed origin
		DispatchedAsSigned {
			who: T::AccountId,
			auth_type: Option<OmniAccountAuthType>,
			result: DispatchResult,
		},
		/// An auth token is requested
		AuthTokenRequested { who: T::AccountId, expires_at: i64 },
		/// Intent is requested by some user
		IntentRequested { who: T::AccountId, intent: Intent },
		/// Intent is accepted - we record the Intent detail (once)
		IntentAccepted { who: T::AccountId, intent_id: IntentId, intent: Intent },
		/// Intent is in-process
		IntentInProcessUpdated {
			who: T::AccountId,
			intent_id: IntentId,
			detail: IntentInProcessDetail,
		},
		/// Intent is completed
		IntentCompleted { who: T::AccountId, intent_id: IntentId, detail: IntentCompletedDetail },
	}

	#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
	pub enum IntentInProcessDetail {
		Swap(SwapInProcessDetail),
	}

	#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
	pub enum IntentCompletedDetail {
		// TODO - we might want to add more details except for just OK/NOK
		//        and maybe also per-intent case
		Success,
		Failure,
	}

	#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
	pub enum SwapInProcessDetail {
		SourceChainBalanceDeducted { asset: ChainAsset, amount: u64 },
		DestChainBalanceAdded { asset: ChainAsset, amount: u64 },
		SingleChainSwapSubmitted { tx_hash: Vec<u8> },
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidAccount,
		EmptyAccount,
		IntentAlreadyExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// dispatch the `call` as RawOrigin::OmniAccount
		#[pallet::call_index(0)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn dispatch_as_omni_account(
			origin: OriginFor<T>,
			who: T::AccountId,
			call: Box<<T as Config>::RuntimeCall>,
			auth_type: Option<OmniAccountAuthType>,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			let result = call.dispatch(RawOrigin::OmniAccount(who.clone()).into());
			system::Pallet::<T>::inc_account_nonce(&who);
			Self::deposit_event(Event::DispatchedAsOmniAccount {
				who,
				auth_type,
				result: result.map(|_| ()).map_err(|e| e.error),
			});
			Ok(Pays::No.into())
		}

		// dispatch the `call` as the standard (frame_system) signed origin
		// TODO: what about other customised origin like collective?
		#[pallet::call_index(1)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn dispatch_as_signed(
			origin: OriginFor<T>,
			who: T::AccountId,
			call: Box<<T as Config>::RuntimeCall>,
			auth_type: Option<OmniAccountAuthType>,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			let result: Result<
				PostDispatchInfo,
				sp_runtime::DispatchErrorWithPostInfo<PostDispatchInfo>,
			> = call.dispatch(frame_system::RawOrigin::Signed(who.clone()).into());
			system::Pallet::<T>::inc_account_nonce(&who);
			Self::deposit_event(Event::DispatchedAsSigned {
				who,
				auth_type,
				result: result.map(|_| ()).map_err(|e| e.error),
			});
			Ok(Pays::No.into())
		}


		// to allow any user to submit intent directly onto chain
		// this extrinsic is currently **unused**, meaning it will do nothing except emitting events
		#[pallet::call_index(2)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn request_intent(origin: OriginFor<T>, intent: Intent) -> DispatchResult {
			let who = T::OmniAccountOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::IntentRequested { who, intent });
			Ok(())
		}



		#[pallet::call_index(3)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn auth_token_requested(
			origin: OriginFor<T>,
			who: T::AccountId,
			expires_at: i64,
		) -> DispatchResult {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::AuthTokenRequested { who, expires_at });
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn intent_accepted(
			origin: OriginFor<T>,
			who: T::AccountId,
			intent_id: IntentId,
			intent: Intent,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::do_accept_intent(who, intent_id, intent)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn intent_in_process_updated(
			origin: OriginFor<T>,
			who: T::AccountId,
			intent_id: IntentId,
			detail: IntentInProcessDetail,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::IntentInProcessUpdated { who, intent_id, detail });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn intent_completed(
			origin: OriginFor<T>,
			who: T::AccountId,
			intent_id: IntentId,
			detail: IntentCompletedDetail,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			if intent_id > Self::completed_intent_ids(&who) {
				CompletedIntentIds::<T>::insert(&who, intent_id);
			}
			Self::deposit_event(Event::IntentCompleted { who, intent_id, detail });
			Ok(Pays::No.into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Given an `Identity`, get its derived OmniAccount
		/// Always uses the OmniAccountConverter to derive the account
		pub fn omni_account(client_id: String, identity: Identity) -> T::AccountId {
			T::OmniAccountConverter::convert(&identity, &client_id)
		}



		fn do_accept_intent(
			who: T::AccountId,
			intent_id: IntentId,
			intent: Intent,
		) -> DispatchResult {
			ensure!(!Intents::<T>::contains_key(&who, intent_id), Error::<T>::IntentAlreadyExists);
			if intent_id > Self::accepted_intent_ids(&who) {
				AcceptedIntentIds::<T>::insert(&who, intent_id);
			}
			Intents::<T>::insert(&who, intent_id, intent.clone());
			Self::deposit_event(Event::IntentAccepted { who, intent_id, intent });
			Ok(())
		}
	}
}

pub struct EnsureOmniAccount<AccountId>(PhantomData<AccountId>);
impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>, AccountId: Decode>
	EnsureOrigin<O> for EnsureOmniAccount<AccountId>
{
	type Success = AccountId;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::OmniAccount(id) => Ok(id),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		let zero_account_id =
			AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed");
		Ok(O::from(RawOrigin::OmniAccount(zero_account_id)))
	}
}
