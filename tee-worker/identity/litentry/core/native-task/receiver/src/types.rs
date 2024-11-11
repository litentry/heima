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

use codec::{Decode, Encode};
use ita_sgx_runtime::Hash;
use ita_stf::{Getter, TrustedCallSigned};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::metadata::{provider::AccessNodeMetadata, NodeMetadata};
use itp_ocall_api::{EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{
	aes256::Aes256Key, key_repository::AccessKey, ShieldingCryptoDecrypt, ShieldingCryptoEncrypt,
};
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use lc_data_providers::DataProviderConfig;
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use sp_core::{ed25519::Pair as Ed25519Pair, H160};
use std::{string::String, sync::Arc};

pub struct NativeTaskContext<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
	Aes256KeyRepository,
	AssertionRepository,
> where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AuthorApi: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfEnclaveSigning: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OCallApi: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + 'static,
	ExtrinsicFactory: CreateExtrinsics + Send + Sync + 'static,
	NodeMetadataRepo: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	Aes256KeyRepository: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
	AssertionRepository:
		AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
{
	pub shielding_key: Arc<ShieldingKeyRepository>,
	pub author_api: Arc<AuthorApi>,
	pub enclave_signer: Arc<StfEnclaveSigning>,
	pub enclave_account: Arc<Ed25519Pair>,
	pub ocall_api: Arc<OCallApi>,
	pub data_provider_config: Arc<DataProviderConfig>,
	pub extrinsic_factory: Arc<ExtrinsicFactory>,
	pub node_metadata_repo: Arc<NodeMetadataRepo>,
	pub aes256_key_repository: Arc<Aes256KeyRepository>,
	pub assertion_repository: Arc<AssertionRepository>,
}

impl<
		ShieldingKeyRepository,
		AuthorApi,
		StfEnclaveSigning,
		OCallApi,
		ExtrinsicFactory,
		NodeMetadataRepo,
		Aes256KeyRepository,
		AssertionRepository,
	>
	NativeTaskContext<
		ShieldingKeyRepository,
		AuthorApi,
		StfEnclaveSigning,
		OCallApi,
		ExtrinsicFactory,
		NodeMetadataRepo,
		Aes256KeyRepository,
		AssertionRepository,
	> where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AuthorApi: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfEnclaveSigning: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OCallApi: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + 'static,
	ExtrinsicFactory: CreateExtrinsics + Send + Sync + 'static,
	NodeMetadataRepo: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	Aes256KeyRepository: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
	AssertionRepository:
		AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: Arc<ShieldingKeyRepository>,
		author_api: Arc<AuthorApi>,
		enclave_signer: Arc<StfEnclaveSigning>,
		enclave_account: Arc<Ed25519Pair>,
		ocall_api: Arc<OCallApi>,
		data_provider_config: Arc<DataProviderConfig>,
		extrinsic_factory: Arc<ExtrinsicFactory>,
		node_metadata_repo: Arc<NodeMetadataRepo>,
		aes256_key_repository: Arc<Aes256KeyRepository>,
		assertion_repository: Arc<AssertionRepository>,
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
	NativeRequestSendFailed,
}
