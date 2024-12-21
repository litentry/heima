/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

pub mod events;

use crate::{parentchain::events::AssertionCreated, OpaqueCall, ShardIdentifier};
use alloc::vec::Vec;
use codec::{Decode, Encode};
use core::fmt::Debug;
use events::*;
use itp_stf_primitives::traits::{IndirectExecutor, TrustedCallVerification};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{bounded::alloc, H160, H256};
use sp_runtime::{
	generic::Header as HeaderG,
	traits::{BlakeTwo256, Block as ParentchainBlock, Header as ParentchainHeader},
	MultiAddress, MultiSignature,
};

use self::events::ParentchainBlockProcessed;

pub type StorageProof = Vec<Vec<u8>>;

// Basic Types.
pub type Index = u32;
pub type Balance = u128;
pub type Hash = sp_core::H256;

// Account Types.
pub type AccountId = sp_core::crypto::AccountId32;
pub type AccountData = pallet_balances::AccountData<Balance>;
pub type AccountInfo = frame_system::AccountInfo<Index, AccountData>;
pub type Address = MultiAddress<AccountId, ()>;

// todo! make generic
/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
}

// Block Types
pub type BlockNumber = u32;
pub type Header = HeaderG<BlockNumber, BlakeTwo256>;
pub type BlockHash = sp_core::H256;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ParentchainId {
	/// The Litentry Parentchain, the trust root of the enclave and serving finality to sidechains.
	#[codec(index = 0)]
	Litentry,
	/// A target chain containing custom business logic.
	#[codec(index = 1)]
	TargetA,
	/// Another target chain containing custom business logic.
	#[codec(index = 2)]
	TargetB,
}

#[cfg(feature = "std")]
impl std::fmt::Display for ParentchainId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let message = match self {
			ParentchainId::Litentry => "Litentry",
			ParentchainId::TargetA => "TargetA",
			ParentchainId::TargetB => "TargetB",
		};
		write!(f, "{}", message)
	}
}

pub trait IdentifyParentchain {
	fn parentchain_id(&self) -> ParentchainId;
}

pub trait FilterEvents {
	type Error: From<ParentchainEventProcessingError> + Debug;

	fn get_link_identity_events(&mut self) -> Result<Vec<LinkIdentityRequested>, Self::Error>;

	fn get_vc_requested_events(&mut self) -> Result<Vec<VCRequested>, Self::Error>;

	fn get_deactivate_identity_events(
		&mut self,
	) -> Result<Vec<DeactivateIdentityRequested>, Self::Error>;

	fn get_activate_identity_events(
		&mut self,
	) -> Result<Vec<ActivateIdentityRequested>, Self::Error>;

	fn get_enclave_unauthorized_events(&mut self) -> Result<Vec<EnclaveUnauthorized>, Self::Error>;

	fn get_opaque_task_posted_events(&mut self) -> Result<Vec<OpaqueTaskPosted>, Self::Error>;

	fn get_assertion_created_events(&mut self) -> Result<Vec<AssertionCreated>, Self::Error>;

	fn get_parentchain_block_proccessed_events(
		&mut self,
	) -> Result<Vec<ParentchainBlockProcessed>, Self::Error>;

	fn get_enclave_added_events(&mut self) -> Result<Vec<EnclaveAdded>, Self::Error>;

	fn get_enclave_removed_events(&mut self) -> Result<Vec<EnclaveRemoved>, Self::Error>;

	fn get_account_store_updated_events(&mut self)
		-> Result<Vec<AccountStoreUpdated>, Self::Error>;
}

#[derive(Debug)]
pub enum ExtrinsicStatus {
	Success,
	Failed,
}

pub type ProcessedEventsArtifacts = (Vec<H256>, Vec<H160>, Vec<H160>);

pub trait HandleParentchainEvents<Executor, TCS, Error, RRU, SRU, ERU>
where
	Executor: IndirectExecutor<TCS, Error, RRU, SRU, ERU>,
	TCS: PartialEq + Encode + Decode + Debug + Clone + Send + Sync + TrustedCallVerification,
{
	type Output;

	fn handle_events<Block>(
		&self,
		executor: &Executor,
		events: &mut impl FilterEvents,
		block_number: <<Block as ParentchainBlock>::Header as ParentchainHeader>::Number,
	) -> Result<Self::Output, Error>
	where
		Block: ParentchainBlock;
}

#[derive(Debug)]
pub enum ParentchainEventProcessingError {
	FunctionalityDisabled,
	LinkIdentityFailure,
	DeactivateIdentityFailure,
	ActivateIdentityFailure,
	VCRequestedFailure,
	EnclaveUnauthorizedFailure,
	OpaqueTaskPostedFailure,
	AssertionCreatedFailure,
	ParentchainBlockProcessedFailure,
	RelayerAddFailure,
	RelayerRemoveFailure,
	EnclaveAddFailure,
	EnclaveRemoveFailure,
	BtcWalletGeneratedFailure,
	AccountStoreUpdatedFailure,
}

impl core::fmt::Display for ParentchainEventProcessingError {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = match &self {
			ParentchainEventProcessingError::FunctionalityDisabled =>
				"Parentchain Event Processing Error: FunctionalityDisabled",
			ParentchainEventProcessingError::LinkIdentityFailure =>
				"Parentchain Event Processing Error: LinkIdentityFailure",
			ParentchainEventProcessingError::DeactivateIdentityFailure =>
				"Parentchain Event Processing Error: DeactivateIdentityFailure",
			ParentchainEventProcessingError::ActivateIdentityFailure =>
				"Parentchain Event Processing Error: ActivateIdentityFailure",
			ParentchainEventProcessingError::VCRequestedFailure =>
				"Parentchain Event Processing Error: VCRequestedFailure",
			ParentchainEventProcessingError::EnclaveUnauthorizedFailure =>
				"Parentchain Event Processing Error: EnclaveUnauthorizedFailure",
			ParentchainEventProcessingError::OpaqueTaskPostedFailure =>
				"Parentchain Event Processing Error: OpaqueTaskPostedFailure",
			ParentchainEventProcessingError::AssertionCreatedFailure =>
				"Parentchain Event Processing Error: AssertionCreatedFailure",
			ParentchainEventProcessingError::ParentchainBlockProcessedFailure =>
				"Parentchain Event Processing Error: ParentchainBlockProcessedFailure",
			ParentchainEventProcessingError::RelayerAddFailure =>
				"Parentchain Event Processing Error: RelayerAddFailure",
			ParentchainEventProcessingError::RelayerRemoveFailure =>
				"Parentchain Event Processing Error: RelayerRemoveFailure",
			ParentchainEventProcessingError::EnclaveAddFailure =>
				"Parentchain Event Processing Error: EnclaveAddFailure",
			ParentchainEventProcessingError::EnclaveRemoveFailure =>
				"Parentchain Event Processing Error: EnclaveRemoveFailure",
			ParentchainEventProcessingError::BtcWalletGeneratedFailure =>
				"Parentchain Event Processing Error: BtcWalletGeneratedFailure",
			ParentchainEventProcessingError::AccountStoreUpdatedFailure =>
				"Parentchain Event Processing Error: AccountStoreUpdatedFailure",
		};
		write!(f, "{}", message)
	}
}

impl From<ParentchainEventProcessingError> for () {
	fn from(_: ParentchainEventProcessingError) -> Self {}
}

/// a wrapper to target calls to specific parentchains
#[derive(Encode, Debug, Clone, PartialEq, Eq)]
pub enum ParentchainCall {
	Litentry(OpaqueCall),
	TargetA(OpaqueCall),
	TargetB(OpaqueCall),
}

impl ParentchainCall {
	pub fn as_litentry(&self) -> Option<OpaqueCall> {
		if let Self::Litentry(call) = self {
			Some(call.clone())
		} else {
			None
		}
	}
	pub fn as_target_a(&self) -> Option<OpaqueCall> {
		if let Self::TargetA(call) = self {
			Some(call.clone())
		} else {
			None
		}
	}
	pub fn as_target_b(&self) -> Option<OpaqueCall> {
		if let Self::TargetB(call) = self {
			Some(call.clone())
		} else {
			None
		}
	}
	pub fn as_opaque_call_for(&self, parentchain_id: ParentchainId) -> Option<OpaqueCall> {
		match parentchain_id {
			ParentchainId::Litentry =>
				if let Self::Litentry(call) = self {
					Some(call.clone())
				} else {
					None
				},
			ParentchainId::TargetA =>
				if let Self::TargetA(call) = self {
					Some(call.clone())
				} else {
					None
				},
			ParentchainId::TargetB =>
				if let Self::TargetB(call) = self {
					Some(call.clone())
				} else {
					None
				},
		}
	}
}

// Moved from `itc_light_client::light_client_init_params` to de-couple deps
use sp_consensus_grandpa::AuthorityList;

#[derive(Encode, Decode, Clone)]
pub struct GrandpaParams<Header> {
	pub genesis_header: Header,
	pub authorities: AuthorityList,
	pub authority_proof: Vec<Vec<u8>>,
}

impl<Header> GrandpaParams<Header> {
	pub fn new(
		genesis_header: Header,
		authorities: AuthorityList,
		authority_proof: Vec<Vec<u8>>,
	) -> Self {
		Self { genesis_header, authorities, authority_proof }
	}
}

#[derive(Encode, Decode, Clone)]
pub struct SimpleParams<Header> {
	pub genesis_header: Header,
}

impl<Header> SimpleParams<Header> {
	pub fn new(genesis_header: Header) -> Self {
		Self { genesis_header }
	}
}

// Moved from `itc_parent::primitives`
use sp_runtime::traits::Block;

pub type ParachainBlock = crate::Block;
pub type SolochainBlock = crate::Block;
pub type HeaderFor<B> = <B as Block>::Header;
pub type SolochainHeader = HeaderFor<SolochainBlock>;
pub type ParachainHeader = HeaderFor<ParachainBlock>;
pub type SolochainParams = GrandpaParams<SolochainHeader>;
pub type ParachainParams = SimpleParams<ParachainHeader>;

/// Initialization primitives, used by both service and enclave.
/// Allows to use a single E-call for the initialization of different parentchain types.
#[derive(Encode, Decode, Clone)]
pub enum ParentchainInitParams {
	Solochain { id: ParentchainId, shard: ShardIdentifier, params: SolochainParams },
	Parachain { id: ParentchainId, shard: ShardIdentifier, params: ParachainParams },
}

impl ParentchainInitParams {
	pub fn id(&self) -> &ParentchainId {
		match self {
			Self::Solochain { id, .. } => id,
			Self::Parachain { id, .. } => id,
		}
	}
	pub fn is_solochain(&self) -> bool {
		matches!(self, Self::Solochain { .. })
	}
	pub fn is_parachain(&self) -> bool {
		matches!(self, Self::Parachain { .. })
	}
}

impl From<(ParentchainId, ShardIdentifier, SolochainParams)> for ParentchainInitParams {
	fn from(value: (ParentchainId, ShardIdentifier, SolochainParams)) -> Self {
		Self::Solochain { id: value.0, shard: value.1, params: value.2 }
	}
}

impl From<(ParentchainId, ShardIdentifier, ParachainParams)> for ParentchainInitParams {
	fn from(value: (ParentchainId, ShardIdentifier, ParachainParams)) -> Self {
		Self::Parachain { id: value.0, shard: value.1, params: value.2 }
	}
}
