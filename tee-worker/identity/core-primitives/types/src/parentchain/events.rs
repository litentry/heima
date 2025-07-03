use crate::{
	AccountId, Assertion, Balance, BlockNumber, Hash, MrEnclave, RsaRequest, ShardIdentifier,
	WorkerType,
};
use alloc::{format, vec::Vec};
use codec::{Decode, Encode};
use core::fmt::Debug;
use itp_utils::{hex::ToHexPrefixed, stringify::account_id_to_string};
use litentry_primitives::{Address32, MemberAccount};
use sp_core::H160;
use substrate_api_client::ac_node_api::StaticEvent;

// System pallet events
#[derive(Encode, Decode, Debug)]
pub struct ExtrinsicSuccess;

impl StaticEvent for ExtrinsicSuccess {
	const PALLET: &'static str = "System";
	const EVENT: &'static str = "ExtrinsicSuccess";
}

#[derive(Encode, Decode)]
pub struct ExtrinsicFailed;

impl StaticEvent for ExtrinsicFailed {
	const PALLET: &'static str = "System";
	const EVENT: &'static str = "ExtrinsicFailed";
}

#[derive(Encode, Decode, Debug)]
pub struct BalanceTransfer {
	pub from: AccountId,
	pub to: AccountId,
	pub amount: Balance,
}

// Balances pallet events

impl core::fmt::Display for BalanceTransfer {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"BalanceTransfer :: from: {}, to: {}, amount: {}",
			account_id_to_string::<AccountId>(&self.from),
			account_id_to_string::<AccountId>(&self.to),
			self.amount
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for BalanceTransfer {
	const PALLET: &'static str = "Balances";
	const EVENT: &'static str = "Transfer";
}

// Teebag pallet events

#[derive(Encode, Decode, Debug)]
pub struct ParentchainBlockProcessed {
	pub shard: ShardIdentifier,
	pub block_number: BlockNumber,
	pub block_hash: Hash,
	pub task_merkle_root: Hash,
}

impl core::fmt::Display for ParentchainBlockProcessed {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"ParentchainBlockProcessed :: nr {} shard: {}, merkle: {:?}, block hash {:?}",
			self.block_number, self.shard, self.task_merkle_root, self.block_hash
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for ParentchainBlockProcessed {
	const PALLET: &'static str = "Teebag";
	const EVENT: &'static str = "ParentchainBlockProcessed";
}

#[derive(Encode, Decode, Debug)]
pub struct EnclaveUnauthorized {
	pub worker_type: WorkerType,
	pub mrenclave: MrEnclave,
}

impl core::fmt::Display for EnclaveUnauthorized {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"EnclaveUnauthorized :: worker_type: {:?}, mrenclave: {}",
			self.worker_type,
			self.mrenclave.to_hex()
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for EnclaveUnauthorized {
	const PALLET: &'static str = "Teebag";
	const EVENT: &'static str = "EnclaveUnauthorized";
}

#[derive(Encode, Decode, Debug)]
pub struct EnclaveAdded {
	pub who: Address32,
	pub worker_type: WorkerType,
	pub url: Vec<u8>,
}

impl core::fmt::Display for EnclaveAdded {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"EnclaveAdded :: who: {:?}, worker_type: {:?}, url: {:?}",
			self.who, self.worker_type, self.url
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for EnclaveAdded {
	const PALLET: &'static str = "Teebag";
	const EVENT: &'static str = "EnclaveAdded";
}

#[derive(Encode, Decode, Debug)]
pub struct EnclaveRemoved {
	pub who: Address32,
}

impl core::fmt::Display for EnclaveRemoved {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!("EnclaveRemoved :: who: {:?}", self.who);
		write!(f, "{}", message)
	}
}

impl StaticEvent for EnclaveRemoved {
	const PALLET: &'static str = "Teebag";
	const EVENT: &'static str = "EnclaveRemoved";
}

// omni-account pallet events
#[derive(Encode, Decode, Debug)]
pub struct AccountStoreUpdated {
	pub who: AccountId,
	pub account_store: Vec<MemberAccount>,
}

impl core::fmt::Display for AccountStoreUpdated {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"AccountStoreCreated :: who: {}, account_store: {:?}",
			account_id_to_string::<AccountId>(&self.who),
			self.account_store
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for AccountStoreUpdated {
	const PALLET: &'static str = "OmniAccount";
	const EVENT: &'static str = "AccountStoreUpdated";
}

// Identity-worker events

#[derive(Encode, Decode, Debug)]
pub struct LinkIdentityRequested {
	pub shard: ShardIdentifier,
	pub account: AccountId,
	pub encrypted_identity: Vec<u8>,
	pub encrypted_validation_data: Vec<u8>,
	pub encrypted_web3networks: Vec<u8>,
}

impl core::fmt::Display for LinkIdentityRequested {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"LinkIdentityRequested :: shard: {}, account: {}, identity: {:?}, validation_data: {:?}, web3networks: {:?}",
			self.shard,
			account_id_to_string::<AccountId>(&self.account),
			self.encrypted_identity,
			self.encrypted_validation_data,
			self.encrypted_web3networks
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for LinkIdentityRequested {
	const PALLET: &'static str = "IdentityManagement";
	const EVENT: &'static str = "LinkIdentityRequested";
}

#[derive(Encode, Decode, Debug)]
pub struct DeactivateIdentityRequested {
	pub shard: ShardIdentifier,
	pub account: AccountId,
	pub encrypted_identity: Vec<u8>,
}

impl core::fmt::Display for DeactivateIdentityRequested {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"DeactivateIdentityRequested :: shard: {}, account: {}, identity: {:?}",
			self.shard,
			account_id_to_string::<AccountId>(&self.account),
			self.encrypted_identity
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for DeactivateIdentityRequested {
	const PALLET: &'static str = "IdentityManagement";
	const EVENT: &'static str = "DeactivateIdentityRequested";
}

#[derive(Encode, Decode, Debug)]
pub struct ActivateIdentityRequested {
	pub shard: ShardIdentifier,
	pub account: AccountId,
	pub encrypted_identity: Vec<u8>,
}

impl core::fmt::Display for ActivateIdentityRequested {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"ActivateIdentityRequested :: shard: {}, account: {}, identity: {:?}",
			self.shard,
			account_id_to_string::<AccountId>(&self.account),
			self.encrypted_identity
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for ActivateIdentityRequested {
	const PALLET: &'static str = "IdentityManagement";
	const EVENT: &'static str = "ActivateIdentityRequested";
}

// VCManagement events

#[derive(Encode, Decode, Debug)]
pub struct VCRequested {
	pub shard: ShardIdentifier,
	pub account: AccountId,
	pub assertion: Assertion,
}

impl core::fmt::Display for VCRequested {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"VCRequested :: shard: {}, account: {}, assertion: {:?}",
			self.shard,
			account_id_to_string::<AccountId>(&self.account),
			self.assertion
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for VCRequested {
	const PALLET: &'static str = "VCManagement";
	const EVENT: &'static str = "VCRequested";
}

#[derive(Encode, Decode, Debug)]
pub struct OpaqueTaskPosted {
	pub request: RsaRequest,
}

impl core::fmt::Display for OpaqueTaskPosted {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!("OpaqueTaskPosted :: request: {:?}", self.request);
		write!(f, "{}", message)
	}
}

impl StaticEvent for OpaqueTaskPosted {
	const PALLET: &'static str = "Teebag";
	const EVENT: &'static str = "OpaqueTaskPosted";
}

#[derive(Encode, Decode, Debug)]
pub struct AssertionCreated {
	pub id: H160,
	pub byte_code: Vec<u8>,
	pub secrets: Vec<Vec<u8>>,
}

impl core::fmt::Display for AssertionCreated {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		let message = format!(
			"{:?} :: byte_code: {:?}, secrets: {:?}",
			AssertionCreated::EVENT,
			self.byte_code,
			self.secrets
		);
		write!(f, "{}", message)
	}
}

impl StaticEvent for AssertionCreated {
	const PALLET: &'static str = "EvmAssertions";
	const EVENT: &'static str = "AssertionCreated";
}
