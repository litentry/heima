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

// TODO: the sidechain block number type is chaotic from upstream
use crate::{
	Assertion, Identity, IdentityNetworkTuple, ParentchainBlockNumber, RequestAesKey,
	ShardIdentifier, Web2ValidationData, Web3Network,
};
use codec::{Decode, Encode};
use itp_sgx_runtime_primitives::types::{AccountId, BlockNumber as SidechainBlockNumber};
use sp_core::H256;
use sp_runtime::traits::ConstU32;
use sp_std::prelude::Vec;

/// Here a few Request structs are defined for asynchronously stf-tasks handling.
/// A `callback` exists for some request types to submit a callback TrustedCall to top pool.
/// We use the encoded version just to avoid cyclic dependency, otherwise we have
/// ita-stf -> lc-stf-task-sender -> ita-stf
///
/// In this way we make sure the state is processed "chronologically" by the StfExecutor.
/// We can't write any state in this state, otherwise we can be trapped into a situation
/// where the state doesn't match the apriori state that is recorded before executing any
/// trusted calls in block production (InvalidAprioriHash error).
///
/// Reading state is not a problem. However, we prefer to read the required storage before
/// sending the stf-task and pass it as parameters in `Request`, e.g. `challenge_code` below.
/// The reason is we actually want the "snapshot" state when the preflight TrustedCall gets
/// executed instead of the "live" state.
///
/// The callback TrustedCall will be appended to the end of top pool but we don't see a
/// problem. In case some preflight TrustedCall and callback TrustedCall are going to change
/// the same storage, we should implement them carefully and always treat it as if both
/// TrustedCalls can get executed in any order.
///
/// For more information, please see:
/// https://github.com/litentry/tee-worker/issues/110
/// https://www.notion.so/web3builders/Sidechain-block-importer-and-block-production-28292233b4c74f4ab8110a0014f8d9df

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Web2IdentityVerificationRequest {
	pub shard: ShardIdentifier,
	pub who: Identity,
	pub identity: Identity,
	pub raw_msg: Vec<u8>,
	pub validation_data: Web2ValidationData,
	pub web3networks: Vec<Web3Network>,
	pub top_hash: H256,
	pub maybe_key: Option<RequestAesKey>,
	pub req_ext_hash: H256,
}

pub type MaxIdentityLength = ConstU32<64>;
/// TODO: adapt struct fields later
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AssertionBuildRequest {
	pub shard: ShardIdentifier,
	pub signer: AccountId,
	pub who: Identity,
	pub assertion: Assertion,
	pub identities: Vec<IdentityNetworkTuple>,
	pub top_hash: H256,
	pub parachain_block_number: ParentchainBlockNumber,
	pub sidechain_block_number: SidechainBlockNumber,
	pub parachain_runtime_version: u32,
	pub sidechain_runtime_version: u32,
	pub maybe_key: Option<RequestAesKey>,
	pub should_create_id_graph: bool,
	pub req_ext_hash: H256,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum RequestType {
	#[codec(index = 0)]
	IdentityVerification(Web2IdentityVerificationRequest),
}

impl From<Web2IdentityVerificationRequest> for RequestType {
	fn from(r: Web2IdentityVerificationRequest) -> Self {
		RequestType::IdentityVerification(r)
	}
}
