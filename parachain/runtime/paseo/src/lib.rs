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
#![allow(clippy::identity_op)]
#![allow(clippy::items_after_test_module)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "512"]

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

use core_primitives::LITENTRY_PARA_ID;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use cumulus_primitives_core::{AggregateMessageOrigin, InteriorLocation, PalletInstance, ParaId};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		tokens::UnityOrOuterConversion, ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, Contains, ContainsLengthBound,
		EnsureOrigin, Everything, FindAuthor, FromContains, InstanceFilter, OnFinalize, SortedMembers, TransformOrigin,
		WithdrawReasons,
	},
	weights::{constants::RocksDbWeight, ConstantMultiplier, Weight},
	ConsensusEngineId, PalletId,
};
use frame_system::EnsureRoot;
use hex_literal::hex;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

use runtime_common::EnsureEnclaveSigner;
// for TEE
pub use pallet_balances::Call as BalancesCall;

use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};

use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, RuntimeDebug, H160, H256, U256};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto,
		DispatchInfoOf, Dispatchable, PostDispatchInfoOf, UniqueSaturatedInto,
	},
	transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
	ApplyExtrinsicResult,
};
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// XCM Imports
use xcm_builder::PayOverXcm;
use xcm_executor::XcmExecutor;
use xcm::latest::prelude::Junction;

pub use constants::currency::deposit;
pub use core_primitives::{
	opaque, teebag::OperationalMode as TeebagOperationalMode, AccountId, Amount, AssetId, Balance,
	BlockNumber, DefaultOmniAccountConverter, Hash, Header, Identity, Nonce, Signature, DAYS,
	HOURS, MINUTES, SLOT_DURATION,
};
pub use runtime_common::currency::*;

use runtime_common::{
	impl_runtime_transaction_payment_fees, prod_or_fast, AccountIndex, BlockHashCount, BlockLength,
	CouncilInstance, CouncilMembershipInstance, DeveloperCommitteeInstance,
	DeveloperCommitteeMembershipInstance, EnsureOmniAccount, EnsureRootOrAllCouncil,
	EnsureRootOrAllTechnicalCommittee, EnsureRootOrHalfCouncil, EnsureRootOrHalfTechnicalCommittee,
	EnsureRootOrTwoThirdsCouncil, EnsureRootOrTwoThirdsTechnicalCommittee,
	IMPExtrinsicWhitelistInstance, NegativeImbalance, RuntimeBlockWeights, SlowAdjustingFeeUpdate,
	TechnicalCommitteeInstance, TechnicalCommitteeMembershipInstance,
	VCMPExtrinsicWhitelistInstance, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO, WEIGHT_PER_GAS,
	WEIGHT_TO_FEE_FACTOR,
	impls::{ContainsParts, LocatableAssetConverter, VersionedLocationConverter}
};
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

use pallet_ethereum::{Call::transact, PostLogContent, TransactionStatus};
use pallet_evm::{
	EVMCurrencyAdapter, FeeCalculator, GasWeightMapping,
	OnChargeEVMTransaction as OnChargeEVMTransactionT, Runner,
};
// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod asset_config;
pub mod constants;
pub mod precompiles;

#[cfg(test)]
mod tests;
pub mod weights;
pub mod xcm_config;

pub use precompiles::RococoNetworkPrecompiles;
pub type Precompiles = RococoNetworkPrecompiles<Runtime>;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic =
	fp_self_contained::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, H160>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	// see https://github.com/paritytech/substrate/pull/10043
	//
	// With this type the hooks of pallets will be executed
	// in the order that they are declared in `construct_runtime!`
	// it was reverse order before.
	// See the comment before collation related pallets too.
	AllPalletsWithSystem,
>;

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => {
				call.pre_dispatch_self_contained(info, dispatch_info, len)
			},
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				)))
			},
			_ => None,
		}
	}
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	// Note:
	// It's important to match `paseo-parachain-runtime`, which is runtime pkg name
	spec_name: create_runtime_str!("paseo-parachain"),
	impl_name: create_runtime_str!("paseo-parachain"),
	authoring_version: 1,
	// same versioning-mechanism as polkadot: use last digit for minor updates
	spec_version: 9202,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// A timestamp: milliseconds since the unix epoch.
pub type Moment = u64;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// using generic substrate prefix
	pub const SS58Prefix: u16 = 42;
}

impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type RuntimeCall = RuntimeCall;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Nonce = Nonce;
	type Block = Block;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type BlockHashCount = BlockHashCount;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = BaseCallFilter;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = BlockLength;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type RuntimeTask = ();
	type SingleBlockMigrations = ();
	type MultiBlockMigrator= ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

impl pallet_asset_rate::Config for Runtime {
	type WeightInfo = weights::pallet_asset_rate::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type CreateOrigin = EnsureRoot<AccountId>;
	type RemoveOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type AssetKind = <Runtime as pallet_treasury::Config>::AssetKind;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = runtime_common::impls::benchmarks::AssetRateArguments;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	/// Fully permissioned proxy. Can execute any call on behalf of _proxied_.
	#[codec(index = 0)]
	Any,
	/// Can execute any call that does not transfer funds, including asset transfers.
	#[codec(index = 1)]
	NonTransfer,
	/// Proxy with the ability to reject time-delay proxy announcements.
	#[codec(index = 2)]
	CancelProxy,
	/// Collator selection proxy. Can execute calls related to collator selection mechanism.
	#[codec(index = 3)]
	Collator,
	/// Governance
	#[codec(index = 4)]
	Governance,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances(..)
					| RuntimeCall::Vesting(pallet_vesting::Call::vested_transfer { .. })
			),
			ProxyType::CancelProxy => matches!(
				c,
				RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
					| RuntimeCall::Utility(..)
					| RuntimeCall::Multisig(..)
			),
			ProxyType::Collator => matches!(
				c,
				RuntimeCall::ParachainStaking(..)
					| RuntimeCall::Utility(..)
					| RuntimeCall::Multisig(..)
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..)
					| RuntimeCall::Council(..)
					| RuntimeCall::TechnicalCommittee(..)
					| RuntimeCall::Treasury(..)
					| RuntimeCall::DeveloperCommittee(..)
			),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (ParachainStaking,);
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRootOrAllCouncil;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
	type OriginPrivilegeCmp = frame_support::traits::EqualPrivilegeOnly;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = 1 * DOLLARS;
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = weights::pallet_preimage::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRootOrAllCouncil;
	type Consideration = frame_support::traits::fungible::HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		frame_support::traits::LinearStoragePrice<
			PreimageBaseDeposit,
			PreimageByteDeposit,
			Balance,
		>,
	>;
}

parameter_types! {
	pub const IndexDeposit: Balance = 1 * DOLLARS;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = WEIGHT_TO_FEE_FACTOR; // 10^6
	pub const WeightToFeeFactor: Balance = WEIGHT_TO_FEE_FACTOR; // 10^6
}
impl_runtime_transaction_payment_fees!(constants);

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction =
		pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type WeightToFee = ConstantMultiplier<Balance, WeightToFeeFactor>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
}

parameter_types! {
	pub LaunchPeriod: BlockNumber = prod_or_fast!(10 * MINUTES, 5 * MINUTES, "ROCOCO_LAUNCHPERIOD");
	pub VotingPeriod: BlockNumber = prod_or_fast!(10 * MINUTES, 5 * MINUTES, "ROCOCO_VOTINGPERIOD");
	pub FastTrackVotingPeriod: BlockNumber = prod_or_fast!(8 * MINUTES, 2 * MINUTES, "ROCOCO_FASTTRACKVOTINGPERIOD");
	pub const InstantAllowed: bool = true;
	pub const MinimumDeposit: Balance = 100 * DOLLARS;
	pub EnactmentPeriod: BlockNumber = prod_or_fast!(5 * MINUTES, 2 * MINUTES, "ROCOCO_ENACTMENTPERIOD");
	pub CooloffPeriod: BlockNumber = prod_or_fast!(10 * MINUTES, 2 * MINUTES, "ROCOCO_COOLOFFPERIOD");
}

impl pallet_democracy::Config for Runtime {
	type Preimages = Preimage;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfCouncil;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureRootOrTwoThirdsCouncil;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureRootOrAllCouncil;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type InstantOrigin = EnsureRootOrAllTechnicalCommittee;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureRootOrTwoThirdsCouncil;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
	type BlacklistOrigin = EnsureRootOrAllCouncil;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;
	type CooloffPeriod = CooloffPeriod;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = ConstU32<100>;
	type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
	type MaxProposals = ConstU32<100>;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const CouncilDefaultMaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<CouncilInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = CouncilDefaultMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxProposalWeight;
}

impl pallet_membership::Config<CouncilMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRootOrTwoThirdsCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
	type MembershipInitialized = Council;
	type MembershipChanged = Council;
	type MaxMembers = CouncilDefaultMaxMembers;
	type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
}

impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = CouncilDefaultMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxProposalWeight;
}

impl pallet_membership::Config<TechnicalCommitteeMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRootOrTwoThirdsCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = CouncilDefaultMaxMembers;
	type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

impl pallet_collective::Config<DeveloperCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = CouncilDefaultMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxProposalWeight;
}

impl pallet_membership::Config<DeveloperCommitteeMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRootOrTwoThirdsCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
	type MembershipInitialized = DeveloperCommittee;
	type MembershipChanged = DeveloperCommittee;
	type MaxMembers = CouncilDefaultMaxMembers;
	type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
	pub const ProposalBondMaximum: Balance = 20 * DOLLARS;
	pub SpendPeriod: BlockNumber = prod_or_fast!(10 * MINUTES, 2 * MINUTES, "ROCOCO_SPENDPERIOD");
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");

	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(5);
	pub TipReportDepositBase: Balance = deposit(1, 0);
	pub BountyDepositBase: Balance = deposit(1, 0);
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 35 * DAYS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub CuratorDepositMin: Balance = DOLLARS;
	pub CuratorDepositMax: Balance = 100 * DOLLARS;
	pub BountyValueMinimum: Balance = 5 * DOLLARS;
	pub DataDepositPerByte: Balance = deposit(0, 1);
	pub const MaximumReasonLength: u32 = 8192;
	pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
	pub TreasuryInteriorLocation: InteriorLocation = PalletInstance(18).into();
}

pub struct EnsureRootOrTwoThirdsCouncilWrapper;
impl EnsureOrigin<RuntimeOrigin> for EnsureRootOrTwoThirdsCouncilWrapper {
	type Success = Balance;
	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		match EnsureRootOrTwoThirdsCouncil::try_origin(o) {
			Ok(_) => Ok(Balance::max_value()),
			Err(o) => Err(o),
		}
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrTwoThirdsCouncil;
	type RejectOrigin = EnsureRootOrHalfCouncil;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	// Once passed, at most all is allowed to be spent
	type SpendOrigin = EnsureRootOrTwoThirdsCouncilWrapper;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	// Rcococo bounty enabled
	type SpendFunds = Bounties;
	type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
	type MaxApprovals = ConstU32<100>;
	type AssetKind = u32;
	type Beneficiary = AccountId;
	type BeneficiaryLookup = Indices;
	type Paymaster = PayOverXcm<
		TreasuryInteriorLocation,
		crate::xcm_config::XcmRouter,
		crate::PolkadotXcm,
		ConstU32<{ 6 * HOURS }>,
		Self::Beneficiary,
		Self::AssetKind,
		LocatableAssetConverter,
		VersionedLocationConverter,
	>;
	type BalanceConverter = UnityOrOuterConversion<
		ContainsParts<
			FromContains<
				xcm_builder::IsChildSystemParachain<ParaId>,
				xcm_builder::IsParentsOnly<ConstU8<1>>,
			>,
		>,
		AssetRate,
	>;
	type PayoutPeriod = PayoutSpendPeriod;
}

pub struct CouncilProvider;
impl SortedMembers<AccountId> for CouncilProvider {
	fn contains(who: &AccountId) -> bool {
		Council::is_member(who)
	}

	fn sorted_members() -> Vec<AccountId> {
		// pallet_collective::pallet::Members::<T, I>::get()
		// TODO
		unimplemented!()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add(_: &AccountId) {
		unimplemented!()
	}
}

impl ContainsLengthBound for CouncilProvider {
	fn max_len() -> usize {
		CouncilDefaultMaxMembers::get() as usize
	}
	fn min_len() -> usize {
		0
	}
}

impl pallet_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyValueMinimum = BountyValueMinimum;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
	type ChildBountyManager = ();
}

impl pallet_tips::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = CouncilProvider;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type WeightInfo = ();
}

parameter_types! {
	pub const ByteDeposit: Balance = deposit(0, 1);
	pub const MaxAdditionalFields: u32 = 100;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	// Add one item in storage and take 258 bytes
	type BasicDeposit = ConstU128<{ deposit(1, 258) }>;
	// Add one item in storage and take 53 bytes
	type SubAccountDeposit = ConstU128<{ deposit(1, 53) }>;
	type MaxSubAccounts = ConstU32<100>;
	type MaxRegistrars = ConstU32<20>;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type RegistrarOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
	type ByteDeposit = ByteDeposit;
	type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
	type OffchainSignature = Signature;
	type SigningPublicKey = <Signature as sp_runtime::traits::Verify>::Signer;
	type UsernameAuthorityOrigin = EnsureRoot<Self::AccountId>;
	type PendingUsernameExpiration = ConstU32<{ 7 * DAYS }>;
	type MaxSuffixLength = ConstU32<7>;
	type MaxUsernameLength = ConstU32<32>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_account_fix::Config for Runtime {
	type Currency = Balances;
	type IncConsumerOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type AddBalanceOrigin = EnsureRoot<AccountId>;
	type BurnOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	// We use pallet_xcm to confirm the version of xcm
	type VersionWrapper = PolkadotXcm;
	type ControllerOrigin = EnsureRootOrAllCouncil;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = weights::cumulus_pallet_xcmp_queue::WeightInfo<Runtime>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
}

parameter_types! {
	/// All messages that came into the `DmpSink`.
	pub static RecordedMessages: Vec<Vec<u8>> = vec![];
}

/// Can be used as [`Config::DmpSink`] to record all messages that came in.
pub struct RecordingDmpSink;
impl frame_support::traits::HandleMessage for RecordingDmpSink {
	type MaxMessageLen = ConstU32<16>;

	fn handle_message(msg: frame_support::BoundedSlice<u8, Self::MaxMessageLen>) {
		RecordedMessages::mutate(|n| n.push(msg.to_vec()));
	}

	fn handle_messages<'a>(_: impl Iterator<Item = frame_support::BoundedSlice<'a, u8, Self::MaxMessageLen>>) {
		unimplemented!()
	}

	fn sweep_queue() {
		unimplemented!()
	}

	fn footprint() -> frame_support::traits::QueueFootprint {
		unimplemented!()
	}
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// TODO need to generate
	type WeightInfo = ();
	// type WeightInfo = weights::cumulus_pallet_dmp_queue::WeightInfo<Runtime>;
	type DmpSink = RecordingDmpSink;
}

parameter_types! {
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = ParachainStaking;
	type NextSessionRotation = ParachainStaking;
	type SessionManager = ParachainStaking;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Self>;
}

parameter_types! {
	/// Default fixed percent a collator takes off the top of due rewards
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(0);
	/// Default percent of inflation set aside for parachain bond every round
	pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(0);
	pub const MinCollatorStk: Balance = 50 * DOLLARS;
	pub const MinCandidateStk: Balance = 50 * DOLLARS;
	pub const MinDelegation: Balance = 50 * DOLLARS;
	pub const MinDelegatorStk: Balance = 50 * DOLLARS;
}

impl pallet_parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = EnsureRootOrAllCouncil;
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	type MinBlocksPerRound = ConstU32<{ prod_or_fast!(2 * MINUTES, 2) }>;
	/// Blocks per round
	type DefaultBlocksPerRound = ConstU32<{ prod_or_fast!(2 * MINUTES, 2) }>;
	/// Rounds before the collator leaving the candidates request can be executed
	type LeaveCandidatesDelay = ConstU32<{ prod_or_fast!(8, 1) }>;
	/// Rounds before the candidate bond increase/decrease can be executed
	type CandidateBondLessDelay = ConstU32<{ prod_or_fast!(8, 1) }>;
	/// Rounds before the delegator exit can be executed
	type LeaveDelegatorsDelay = ConstU32<{ prod_or_fast!(8, 1) }>;
	/// Rounds before the delegator revocation can be executed
	type RevokeDelegationDelay = ConstU32<{ prod_or_fast!(8, 1) }>;
	/// Rounds before the delegator bond increase/decrease can be executed
	type DelegationBondLessDelay = ConstU32<{ prod_or_fast!(8, 1) }>;
	/// Rounds before the reward is paid
	type RewardPaymentDelay = ConstU32<2>;
	/// Minimum collators selected per round, default at genesis and minimum forever after
	type MinSelectedCandidates = ConstU32<1>;
	/// Maximum top delegations per candidate
	type MaxTopDelegationsPerCandidate = ConstU32<1000>;
	/// Maximum bottom delegations per candidate
	type MaxBottomDelegationsPerCandidate = ConstU32<200>;
	/// Maximum delegations per delegator
	type MaxDelegationsPerDelegator = ConstU32<100>;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
	/// Minimum stake required to become a collator
	type MinCollatorStk = MinCollatorStk;
	/// Minimum stake required to be reserved to be a candidate
	type MinCandidateStk = MinCandidateStk;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegation = MinDelegation;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegatorStk = MinDelegatorStk;
	type OnCollatorPayout = ();
	type OnNewRound = ();
	type WeightInfo = weights::pallet_parachain_staking::WeightInfo<Runtime>;
	type IssuanceAdapter = AssetsHandler;
	type OnAllDelegationRemoved = ScoreStaking;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 10 * CENTS;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
			WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type BlockNumberProvider = System;
}

parameter_types! {
	pub const BridgeChainId: u8 = 3;
	pub const ProposalLifetime: BlockNumber = 50400; // ~7 days
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl pallet_chain_bridge::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgeCommitteeOrigin = EnsureRootOrHalfCouncil;
	type Proposal = RuntimeCall;
	type BridgeChainId = BridgeChainId;
	type Balance = Balance;
	type ProposalLifetime = ProposalLifetime;
	type WeightInfo = weights::pallet_chain_bridge::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MaximumIssuance: Balance = 20_000_000 * DOLLARS;
	// Ethereum LIT total issuance in parachain decimal form
	pub const ExternalTotalIssuance: Balance = 100_000_000 * DOLLARS;
	// bridge::derive_resource_id(1, &bridge::hashing::blake2_128(b"LIT"));
	pub const NativeTokenResourceId: [u8; 32] = hex!("00000000000000000000000000000063a7e2be78898ba83824b0c0cc8dfb6001");
}

pub struct TransferAssetsAnyone;
impl SortedMembers<AccountId> for TransferAssetsAnyone {
	fn sorted_members() -> Vec<AccountId> {
		vec![]
	}

	fn contains(_who: &AccountId) -> bool {
		true
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add(_: &AccountId) {
		unimplemented!()
	}
}

impl pallet_bridge_transfer::Config for Runtime {
	type BridgeHandler = AssetsHandler;
	type BridgeOrigin = pallet_chain_bridge::EnsureBridge<Runtime>;
	type TransferAssetsMembers = TransferAssetsAnyone;
	type WeightInfo = weights::pallet_bridge_transfer::WeightInfo<Runtime>;
}

impl pallet_assets_handler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type TreasuryAccount = TreasuryAccount;
	type SetMaximumIssuanceOrigin = EnsureRootOrHalfCouncil;
	type DefaultMaximumIssuance = MaximumIssuance;
	type ExternalTotalIssuance = ExternalTotalIssuance;
}

impl pallet_extrinsic_filter::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = EnsureRootOrHalfTechnicalCommittee;
	type NormalModeFilter = NormalModeFilter;
	type SafeModeFilter = SafeModeFilter;
	type TestModeFilter = Everything;
	type WeightInfo = weights::pallet_extrinsic_filter::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MomentsPerDay: Moment = 86_400_000; // [ms/d]
}

impl pallet_teebag::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MomentsPerDay = MomentsPerDay;
	type SetAdminOrigin = EnsureRootOrHalfCouncil;
	type MaxEnclaveIdentifier = ConstU32<3>;
	type MaxAuthorizedEnclave = ConstU32<5>;
	type WeightInfo = weights::pallet_teebag::WeightInfo<Runtime>;
}

impl pallet_identity_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type TEECallOrigin = EnsureEnclaveSigner<Runtime>;
	type DelegateeAdminOrigin = EnsureRootOrAllCouncil;
	type ExtrinsicWhitelistOrigin = IMPExtrinsicWhitelist;
	type MaxOIDCClientRedirectUris = ConstU32<10>;
}

impl pallet_omni_account::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type TEECallOrigin = EnsureEnclaveSigner<Runtime>;
	type MaxAccountStoreLength = ConstU32<64>;
	type OmniAccountOrigin = EnsureOmniAccount;
	type OmniAccountConverter = DefaultOmniAccountConverter;
}

impl pallet_bitacross::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type TEECallOrigin = EnsureEnclaveSigner<Runtime>;
	type SetAdminOrigin = EnsureRootOrAllCouncil;
}

impl pallet_evm_assertions::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssertionId = H160;
	type ContractDevOrigin = pallet_collective::EnsureMember<AccountId, DeveloperCommitteeInstance>;
	type TEECallOrigin = EnsureEnclaveSigner<Runtime>;
}

impl pallet_group::Config<IMPExtrinsicWhitelistInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type GroupManagerOrigin = EnsureRootOrAllCouncil;
}

impl pallet_vc_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_vc_management::WeightInfo<Runtime>;
	type TEECallOrigin = EnsureEnclaveSigner<Runtime>;
	type SetAdminOrigin = EnsureRootOrHalfCouncil;
	type DelegateeAdminOrigin = EnsureRootOrAllCouncil;
	type ExtrinsicWhitelistOrigin = VCMPExtrinsicWhitelist;
}

impl pallet_group::Config<VCMPExtrinsicWhitelistInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type GroupManagerOrigin = EnsureRootOrAllCouncil;
}

// For OnChargeEVMTransaction implementation
type CurrencyAccountId<T> = <T as frame_system::Config>::AccountId;
type BalanceFor<T> =
	<<T as pallet_evm::Config>::Currency as Currency<CurrencyAccountId<T>>>::Balance;
type PositiveImbalanceFor<T> =
	<<T as pallet_evm::Config>::Currency as Currency<CurrencyAccountId<T>>>::PositiveImbalance;
type NegativeImbalanceFor<T> =
	<<T as pallet_evm::Config>::Currency as Currency<CurrencyAccountId<T>>>::NegativeImbalance;
pub struct OnChargeEVMTransaction<OU>(sp_std::marker::PhantomData<OU>);
impl<T, OU> OnChargeEVMTransactionT<T> for OnChargeEVMTransaction<OU>
where
	T: pallet_evm::Config,
	PositiveImbalanceFor<T>: Imbalance<BalanceFor<T>, Opposite = NegativeImbalanceFor<T>>,
	NegativeImbalanceFor<T>: Imbalance<BalanceFor<T>, Opposite = PositiveImbalanceFor<T>>,
	OU: OnUnbalanced<NegativeImbalanceFor<T>>,
	U256: UniqueSaturatedInto<BalanceFor<T>>,
{
	type LiquidityInfo = Option<NegativeImbalanceFor<T>>;

	fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
		EVMCurrencyAdapter::<<T as pallet_evm::Config>::Currency, ()>::withdraw_fee(who, fee)
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		base_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) -> Self::LiquidityInfo {
		<EVMCurrencyAdapter<<T as pallet_evm::Config>::Currency, OU> as OnChargeEVMTransactionT<
			T,
		>>::correct_and_deposit_fee(who, corrected_fee, base_fee, already_withdrawn)
	}
	// This is the only difference of OnChargeEVMTransaction regarding EVMCurrencyAdapter
	// We can use parachain TransactionPayment logic to handle evm tip
	fn pay_priority_fee(tip: Self::LiquidityInfo) {
		if let Some(tip) = tip {
			OU::on_unbalanced(tip);
		}
	}
}

pub struct TransactionPaymentAsGasPrice;
impl FeeCalculator for TransactionPaymentAsGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		// We do not want to involve Transaction Payment Multiplier here
		// It will biased normal transfer (base weight is not biased by Multiplier) too much for
		// Ethereum tx
		// This is hardcoded ConstantMultiplier<Balance, WeightToFeeFactor>, WeightToFeeFactor =
		// MILLICENTS / 10
		let weight_to_fee: u128 = WEIGHT_TO_FEE_FACTOR;
		let min_gas_price = weight_to_fee.saturating_mul(WEIGHT_PER_GAS as u128);
		(min_gas_price.into(), <Runtime as frame_system::Config>::DbWeight::get().reads(1))
	}
}

parameter_types! {
	pub WeightPerGas: Weight = Weight::from_parts(WEIGHT_PER_GAS, 0);
	pub ChainId: u64 = LITENTRY_PARA_ID.into();
	pub BlockGasLimit: U256 = U256::from(
		NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT.ref_time() / WEIGHT_PER_GAS
	);
	pub PrecompilesValue: Precompiles = RococoNetworkPrecompiles::<_>::new();
	// BlockGasLimit / MAX_POV_SIZE
	pub GasLimitPovSizeRatio: u64 = 4;
}

pub struct FindAuthorTruncated<T>(sp_std::marker::PhantomData<T>);
impl<T: pallet_aura::Config> FindAuthor<H160> for FindAuthorTruncated<T>
where
	pallet_aura::Pallet<T>: FindAuthor<u32>,
{
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = pallet_aura::Pallet::<T>::find_author(digests) {
			let authority_id =
			pallet_aura::Authorities::<Runtime>::get()[author_index as usize].clone();
			return Some(H160::from_slice(&authority_id.encode()[4..24]));
		}

		None
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = TransactionPaymentAsGasPrice;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressTruncated;
	type WithdrawOrigin = pallet_evm::EnsureAddressTruncated;
	// From evm address to parachain address
	type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ChainId;
	type OnChargeTransaction = OnChargeEVMTransaction<DealWithFees<Runtime>>;
	type BlockGasLimit = BlockGasLimit;
	type Timestamp = Timestamp;
	type OnCreate = ();
	type FindAuthor = FindAuthorTruncated<Runtime>;
	// BlockGasLimit / MAX_POV_SIZE
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type WeightInfo = weights::pallet_evm::WeightInfo<Runtime>;
	type SuicideQuickClearLimit = ConstU32<0>;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	// Maximum length (in bytes) of revert message to include in Executed event
	type ExtraDataLength = ConstU32<256>;
}

parameter_types! {
	pub const DefaultYearlyInflation: Perbill = Perbill::from_perthousand(5);
}

pub struct IdentityAccountIdConvert;

impl pallet_score_staking::AccountIdConvert<Runtime> for IdentityAccountIdConvert {
	fn convert(account: AccountId) -> <Runtime as frame_system::Config>::AccountId {
		account
	}
}

impl pallet_score_staking::Config for Runtime {
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type AccountIdConvert = IdentityAccountIdConvert;
	type AdminOrigin = EnsureRootOrHalfCouncil;
	type YearlyIssuance = ConstU128<{ 100_000_000 * UNIT }>;
	type YearlyInflation = DefaultYearlyInflation;
	type MaxScoreUserCount = ConstU32<1_000_000>;
}

impl runtime_common::BaseRuntimeRequirements for Runtime {}
impl runtime_common::ParaRuntimeRequirements for Runtime {}

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<
		cumulus_primitives_core::AggregateMessageOrigin,
	>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
		RuntimeCall,
	>;
	type Size = u32;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = sp_core::ConstU32<{ 64 * 1024 }>;
	type MaxStale = sp_core::ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = ();
}

construct_runtime! {
	pub enum Runtime
	{
		// Core
		System: frame_system = 0,
		Timestamp: pallet_timestamp = 1,
		Scheduler: pallet_scheduler = 2,
		Utility: pallet_utility = 3,
		Multisig: pallet_multisig = 4,
		Proxy: pallet_proxy = 5,
		Preimage: pallet_preimage = 6,
		Indices: pallet_indices = 7,

		// Token related
		Balances: pallet_balances = 10,
		Vesting: pallet_vesting = 11,
		TransactionPayment: pallet_transaction_payment = 12,
		Treasury: pallet_treasury = 13,

		// Governance
		Democracy: pallet_democracy = 21,
		Council: pallet_collective::<Instance1> = 22,
		CouncilMembership: pallet_membership::<Instance1> = 23,
		TechnicalCommittee: pallet_collective::<Instance2> = 24,
		TechnicalCommitteeMembership: pallet_membership::<Instance2> = 25,
		Bounties: pallet_bounties = 26,
		Tips: pallet_tips = 27,
		ParachainIdentity: pallet_identity = 28,

		// Parachain
		ParachainSystem: cumulus_pallet_parachain_system = 30,
		ParachainInfo: parachain_info = 31,

		AssetRate: pallet_asset_rate = 39,

		// Collator support
		// About the order of these 5 pallets, the comment in cumulus seems to be outdated.
		//
		// The main thing is Authorship looks for the block author (T::FindAuthor::find_author)
		// in its `on_initialize` hook -> Session::find_author, where Session::validators() is enquired.
		// Meanwhile Session could modify the validators storage in its `on_initialize` hook. If Session
		// comes after Authorship, the changes on validators() will only take effect in the next block.
		//
		// I assume it's the desired behavior though or it doesn't really matter.
		//
		// also see the comment above `AllPalletsWithSystem` and
		// https://github.com/litentry/litentry-parachain/issues/336
		Authorship: pallet_authorship = 40,
		//41 is for old CollatorSelection, replaced by ParachainSTaking
		Session: pallet_session = 42,
		Aura: pallet_aura = 43,
		AuraExt: cumulus_pallet_aura_ext = 44,
		ParachainStaking: pallet_parachain_staking = 45,

		// XCM helpers
		XcmpQueue: cumulus_pallet_xcmp_queue = 50,
		PolkadotXcm: pallet_xcm = 51,
		CumulusXcm: cumulus_pallet_xcm = 52,
		DmpQueue: cumulus_pallet_dmp_queue = 53,
		// XTokens: orml_xtokens = 54,
		// 55 is saved for old pallet: Tokens: orml_tokens
		Assets: pallet_assets = 56,
		MessageQueue: pallet_message_queue = 57,

		// Rococo pallets
		ChainBridge: pallet_chain_bridge= 60,
		BridgeTransfer: pallet_bridge_transfer = 61,
		ExtrinsicFilter: pallet_extrinsic_filter = 63,
		IdentityManagement: pallet_identity_management = 64,
		AssetManager: pallet_asset_manager = 65,
		VCManagement: pallet_vc_management = 66,
		IMPExtrinsicWhitelist: pallet_group::<Instance1> = 67,
		VCMPExtrinsicWhitelist: pallet_group::<Instance2> = 68,
		Bitacross: pallet_bitacross = 70,
		EvmAssertions: pallet_evm_assertions = 71,

		// Developer council
		DeveloperCommittee: pallet_collective::<Instance3> = 73,
		DeveloperCommitteeMembership: pallet_membership::<Instance3> = 74,
		ScoreStaking: pallet_score_staking = 75,

		// New Bridge Added
		AssetsHandler: pallet_assets_handler = 76,

		OmniAccount: pallet_omni_account = 84,

		// TEE
		Teebag: pallet_teebag = 93,

		// Frontier
		EVM: pallet_evm = 120,
		Ethereum: pallet_ethereum = 121,

		// TMP
		AccountFix: pallet_account_fix = 254,
		Sudo: pallet_sudo = 255,
	}
}

pub struct BaseCallFilter;
impl Contains<RuntimeCall> for BaseCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		if matches!(
			call,
			RuntimeCall::Sudo(_)
				| RuntimeCall::System(_)
				| RuntimeCall::Timestamp(_)
				| RuntimeCall::ParachainSystem(_)
				| RuntimeCall::ExtrinsicFilter(_)
				| RuntimeCall::Multisig(_)
				| RuntimeCall::Council(_)
				| RuntimeCall::CouncilMembership(_)
				| RuntimeCall::TechnicalCommittee(_)
				| RuntimeCall::TechnicalCommitteeMembership(_)
				| RuntimeCall::Utility(_)
		) {
			// always allow core calls
			return true;
		}

		pallet_extrinsic_filter::Pallet::<Runtime>::contains(call)
	}
}

pub struct SafeModeFilter;
impl Contains<RuntimeCall> for SafeModeFilter {
	fn contains(_call: &RuntimeCall) -> bool {
		false
	}
}

pub struct NormalModeFilter;
impl Contains<RuntimeCall> for NormalModeFilter {
	fn contains(call: &RuntimeCall) -> bool {
		matches!(
			call,
			// Vesting::vest
			RuntimeCall::Vesting(pallet_vesting::Call::vest { .. }) |
			// ChainBridge
			RuntimeCall::ChainBridge(_) |
			// Bounties
			RuntimeCall::Bounties(_) |
			// BridgeTransfer
			RuntimeCall::BridgeTransfer(_) |
			// XTokens::transfer for normal users
			// RuntimeCall::XTokens(orml_xtokens::Call::transfer { .. }) |
			// collective
			RuntimeCall::DeveloperCommittee(_) |
			// memberships
			RuntimeCall::CouncilMembership(_) |
			RuntimeCall::TechnicalCommitteeMembership(_) |
			RuntimeCall::DeveloperCommitteeMembership(_) |
			// democracy, we don't subdivide the calls, so we allow public proposals
			RuntimeCall::Democracy(_) |
			// Preimage
			RuntimeCall::Preimage(_) |
			// Identity
			RuntimeCall::ParachainIdentity(_) |
			// Utility
			RuntimeCall::Utility(_) |
			// Seesion
			RuntimeCall::Session(_) |
			// Balance
			RuntimeCall::Balances(_) |
			// IMP and VCMP
			RuntimeCall::IdentityManagement(_) |
			RuntimeCall::VCManagement(_) |
			// TEE pallets
			RuntimeCall::Teebag(_) |
			// ParachainStaking; Only the collator part
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::join_candidates { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::schedule_leave_candidates { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::execute_leave_candidates { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::cancel_leave_candidates { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::go_offline { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::go_online { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::candidate_bond_more { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::schedule_candidate_bond_less { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::execute_candidate_bond_less { .. }) |
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::cancel_candidate_bond_less { .. }) |
			// Group
			RuntimeCall::IMPExtrinsicWhitelist(_) |
			RuntimeCall::VCMPExtrinsicWhitelist(_) |
			// EVM
			// Substrate EVM extrinsic not allowed
			// So no EVM pallet
			RuntimeCall::Ethereum(_) |
			// AccountFix
			RuntimeCall::AccountFix(_) |
			RuntimeCall::AssetsHandler(_) |
			RuntimeCall::Bitacross(_) |
			RuntimeCall::EvmAssertions(_) |
			RuntimeCall::ScoreStaking(_) |
			RuntimeCall::OmniAccount(_)
		)
	}
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_asset_manager, AssetManager]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[pallet_utility, Utility]
		[pallet_treasury, Treasury]
		[pallet_democracy, Democracy]
		[pallet_collective, Council]
		[pallet_proxy, Proxy]
		[pallet_membership, CouncilMembership]
		[pallet_multisig, Multisig]
		[paleet_evm, EVM]
		[pallet_extrinsic_filter, ExtrinsicFilter]
		[pallet_scheduler, Scheduler]
		[pallet_preimage, Preimage]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_parachain_staking, ParachainStaking]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[pallet_identity_management, IdentityManagement]
		[pallet_vc_management, VCManagement]
		[pallet_chain_bridge,ChainBridge]
		[pallet_bridge_transfer,BridgeTransfer]
		[pallet_teebag, Teebag]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}

		fn account_basic(address: H160) -> pallet_evm::Account {
			let (account, _) = EVM::account_basic(&address);
			account
		}

		fn gas_price() -> U256 {
			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
			gas_price
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			pallet_evm::AccountCodes::<Runtime>::get(address)
		}

		fn author() -> H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;

			// Reused approach from Moonbeam since Frontier implementation doesn't support this
			let mut estimated_transaction_len = data.len() +
				// to: 20
				// from: 20
				// value: 32
				// gas_limit: 32
				// nonce: 32
				// 1 byte transaction action variant
				// chain id 8 bytes
				// 65 bytes signature
				210;
			if max_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if max_priority_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}

			let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
			let without_base_extrinsic_weight = true;

			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};

			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				Vec::new(),
				is_transactional,
				validate,
				weight_limit,
				proof_size_base_cost,
				config
					.as_ref()
					.unwrap_or_else(|| <Runtime as pallet_evm::Config>::config()),
			)
			.map_err(|err| err.error.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;

			// Reused approach from Moonbeam since Frontier implementation doesn't support this
			let mut estimated_transaction_len = data.len() +
				// to: 20
				// from: 20
				// value: 32
				// gas_limit: 32
				// nonce: 32
				// 1 byte transaction action variant
				// chain id 8 bytes
				// 65 bytes signature
				210;
			if max_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if max_priority_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}

			let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
			let without_base_extrinsic_weight = true;

			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};

			#[allow(clippy::or_fun_call)] // suggestion not helpful here
			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				Vec::new(),
				is_transactional,
				validate,
				weight_limit,
				proof_size_base_cost,
				config
					.as_ref()
					.unwrap_or(<Runtime as pallet_evm::Config>::config()),
				)
				.map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			pallet_ethereum::CurrentBlock::<Runtime>::get()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			pallet_ethereum::CurrentReceipts::<Runtime>::get()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentReceipts::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn extrinsic_filter(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> Vec<pallet_ethereum::Transaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
				_ => None
			}).collect::<Vec<pallet_ethereum::Transaction>>()
		}

		fn elasticity() -> Option<Permill> {
			None
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> (Option<pallet_ethereum::Block>, Option<Vec<fp_rpc::TransactionStatus>>) {
			for ext in xts.into_iter() {
				let _ = Executive::apply_extrinsic(ext);
			}

			Ethereum::on_finalize(System::block_number() + 1);

			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}
	}

	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(transaction: pallet_ethereum::Transaction) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)
		}
	}

	impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
		fn trace_transaction(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			traced_transaction: &pallet_ethereum::Transaction,
			header: &<Block as BlockT>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			use moonbeam_evm_tracer::tracer::EvmTracer;

			// Apply the a subset of extrinsics: all the substrate-specific or ethereum
			// transactions that preceded the requested transaction.
			for ext in extrinsics.into_iter() {
				let _ = match &ext.0.function {
					RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => {
						if transaction == traced_transaction {
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
							return Ok(());
						} else {
							Executive::apply_extrinsic(ext)
						}
					}
					_ => Executive::apply_extrinsic(ext),
				};
			}
			Err(sp_runtime::DispatchError::Other(
				"Failed to find Ethereum transaction among the extrinsics.",
			))
		}

		fn trace_block(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			known_transactions: Vec<H256>,
			header: &<Block as BlockT>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			use moonbeam_evm_tracer::tracer::EvmTracer;

			let mut config = <Runtime as pallet_evm::Config>::config().clone();
			config.estimate = true;

			// Apply all extrinsics. Ethereum extrinsics are traced.
			for ext in extrinsics.into_iter() {
				match &ext.0.function {
					RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => {
						if known_transactions.contains(&transaction.hash()) {
							// Each known extrinsic is a new call stack.
							EvmTracer::emit_new();
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
						} else {
							let _ = Executive::apply_extrinsic(ext);
						}
					}
					_ => {
						let _ = Executive::apply_extrinsic(ext);
					}
				};
			}

			Ok(())
		}
		
		fn trace_call(
			header: &<Block as BlockT>::Header,
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<(), sp_runtime::DispatchError> {
			#[cfg(feature = "evm-tracing")]
			{
				use moonbeam_evm_tracer::tracer::EvmTracer;

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem.
				Executive::initialize_block(header);

				EvmTracer::new().trace(|| {
					let is_transactional = false;
					let validate = true;
					let without_base_extrinsic_weight = true;


					// Estimated encoded transaction size must be based on the heaviest transaction
					// type (EIP1559Transaction) to be compatible with all transaction types.
					let mut estimated_transaction_len = data.len() +
					// pallet ethereum index: 1
					// transact call index: 1
					// Transaction enum variant: 1
					// chain_id 8 bytes
					// nonce: 32
					// max_priority_fee_per_gas: 32
					// max_fee_per_gas: 32
					// gas_limit: 32
					// action: 21 (enum varianrt + call address)
					// value: 32
					// access_list: 1 (empty vec size)
					// 65 bytes signature
					258;

					if access_list.is_some() {
						estimated_transaction_len += access_list.encoded_size();
					}

					let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();

					let (weight_limit, proof_size_base_cost) =
						match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
							gas_limit,
							without_base_extrinsic_weight
						) {
							weight_limit if weight_limit.proof_size() > 0 => {
								(Some(weight_limit), Some(estimated_transaction_len as u64))
							}
							_ => (None, None),
						};

					let _ = <Runtime as pallet_evm::Config>::Runner::call(
						from,
						to,
						data,
						value,
						gas_limit,
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						validate,
						weight_limit,
						proof_size_base_cost,
						<Runtime as pallet_evm::Config>::config(),
					);
				});
				Ok(())
			}
			#[cfg(not(feature = "evm-tracing"))]
			Err(sp_runtime::DispatchError::Other(
				"Missing `evm-tracing` compile time feature flag.",
			))
		}
	}

	impl moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block> for Runtime {
		fn extrinsic_filter(
			xts_ready: Vec<<Block as BlockT>::Extrinsic>,
			xts_future: Vec<<Block as BlockT>::Extrinsic>,
		) -> moonbeam_rpc_primitives_txpool::TxPoolResponse {
			moonbeam_rpc_primitives_txpool::TxPoolResponse {
				ready: xts_ready
					.into_iter()
					.filter_map(|xt| match xt.0.function {
						RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => Some(transaction),
						_ => None,
					})
					.collect(),
				future: xts_future
					.into_iter()
					.filter_map(|xt| match xt.0.function {
						RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => Some(transaction),
						_ => None,
					})
					.collect(),
			}
		}
	}

	impl pallet_omni_account_runtime_api::OmniAccountApi<Block, AccountId> for Runtime {
		fn omni_account(identity: Identity) -> AccountId {
			OmniAccount::omni_account(identity)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			log::info!("try-runtime::on_runtime_upgrade paseo.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(block: Block, state_root_check: bool,signature_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			log::info!(
				target: "runtime::Paseo", "try-runtime: executing block #{} ({:?}) / root checks: {:?} / sanity-checks: {:?}",
				block.header.number,
				block.header.hash(),
				state_root_check,
				select,
			);
			Executive::try_execute_block(block, state_root_check, signature_check,select).expect("try_execute_block failed")
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, BenchmarkError};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

#[allow(dead_code)]
struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
