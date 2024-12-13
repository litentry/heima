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
#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
	ensure,
	pallet_prelude::*,
	traits::{Get, Time},
	weights::Weight,
};
use frame_system::pallet_prelude::*;
use parity_scale_codec::Decode;
use sp_core::{ed25519::Public as Ed25519Public, H256};
use sp_runtime::traits::{CheckedSub, SaturatedConversion};
use sp_std::{prelude::*, str};

use core_primitives::*;

pub use pallet::*;

pub mod weights;
pub use crate::weights::WeightInfo;

const MAX_RA_REPORT_LEN: usize = 5244;
const MAX_DCAP_QUOTE_LEN: usize = 5000;
const MAX_URL_LEN: usize = 256;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_timestamp::Config + pallet_utility::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		type MomentsPerDay: Get<Self::Moment>;
		/// The origin who can set the admin account
		type SetAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Maximum number of enclave identifiers allowed to be registered for a given `worker_type`
		#[pallet::constant]
		type MaxEnclaveIdentifier: Get<u32>;
		/// Maximum number of authorized enclave for a given `worker_type`
		#[pallet::constant]
		type MaxAuthorizedEnclave: Get<u32>;
	}

	// TODO: maybe add more sidechain lifecycle events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ModeSet {
			new_mode: OperationalMode,
		},
		AdminSet {
			new_admin: Option<T::AccountId>,
		},
		EnclaveAdded {
			who: T::AccountId,
			worker_type: WorkerType,
			url: Vec<u8>,
		},
		EnclaveRemoved {
			who: T::AccountId,
		},
		EnclaveAuthorized {
			worker_type: WorkerType,
			mrenclave: MrEnclave,
		},
		EnclaveUnauthorized {
			worker_type: WorkerType,
			mrenclave: MrEnclave,
		},
		OpaqueTaskPosted {
			request: RsaRequest,
		},
		ParentchainBlockProcessed {
			who: T::AccountId,
			block_number: BlockNumberFor<T>,
			block_hash: H256,
			task_merkle_root: H256,
		},
		SidechainBlockFinalized {
			who: T::AccountId,
			sidechain_block_number: SidechainBlockNumber,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// This operation needs the admin permission
		RequireAdminOrRoot,
		/// Failed to decode enclave signer.
		EnclaveSignerDecodeError,
		/// Sender does not match attested enclave in report.
		SenderIsNotAttestedEnclave,
		/// Verifying RA report failed.
		RemoteAttestationVerificationFailed,
		/// RA report is too old.
		RemoteAttestationTooOld,
		/// Invalid attestion type, e.g., an `Ignore` type under non-dev mode
		InvalidAttestationType,
		/// The enclave cannot attest, because its building mode is not allowed.
		InvalidSgxMode,
		/// The specified dcap provider is not yet supported
		DcapProviderNotSupported,
		/// The enclave doesn't exist.
		EnclaveNotExist,
		/// The enclave identifier doesn't exist.
		EnclaveIdentifierNotExist,
		/// The enclave identifier already exists.
		EnclaveIdentifierAlreadyExist,
		/// The worker type is unexpected, becuase e.g. when we try to re-register an
		/// existing enclave with a differnet worker type
		UnexpectedWorkerType,
		/// The shard doesn't match the enclave.
		WrongMrenclaveForShard,
		/// The worker url is too long.
		EnclaveUrlTooLong,
		/// The raw attestation data is too long.
		AttestationTooLong,
		/// The worker mode is unexpected, because e.g. a non-sidechain worker calls
		/// sidechain specific extrinsic
		UnexpectedWorkerMode,
		/// The authorized enclave doesn't exist.
		AuthorizedEnclaveNotExist,
		/// The authorized enclave already exists.
		AuthorizedEnclaveAlreadyExist,
		/// Enclave not authorized upon registration.
		EnclaveNotAuthorized,
		/// The provided collateral data is invalid.
		CollateralInvalid,
		/// MaxEnclaveIdentifier overflow.
		MaxEnclaveIdentifierOverflow,
		/// MaxAuthorizedEnclave overflow.
		MaxAuthorizedEnclaveOverflow,
		/// A proposed block is unexpected.
		ReceivedUnexpectedSidechainBlock,
		/// The value for the next finalization candidate is invalid.
		InvalidNextFinalizationCandidateBlockNumber,
	}

	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mode)]
	pub type Mode<T: Config> = StorageValue<_, OperationalMode, ValueQuery>;

	// records the enclave identifier list for each worker_type
	// T::AccountId is used as identifier as it's unique for each enclave
	#[pallet::storage]
	#[pallet::getter(fn enclave_identifier)]
	pub type EnclaveIdentifier<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		WorkerType,
		BoundedVec<T::AccountId, T::MaxEnclaveIdentifier>,
		ValueQuery,
	>;

	// registry that holds all registered enclaves, using T::AccountId as the key
	// having `worker_type` and `mrenclave` in each `Enclave` instance might seem a bit redundant,
	// but it increases flexibility where we **could** allow the same type of worker to have
	// distinct mrenclaves: e.g. when multiple versions of enclave are permitted in TEE cluster.
	//
	// It simplifies the lookup a bit too, otherwise we might need several storages.
	#[pallet::storage]
	#[pallet::getter(fn enclave_registry)]
	pub type EnclaveRegistry<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Enclave, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn allow_sgx_debug_mode)]
	pub type AllowSGXDebugMode<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn quoting_enclave_registry)]
	pub type QuotingEnclaveRegistry<T: Config> = StorageValue<_, QuotingEnclave, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tcb_info)]
	pub type TcbInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, Fmspc, TcbInfoOnChain, ValueQuery>;

	// keep track of a list of scheduled/allowed enchalves, mainly used for enclave updates,
	// can only be modified by AdminOrigin
	// (worker_type, sidechain_block_number) -> expected MrEnclave
	//
	// about the first time enclave registration:
	// prior to `register_enclave` this map needs to be populated with ((worker_type, 0),
	// expected-mrenclave), otherwise the registration will fail
	//
	// For NON-sidechain worker_type, we still use this storage to whitelist mrenclave, in this case
	// the `SidechainBlockNumber` is ignored - you could always set it to 0.
	//
	// Theorectically we could always push the enclave in `register_enclave`, but we want to
	// limit the mrenclave that can be registered, as the parachain is supposed to process enclaves
	// with specific mrenclaves.
	//
	// Update:
	// this storage is deprecated and will be removed (discarded) once we move to litentry-parachain
	// see https://linear.app/litentry/issue/P-882/simplify-scheduledenclave-usage
	//
	// use `AuthorizedEnclave` instead
	#[pallet::storage]
	#[pallet::getter(fn scheduled_enclave)]
	pub type ScheduledEnclave<T: Config> =
		StorageMap<_, Blake2_128Concat, (WorkerType, SidechainBlockNumber), MrEnclave, OptionQuery>;

	// Keep trace of a list of authorized (mr-)enclaves, enclave registration with non-authorized
	// mrenclave will be rejected
	//
	// Removing an mrenclave within `AuthorizedEnclave`` will cause the enclave to be removed from
	// the EnclaveRegistry too, which stops the sidechain from producing blocks as it will never be
	// able to claim the slot. Additionally, the vc-task-runner is informed to stop operating too.
	//
	// In this case, restarting the worker won't help, as the mrenclave is not authorized anymore,
	// so the registration will fail in the first place.
	#[pallet::storage]
	#[pallet::getter(fn authorized_enclave)]
	pub type AuthorizedEnclave<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		WorkerType,
		BoundedVec<MrEnclave, T::MaxAuthorizedEnclave>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn latest_sidechain_block_confirmation)]
	pub type LatestSidechainBlockConfirmation<T: Config> =
		StorageMap<_, Blake2_128Concat, ShardIdentifier, SidechainBlockConfirmation, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sidechain_block_finalization_candidate)]
	pub type SidechainBlockFinalizationCandidate<T: Config> =
		StorageMap<_, Blake2_128Concat, ShardIdentifier, SidechainBlockNumber, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub allow_sgx_debug_mode: bool,
		pub admin: Option<T::AccountId>,
		pub mode: OperationalMode,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { allow_sgx_debug_mode: false, admin: None, mode: OperationalMode::Production }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			AllowSGXDebugMode::<T>::put(self.allow_sgx_debug_mode);
			Mode::<T>::put(self.mode);
			if let Some(ref admin) = self.admin {
				Admin::<T>::put(admin);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		// Needed for the conversion of `mrenclave` to a `Hash`.
		// The condition holds for all known chains.
		<T as frame_system::Config>::Hash: From<[u8; 32]>,
	{
		/// Set the admin account
		///
		/// Weights should be 2 DB writes: 1 for mode and 1 for event
		#[pallet::call_index(0)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal))]
		pub fn set_admin(
			origin: OriginFor<T>,
			new_admin: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::SetAdminOrigin::ensure_origin(origin)?;
			Admin::<T>::put(new_admin.clone());
			Self::deposit_event(Event::AdminSet { new_admin: Some(new_admin) });
			Ok(Pays::No.into())
		}

		/// Set the mode
		///
		/// Weights should be 2 DB writes: 1 for mode and 1 for event
		#[pallet::call_index(1)]
		#[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal))]
		pub fn set_mode(
			origin: OriginFor<T>,
			new_mode: OperationalMode,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			Mode::<T>::put(new_mode);
			Self::deposit_event(Event::ModeSet { new_mode });
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::force_add_enclave())]
		pub fn force_add_enclave(
			origin: OriginFor<T>,
			who: T::AccountId,
			enclave: Enclave,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			Self::add_enclave_identifier_internal(enclave.worker_type, &who)?;
			Self::add_enclave(&who, &enclave)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::force_remove_enclave())]
		pub fn force_remove_enclave(
			origin: OriginFor<T>,
			who: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			Self::remove_enclave(&who)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::force_remove_enclave_by_mrenclave())]
		pub fn force_remove_enclave_by_mrenclave(
			origin: OriginFor<T>,
			mrenclave: MrEnclave,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			let accounts = EnclaveRegistry::<T>::iter()
				.filter_map(
					|(who, enclave)| {
						if enclave.mrenclave == mrenclave {
							Some(who)
						} else {
							None
						}
					},
				)
				.collect::<Vec<T::AccountId>>();

			for who in accounts.into_iter() {
				Self::remove_enclave(&who)?;
			}
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::force_remove_enclave_by_worker_type())]
		pub fn force_remove_enclave_by_worker_type(
			origin: OriginFor<T>,
			worker_type: WorkerType,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			let accounts = EnclaveRegistry::<T>::iter()
				.filter_map(
					|(who, enclave)| {
						if enclave.worker_type == worker_type {
							Some(who)
						} else {
							None
						}
					},
				)
				.collect::<Vec<T::AccountId>>();

			for who in accounts.into_iter() {
				Self::remove_enclave(&who)?;
			}

			Ok(Pays::No.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::force_add_authorized_enclave())]
		pub fn force_add_authorized_enclave(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			mrenclave: MrEnclave,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;

			AuthorizedEnclave::<T>::try_mutate(worker_type, |v| {
				ensure!(!v.contains(&mrenclave), Error::<T>::AuthorizedEnclaveAlreadyExist);
				v.try_push(mrenclave).map_err(|_| Error::<T>::MaxAuthorizedEnclaveOverflow)
			})?;

			Self::deposit_event(Event::EnclaveAuthorized { worker_type, mrenclave });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::force_remove_authorized_enclave())]
		pub fn force_remove_authorized_enclave(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			mrenclave: MrEnclave,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin.clone())?;

			AuthorizedEnclave::<T>::try_mutate(worker_type, |v| {
				ensure!(v.contains(&mrenclave), Error::<T>::AuthorizedEnclaveNotExist);
				v.retain(|e| e != &mrenclave);
				Ok::<(), DispatchErrorWithPostInfo>(())
			})?;

			Self::force_remove_enclave_by_mrenclave(origin, mrenclave)?;

			Self::deposit_event(Event::EnclaveUnauthorized { worker_type, mrenclave });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(8)]
		#[pallet::weight({
            match attestation_type {
                AttestationType::Ias => <T as Config>::WeightInfo::register_enclave_with_ias_attestation(),
                AttestationType::Dcap(_) => <T as Config>::WeightInfo::register_enclave_with_dcap_attestation(),
                AttestationType::Ignore => Weight::zero(),
            }
        })]
		pub fn register_enclave(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			worker_mode: WorkerMode,
			attestation: Vec<u8>,
			worker_url: Vec<u8>,
			shielding_pubkey: Option<Vec<u8>>,
			vc_pubkey: Option<Ed25519Public>,
			attestation_type: AttestationType,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(worker_url.len() <= MAX_URL_LEN, Error::<T>::EnclaveUrlTooLong);

			let mut enclave = Enclave::new(worker_type)
				.with_worker_mode(worker_mode)
				.with_url(worker_url)
				.with_shielding_pubkey(shielding_pubkey)
				.with_vc_pubkey(vc_pubkey)
				.with_attestation_type(attestation_type);

			match attestation_type {
				AttestationType::Ignore => {
					ensure!(
						Self::mode() == OperationalMode::Development,
						Error::<T>::InvalidAttestationType
					);
					enclave.mrenclave =
						<MrEnclave>::decode(&mut attestation.as_slice()).unwrap_or_default();
					enclave.last_seen_timestamp = Self::now().saturated_into();
					enclave.sgx_build_mode = SgxBuildMode::default();
				},
				AttestationType::Ias => {
					let report = Self::verify_ias(&sender, attestation)?;
					enclave.mrenclave = report.mr_enclave;
					enclave.last_seen_timestamp = report.timestamp;
					enclave.sgx_build_mode = report.build_mode;
				},
				AttestationType::Dcap(provider) => {
					ensure!(provider == DcapProvider::Intel, Error::<T>::DcapProviderNotSupported);
					let report = Self::verify_dcap(&sender, attestation)?;
					enclave.mrenclave = report.mr_enclave;
					enclave.last_seen_timestamp = report.timestamp;
					enclave.sgx_build_mode = report.build_mode;
				},
			};

			match Self::mode() {
				OperationalMode::Production | OperationalMode::Maintenance => {
					if !Self::allow_sgx_debug_mode()
						&& enclave.sgx_build_mode == SgxBuildMode::Debug
					{
						return Err(Error::<T>::InvalidSgxMode.into());
					}
					ensure!(
						AuthorizedEnclave::<T>::get(worker_type).contains(&enclave.mrenclave),
						Error::<T>::EnclaveNotAuthorized
					);
				},
				OperationalMode::Development => {
					AuthorizedEnclave::<T>::try_mutate(worker_type, |v| {
						if !v.contains(&enclave.mrenclave) {
							v.try_push(enclave.mrenclave)
								.map_err(|_| Error::<T>::MaxAuthorizedEnclaveOverflow)
						} else {
							Ok(())
						}
					})?;
				},
			};
			Self::add_enclave(&sender, &enclave)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::unregister_enclave())]
		pub fn unregister_enclave(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			Self::remove_enclave(&sender)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::register_quoting_enclave())]
		pub fn register_quoting_enclave(
			origin: OriginFor<T>,
			enclave_identity: Vec<u8>,
			signature: Vec<u8>,
			certificate_chain: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			// Quoting enclaves are registered globally and not for a specific sender
			let _ = ensure_signed(origin)?;
			let quoting_enclave =
				Self::verify_quoting_enclave(enclave_identity, signature, certificate_chain)?;
			<QuotingEnclaveRegistry<T>>::put(quoting_enclave);
			Ok(Pays::No.into())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::register_tcb_info())]
		pub fn register_tcb_info(
			origin: OriginFor<T>,
			tcb_info: Vec<u8>,
			signature: Vec<u8>,
			certificate_chain: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			// TCB info is registered globally and not for a specific sender
			let _ = ensure_signed(origin)?;
			let (fmspc, on_chain_info) =
				Self::verify_tcb_info(tcb_info, signature, certificate_chain)?;
			TcbInfo::<T>::insert(fmspc, on_chain_info);
			Ok(Pays::No.into())
		}

		// ===============================================================================
		// Following extrinsics are for runtime communication between parachain and worker
		// ===============================================================================

		#[pallet::call_index(20)]
		#[pallet::weight(<T as Config>::WeightInfo::post_opaque_task())]
		pub fn post_opaque_task(
			origin: OriginFor<T>,
			request: RsaRequest,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			Self::deposit_event(Event::OpaqueTaskPosted { request });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(21)]
		#[pallet::weight(<T as Config>::WeightInfo::parentchain_block_processed())]
		pub fn parentchain_block_processed(
			origin: OriginFor<T>,
			block_hash: H256,
			block_number: BlockNumberFor<T>,
			task_merkle_root: H256,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let mut enclave =
				EnclaveRegistry::<T>::get(&sender).ok_or(Error::<T>::EnclaveNotExist)?;
			enclave.last_seen_timestamp = Self::now().saturated_into();
			Self::deposit_event(Event::ParentchainBlockProcessed {
				who: sender,
				block_number,
				block_hash,
				task_merkle_root,
			});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(22)]
		#[pallet::weight(<T as Config>::WeightInfo::sidechain_block_imported())]
		pub fn sidechain_block_imported(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			block_number: u64,
			next_finalization_candidate_block_number: u64,
			block_header_hash: H256,
		) -> DispatchResultWithPostInfo {
			let confirmation = SidechainBlockConfirmation { block_number, block_header_hash };

			let sender = ensure_signed(origin)?;
			let mut sender_enclave =
				EnclaveRegistry::<T>::get(&sender).ok_or(Error::<T>::EnclaveNotExist)?;

			ensure!(
				sender_enclave.mrenclave.as_ref() == shard.as_ref(),
				Error::<T>::WrongMrenclaveForShard
			);

			ensure!(
				sender_enclave.worker_mode == WorkerMode::Sidechain,
				Error::<T>::UnexpectedWorkerMode
			);

			sender_enclave.last_seen_timestamp = Self::now().saturated_into();

			// Simple logic for now: only accept blocks from first registered enclave.
			let primary_enclave_identifier =
				EnclaveIdentifier::<T>::get(sender_enclave.worker_type)
					.first()
					.cloned()
					.ok_or(Error::<T>::EnclaveIdentifierNotExist)?;
			if sender != primary_enclave_identifier {
				log::debug!(
					"Ignore block confirmation from non primary enclave identifier: {:?}, primary: {:?}",
					sender, primary_enclave_identifier
				);
				return Ok(Pays::No.into());
			}

			let block_number = confirmation.block_number;
			let finalization_candidate_block_number =
				SidechainBlockFinalizationCandidate::<T>::try_get(shard).unwrap_or(1);

			ensure!(
				block_number == finalization_candidate_block_number,
				Error::<T>::ReceivedUnexpectedSidechainBlock
			);
			ensure!(
				next_finalization_candidate_block_number > finalization_candidate_block_number,
				Error::<T>::InvalidNextFinalizationCandidateBlockNumber
			);

			SidechainBlockFinalizationCandidate::<T>::insert(
				shard,
				next_finalization_candidate_block_number,
			);

			Self::finalize_block(sender, shard, confirmation);
			Ok(Pays::No.into())
		}

		// A wrapper to utility.call to waive the tx fee if the caller is tee-worker
		// the weight is copied from pallet_utility
		#[pallet::call_index(23)]
		#[pallet::weight({
			use pallet_utility::WeightInfo;
			let dispatch_infos = calls.iter().map(|call| call.get_dispatch_info()).collect::<Vec<_>>();
			let dispatch_weight = dispatch_infos.iter()
				.map(|di| di.weight)
				.fold(Weight::zero(), |total: Weight, weight: Weight| total.saturating_add(weight))
				.saturating_add(<T as pallet_utility::Config>::WeightInfo::batch(calls.len() as u32));
			let dispatch_class = {
				let all_operational = dispatch_infos.iter()
					.map(|di| di.class)
					.all(|class| class == DispatchClass::Operational);
				if all_operational {
					DispatchClass::Operational
				} else {
					DispatchClass::Normal
				}
			};
			(dispatch_weight, dispatch_class)
		})]
		pub fn batch(
			origin: OriginFor<T>,
			calls: Vec<<T as pallet_utility::Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin.clone())?;
			let _ = EnclaveRegistry::<T>::get(&sender).ok_or(Error::<T>::EnclaveNotExist)?;
			let _ = pallet_utility::Pallet::<T>::batch(origin, calls)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(24)]
		#[pallet::weight(<T as Config>::WeightInfo::add_enclave_identifier())]
		pub fn add_enclave_identifier(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			who: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			Self::add_enclave_identifier_internal(worker_type, &who)?;
			Ok(Pays::No.into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn ensure_admin_or_root(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
		ensure!(
			ensure_root(origin.clone()).is_ok() || Some(ensure_signed(origin)?) == Self::admin(),
			Error::<T>::RequireAdminOrRoot
		);
		Ok(().into())
	}

	pub fn add_enclave_identifier_internal(
		worker_type: WorkerType,
		who: &T::AccountId,
	) -> Result<(), Error<T>> {
		EnclaveIdentifier::<T>::try_mutate(worker_type, |v| {
			ensure!(!v.contains(who), Error::<T>::EnclaveIdentifierAlreadyExist);
			v.try_push(who.clone()).map_err(|_| Error::<T>::MaxEnclaveIdentifierOverflow)
		})
	}

	pub fn add_enclave(sender: &T::AccountId, enclave: &Enclave) -> Result<(), Error<T>> {
		if let Some(old_enclave) = EnclaveRegistry::<T>::get(sender) {
			ensure!(
				old_enclave.worker_type == enclave.worker_type,
				Error::<T>::UnexpectedWorkerType
			);
		}

		match Self::mode() {
			OperationalMode::Production | OperationalMode::Maintenance => {
				ensure!(
					EnclaveIdentifier::<T>::get(enclave.worker_type).contains(sender),
					Error::<T>::EnclaveIdentifierNotExist
				);
			},
			OperationalMode::Development => {
				if EnclaveRegistry::<T>::get(sender).is_none() {
					Self::add_enclave_identifier_internal(enclave.worker_type, sender)?;
				}
			},
		};

		EnclaveRegistry::<T>::insert(sender, enclave);
		Self::deposit_event(Event::<T>::EnclaveAdded {
			who: sender.clone(),
			worker_type: enclave.worker_type,
			url: enclave.url.clone(),
		});
		Ok(())
	}

	fn remove_enclave(sender: &T::AccountId) -> DispatchResultWithPostInfo {
		let enclave = EnclaveRegistry::<T>::get(sender).ok_or(Error::<T>::EnclaveNotExist)?;

		EnclaveIdentifier::<T>::try_mutate(enclave.worker_type, |v| {
			ensure!(v.contains(sender), Error::<T>::EnclaveIdentifierNotExist);
			v.retain(|e| e != sender);
			Ok::<(), DispatchErrorWithPostInfo>(())
		})?;

		EnclaveRegistry::<T>::remove(sender);
		Self::deposit_event(Event::<T>::EnclaveRemoved { who: sender.clone() });
		Ok(().into())
	}

	pub fn enclave_count(worker_type: WorkerType) -> u32 {
		EnclaveIdentifier::<T>::get(worker_type).iter().count() as u32
	}

	fn verify_ias(
		sender: &T::AccountId,
		ra_report: Vec<u8>,
	) -> Result<SgxReport, DispatchErrorWithPostInfo> {
		ensure!(ra_report.len() <= MAX_RA_REPORT_LEN, Error::<T>::AttestationTooLong);
		let report = verify_ias_report(&ra_report)
			.map_err(|_| Error::<T>::RemoteAttestationVerificationFailed)?;

		let enclave_signer = T::AccountId::decode(&mut &report.pubkey[..])
			.map_err(|_| Error::<T>::EnclaveSignerDecodeError)?;

		ensure!(sender == &enclave_signer, Error::<T>::SenderIsNotAttestedEnclave);

		Self::ensure_timestamp_within_24_hours(report.timestamp)?;
		Ok(report)
	}

	fn verify_dcap(
		sender: &T::AccountId,
		dcap_quote: Vec<u8>,
	) -> Result<SgxReport, DispatchErrorWithPostInfo> {
		ensure!(dcap_quote.len() <= MAX_DCAP_QUOTE_LEN, Error::<T>::AttestationTooLong);
		let timestamp = Self::now();
		let qe = <QuotingEnclaveRegistry<T>>::get();
		let (fmspc, tcb_info, report) =
			verify_dcap_quote(&dcap_quote, timestamp.saturated_into(), &qe).map_err(|e| {
				log::warn!("verify_dcap_quote failed: {:?}", e);
				Error::<T>::RemoteAttestationVerificationFailed
			})?;

		let tcb_info_on_chain = <TcbInfo<T>>::get(fmspc);
		ensure!(tcb_info_on_chain.verify_examinee(&tcb_info), "tcb_info is outdated");

		let enclave_signer = T::AccountId::decode(&mut &report.pubkey[..])
			.map_err(|_| Error::<T>::EnclaveSignerDecodeError)?;
		ensure!(sender == &enclave_signer, Error::<T>::SenderIsNotAttestedEnclave);

		Ok(report)
	}

	fn verify_quoting_enclave(
		enclave_identity: Vec<u8>,
		signature: Vec<u8>,
		certificate_chain: Vec<u8>,
	) -> Result<QuotingEnclave, DispatchErrorWithPostInfo> {
		let verification_time: u64 = Self::now().saturated_into();
		let certs = extract_certs(&certificate_chain);
		ensure!(certs.len() >= 2, "Certificate chain must have at least two certificates");
		let intermediate_slices: Vec<webpki::types::CertificateDer> =
			certs[1..].iter().map(|c| c.as_slice().into()).collect();
		let leaf_cert_der = webpki::types::CertificateDer::from(certs[0].as_slice());
		let leaf_cert = webpki::EndEntityCert::try_from(&leaf_cert_der)
			.map_err(|_| "Failed to parse leaf certificate")?;
		verify_certificate_chain(&leaf_cert, &intermediate_slices, verification_time)?;
		let enclave_identity =
			deserialize_enclave_identity(&enclave_identity, &signature, &leaf_cert)?;

		if enclave_identity.is_valid(verification_time.try_into().unwrap()) {
			Ok(enclave_identity.to_quoting_enclave())
		} else {
			Err(Error::<T>::CollateralInvalid.into())
		}
	}

	pub fn verify_tcb_info(
		tcb_info: Vec<u8>,
		signature: Vec<u8>,
		certificate_chain: Vec<u8>,
	) -> Result<(Fmspc, TcbInfoOnChain), DispatchErrorWithPostInfo> {
		let verification_time: u64 = Self::now().saturated_into();
		let certs = extract_certs(&certificate_chain);
		ensure!(certs.len() >= 2, "Certificate chain must have at least two certificates");
		let intermediate_slices: Vec<webpki::types::CertificateDer> =
			certs[1..].iter().map(|c| c.as_slice().into()).collect();
		let leaf_cert_der = webpki::types::CertificateDer::from(certs[0].as_slice());
		let leaf_cert = webpki::EndEntityCert::try_from(&leaf_cert_der)
			.map_err(|_| "Failed to parse leaf certificate")?;
		verify_certificate_chain(&leaf_cert, &intermediate_slices, verification_time)?;
		let tcb_info = deserialize_tcb_info(&tcb_info, &signature, &leaf_cert)?;
		if tcb_info.is_valid(verification_time.try_into().unwrap()) {
			Ok(tcb_info.to_chain_tcb_info())
		} else {
			Err(Error::<T>::CollateralInvalid.into())
		}
	}

	fn ensure_timestamp_within_24_hours(report_timestamp: u64) -> DispatchResultWithPostInfo {
		let elapsed_time = Self::now()
			.checked_sub(&T::Moment::saturated_from(report_timestamp))
			.ok_or("Underflow while calculating elapsed time since report creation")?;

		if elapsed_time < T::MomentsPerDay::get() {
			Ok(().into())
		} else {
			Err(Error::<T>::RemoteAttestationTooOld.into())
		}
	}

	fn finalize_block(
		sender: T::AccountId,
		shard: ShardIdentifier,
		confirmation: SidechainBlockConfirmation,
	) {
		LatestSidechainBlockConfirmation::<T>::insert(shard, confirmation);
		Self::deposit_event(Event::SidechainBlockFinalized {
			who: sender,
			sidechain_block_number: confirmation.block_number,
		});
	}

	fn now() -> T::Moment {
		pallet_timestamp::Pallet::<T>::now()
	}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(any(test, feature = "runtime-benchmarks", feature = "test-util"))]
pub mod test_util;
