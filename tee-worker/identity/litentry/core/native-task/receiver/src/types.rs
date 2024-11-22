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

use crate::handlers::RequestVcOk;
use alloc::{string::String, sync::Arc, vec::Vec};
use codec::{Decode, Encode};
use ita_sgx_runtime::Hash;
use ita_stf::{
	trusted_call_result::{RequestVcErrorDetail, RequestVcResultOrError},
	AesOutput, Getter, TrustedCallSigned,
};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::{
	api_client::{ExtrinsicReport, TransactionStatus},
	metadata::{provider::AccessNodeMetadata, NodeMetadata},
};
use itp_ocall_api::{EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{
	aes256::Aes256Key, key_repository::AccessKey, ShieldingCryptoDecrypt, ShieldingCryptoEncrypt,
};
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_stf_state_handler::handle_state::HandleState;
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use itp_types::AccountId;
use lc_data_providers::DataProviderConfig;
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use sp_core::{ed25519::Pair as Ed25519Pair, H160};

pub struct NativeTaskContext<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>
where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AA: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	SES: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OA: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + 'static,
	EF: CreateExtrinsics + Send + Sync + 'static,
	NMR: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	AKR: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
	SH: HandleState + Send + Sync + 'static,
{
	pub shielding_key: Arc<ShieldingKeyRepository>,
	pub author_api: Arc<AA>,
	pub enclave_signer: Arc<SES>,
	pub enclave_account: Arc<Ed25519Pair>,
	pub ocall_api: Arc<OA>,
	pub data_provider_config: Arc<DataProviderConfig>,
	pub extrinsic_factory: Arc<EF>,
	pub node_metadata_repo: Arc<NMR>,
	pub aes256_key_repository: Arc<AKR>,
	pub assertion_repository: Arc<AR>,
	pub state_handler: Arc<SH>,
}

impl<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>
	NativeTaskContext<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>
where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AA: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	SES: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OA: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + 'static,
	EF: CreateExtrinsics + Send + Sync + 'static,
	NMR: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	AKR: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
	SH: HandleState + Send + Sync + 'static,
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: Arc<ShieldingKeyRepository>,
		author_api: Arc<AA>,
		enclave_signer: Arc<SES>,
		enclave_account: Arc<Ed25519Pair>,
		ocall_api: Arc<OA>,
		data_provider_config: Arc<DataProviderConfig>,
		extrinsic_factory: Arc<EF>,
		node_metadata_repo: Arc<NMR>,
		aes256_key_repository: Arc<AKR>,
		assertion_repository: Arc<AR>,
		state_handler: Arc<SH>,
	) -> Self {
		Self {
			shielding_key,
			author_api,
			enclave_signer,
			enclave_account,
			ocall_api,
			data_provider_config,
			extrinsic_factory,
			node_metadata_repo,
			aes256_key_repository,
			assertion_repository,
			state_handler,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTaskError {
	UnexpectedCall(String),
	ShieldingKeyRetrievalFailed(String), // Stringified itp_sgx_crypto::Error
	RequestPayloadDecodingFailed,
	ParentchainDataRetrievalFailed(String), // Stringified itp_stf_state_handler::Error
	InvalidSignerAccount,
	UnauthorizedSigner,
	InvalidMemberIdentity,
	MissingAesKey,
	MrEnclaveRetrievalFailed,
	EnclaveSignerRetrievalFailed,
	AuthenticationVerificationFailed,
	ValidationDataVerificationFailed,
	ConnectionHashNotFound(String),
	MetadataRetrievalFailed(String), // Stringified itp_node_api_metadata_provider::Error
	InvalidMetadata(String),         // Stringified itp_node_api_metadata::Error
	TrustedCallSendingFailed(String), // Stringified mpsc::SendError<(H256, TrustedCall)>
	CallSendingFailed(String),
	ExtrinsicConstructionFailed(String), // Stringified itp_extrinsics_factory::Error
	ExtrinsicSendingFailed(String),      // Stringified sgx_status_t
	InvalidRequest,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTaskOk<Hash: Decode> {
	ExtrinsicReport {
		// Hash of the extrinsic.
		extrinsic_hash: Hash,
		// Block hash of the block the extrinsic was included in.
		// Only available if watched until at least `InBlock`.
		block_hash: Option<Hash>,
		// Last known Transaction Status.
		status: TransactionStatus<Hash, Hash>,
	},
	RequestVcResult {
		// Vec<u8> == handlers::request_vc_handlers::RequestVcOk Encoded
		result: Result<Vec<u8>, RequestVcErrorDetail>,
		idx: u8,
		len: u8,
	},
}

impl<Hash: Decode + Clone> From<&ExtrinsicReport<Hash>> for NativeTaskOk<Hash> {
	fn from(report: &ExtrinsicReport<Hash>) -> Self {
		NativeTaskOk::ExtrinsicReport {
			extrinsic_hash: report.extrinsic_hash.clone(),
			block_hash: report.block_hash.clone(),
			status: report.status.clone().into(),
		}
	}
}

impl<Hash: Decode> From<RequestVcResultOrError> for NativeTaskOk<Hash> {
	fn from(result: RequestVcResultOrError) -> Self {
		NativeTaskOk::RequestVcResult { result: result.result, idx: result.idx, len: result.len }
	}
}
