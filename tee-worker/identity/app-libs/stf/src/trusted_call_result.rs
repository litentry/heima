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

// This file contain the RPC response struct which will be encoded and
// passed back to the requester of trustedCall direct invocation (DI).
// They are mostly translated from the callback extrinsics in IMP.

use crate::{Box, String};
use codec::{Decode, Encode};
use itp_stf_interface::StfExecutionResult;
use itp_types::{AccountId, H256};
use litentry_primitives::{AesOutput, VCMPError};
use std::vec::Vec;

#[derive(Encode, Decode, Debug)]
pub enum TrustedCallResult {
	#[codec(index = 0)]
	Empty,
	#[codec(index = 1)]
	Streamed,
	#[codec(index = 2)]
	LinkIdentity(LinkIdentityResult),
	#[codec(index = 3)]
	RequestVC(RequestVCResult),
	#[codec(index = 4)]
	DeactivateIdentity(DeactivateIdentityResult),
	#[codec(index = 5)]
	ActivateIdentity(ActivateIdentityResult),
	#[codec(index = 6)]
	SetIdentityNetworks(SetIdentityNetworksResult),
}

impl StfExecutionResult for TrustedCallResult {
	fn get_encoded_result(self) -> Vec<u8> {
		match self {
			Self::Empty => Vec::default(),
			Self::Streamed => Vec::default(),
			Self::LinkIdentity(result) => result.encode(),
			Self::RequestVC(result) => result.encode(),
			Self::DeactivateIdentity(result) => result.encode(),
			Self::ActivateIdentity(result) => result.encode(),
			Self::SetIdentityNetworks(result) => result.encode(),
		}
	}

	fn force_connection_wait(&self) -> bool {
		matches!(self, Self::Streamed)
	}
}

/// For requests that mutate the IDGraph, the response result will contain at least:
/// - `mutated_id_graph`: the mutated IDGraph, which is the subset of the whole IDGraph
/// - `id_graph_hash`: the new IDGraph hash after the mutation
///
/// Currently it applies to the following DI requests:
/// - `link_identity`
/// - `activate_identity`
/// - `deactivate_identity`
/// - `set_identity_networks`
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct LinkIdentityResult {
	pub mutated_id_graph: AesOutput,
	pub id_graph_hash: H256,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct DeactivateIdentityResult {
	pub mutated_id_graph: AesOutput,
	pub id_graph_hash: H256,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct ActivateIdentityResult {
	pub mutated_id_graph: AesOutput,
	pub id_graph_hash: H256,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct SetIdentityNetworksResult {
	pub mutated_id_graph: AesOutput,
	pub id_graph_hash: H256,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct RequestVCResult {
	pub vc_payload: AesOutput,
	// Mainly used to returning logs in dynamic contract VC.
	pub vc_logs: Option<AesOutput>,
	// see comments in `lc-vc-task-receiver` why it's prefixed with `pre...`
	// they should be referenced/used only when the client's local IDGraph is empty
	pub pre_mutated_id_graph: AesOutput,
	pub pre_id_graph_hash: H256,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct NewRequestVCResult {
	pub vc_payload: AesOutput,
	// Mainly used to returning logs in dynamic contract VC.
	pub vc_logs: Option<AesOutput>,
	// This should be referenced/used only when the client's local AccountStore is empty
	pub pre_mutated_account_store: AesOutput,
	pub omni_account: AccountId,
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq)]
pub enum RequestVcErrorDetail {
	UnexpectedCall(String),
	DuplicateAssertionRequest,
	ShieldingKeyRetrievalFailed(String), // Stringified itp_sgx_crypto::Error
	RequestPayloadDecodingFailed,
	SidechainDataRetrievalFailed(String), // Stringified itp_stf_state_handler::Error
	IdentityAlreadyLinked,
	NoEligibleIdentity,
	InvalidSignerAccount,
	UnauthorizedSigner,
	AssertionBuildFailed(Box<VCMPError>),
	MissingAesKey,
	MrEnclaveRetrievalFailed,
	EnclaveSignerRetrievalFailed,
	SignatureVerificationFailed,
	ConnectionHashNotFound(String),
	MetadataRetrievalFailed(String), // Stringified itp_node_api_metadata_provider::Error
	InvalidMetadata(String),         // Stringified itp_node_api_metadata::Error
	TrustedCallSendingFailed(String), // Stringified mpsc::SendError<(H256, TrustedCall)>
	CallSendingFailed(String),
	ExtrinsicConstructionFailed(String), // Stringified itp_extrinsics_factory::Error
	ExtrinsicSendingFailed(String),      // Stringified sgx_status_t
	ExtractingMemberIdentityFailed,
	OmniAccountStoreRetrievalFailed,
	CreateAccountStoreFailed(String),
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct RequestVcResultOrError {
	pub result: Result<Vec<u8>, RequestVcErrorDetail>,
	pub idx: u8,
	pub len: u8,
}
