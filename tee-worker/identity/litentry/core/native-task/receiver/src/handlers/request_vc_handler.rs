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

use crate::NativeTaskContext;
use alloc::{boxed::Box, format, string::ToString, sync::Arc, vec::Vec};
use codec::{Decode, Encode};
use frame_support::ensure;
use ita_sgx_runtime::VERSION as SIDECHAIN_VERSION;
use ita_stf::{
	aes_encrypt_default, helpers::ensure_self, trusted_call_result::RequestVcErrorDetail, Getter,
	TrustedCallSigned,
};
use itp_enclave_metrics::EnclaveMetric;
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::{
	api_client::compose_call,
	metadata::{pallet_system::SystemConstants, provider::AccessNodeMetadata, NodeMetadata},
};
use itp_ocall_api::{EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{
	aes256::Aes256Key, aes_decrypt, key_repository::AccessKey, AesOutput, ShieldingCryptoDecrypt,
	ShieldingCryptoEncrypt,
};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_stf_state_handler::handle_state::HandleState;
use itp_storage::storage_value_key;
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use itp_types::{
	parentchain::ParentchainId, AccountId, BlockNumber as SidechainBlockNumber, OpaqueCall,
};
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use lc_omni_account::InMemoryStore as OmniAccountStore;
use lc_stf_task_receiver::handler::assertion::create_credential_str;
use litentry_macros::if_development_or;
use litentry_primitives::{
	Assertion, AssertionBuildRequest, Identity, IdentityNetworkTuple, MemberAccount,
	ParentchainBlockNumber, RequestAesKey,
};
use sp_core::{H160, H256 as Hash};
use std::time::Instant;

pub struct RequestVCResult {
	pub vc_payload: AesOutput,
	pub vc_logs: Option<AesOutput>,
	pub pre_mutated_account_store: AesOutput,
	pub omni_account: AccountId,
}

pub fn handle_request_vc<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
	Aes256KeyRepository,
	AssertionRepository,
	StateHandler,
>(
	context: Arc<
		NativeTaskContext<
			ShieldingKeyRepository,
			AuthorApi,
			StfEnclaveSigning,
			OCallApi,
			ExtrinsicFactory,
			NodeMetadataRepo,
			Aes256KeyRepository,
			AssertionRepository,
			StateHandler,
		>,
	>,
	shard: Hash,
	signer: Identity,
	who: Identity,
	assertion: Assertion,
	maybe_key: Option<RequestAesKey>,
	req_ext_hash: Hash,
) -> Result<RequestVCResult, RequestVcErrorDetail>
where
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
	StateHandler: HandleState + Send + Sync + 'static,
	StateHandler::StateT: SgxExternalitiesTrait,
{
	let start_time = Instant::now();
	log::debug!(
		"Processing vc request for {}, assertion: {:?}",
		who.to_did().unwrap_or_default(),
		assertion
	);
	match assertion {
		Assertion::A13(_) => (),
		_ => if_development_or!(
			ensure!(
				ensure_self(&signer, &who) || ensure_alice(&signer_account),
				RequestVcErrorDetail::UnauthorizedSigner,
			),
			ensure!(ensure_self(&signer, &who), RequestVcErrorDetail::UnauthorizedSigner)
		),
	}

	let (parachain_block_number, sidechain_block_number) = context
		.state_handler
		.execute_on_current(&shard, |state, _| {
			// should never be `None`, but use `unwrap_or_default` to not panic
			let parachain_block_number = state
				.get(&storage_value_key("Parentchain", "Number"))
				.and_then(|v| ParentchainBlockNumber::decode(&mut v.as_slice()).ok())
				.unwrap_or_default();
			let sidechain_block_number = state
				.get(&storage_value_key("System", "Number"))
				.and_then(|v| SidechainBlockNumber::decode(&mut v.as_slice()).ok())
				.unwrap_or_default();

			(parachain_block_number, sidechain_block_number)
		})
		.map_err(|e| RequestVcErrorDetail::SidechainDataRetrievalFailed(e.to_string()))?;

	let omni_account = match OmniAccountStore::get_omni_account(who.hash()) {
		Ok(Some(account)) => account,
		Ok(None) => who.to_omni_account(),
		Err(_) => return Err(RequestVcErrorDetail::OmniAccountStoreRetrievalFailed),
	};

	let mut account_store = match OmniAccountStore::get_member_accounts(&omni_account) {
		Ok(Some(member_accounts)) => member_accounts,
		_ => Vec::new(),
	};

	let metadata = match context.node_metadata_repo.get_from_metadata(|m| m.get_metadata().cloned())
	{
		Ok(Some(metadata)) => metadata,
		_ =>
			return Err(RequestVcErrorDetail::MetadataRetrievalFailed(
				"Failed to retrieve metadata".to_string(),
			)),
	};

	let mut is_new_account_store = false;
	if account_store.is_empty() {
		is_new_account_store = true;
		let create_account_store_call = OpaqueCall::from_tuple(&compose_call!(
			&metadata,
			"OmniAccount",
			"create_account_store",
			who.clone()
		));
		let xt = context
			.extrinsic_factory
			.create_extrinsics(&[create_account_store_call], None)
			.map_err(|e| RequestVcErrorDetail::CreateAccountStoreFailed(e.to_string()))?;
		context
			.ocall_api
			.send_to_parentchain(xt, &ParentchainId::Litentry, None)
			.map_err(|e| RequestVcErrorDetail::CreateAccountStoreFailed(e.to_string()))?;

		log::info!("Account store created for {}", who.to_did().unwrap_or_default());

		let member_account = MemberAccount::Public(who.clone());
		account_store.push(member_account);
	}

	let member_identities = account_store
		.iter()
		.map(|member| extract_identity_from_member(context.aes256_key_repository.clone(), member))
		.collect::<Result<Vec<Identity>, &'static str>>()
		.map_err(|_| RequestVcErrorDetail::ExtractingMemberIdentityFailed)?;
	let identities = get_elegible_identities(&member_identities, &assertion);

	ensure!(!identities.is_empty(), RequestVcErrorDetail::NoEligibleIdentity);

	let signer_account =
		signer.to_native_account().ok_or(RequestVcErrorDetail::InvalidSignerAccount)?;
	let parachain_runtime_version = context
		.node_metadata_repo
		.get_from_metadata(|m| {
			m.system_version()
				.map_err(|e| RequestVcErrorDetail::InvalidMetadata(format!("{:?}", e)))
		})
		.map_err(|e| RequestVcErrorDetail::MetadataRetrievalFailed(e.to_string()))??
		.spec_version;
	let sidechain_runtime_version = SIDECHAIN_VERSION.spec_version;

	let req = AssertionBuildRequest {
		shard,
		signer: signer_account,
		who: who.clone(),
		assertion: assertion.clone(),
		identities,
		top_hash: Hash::zero(),
		parachain_block_number,
		sidechain_block_number,
		parachain_runtime_version,
		sidechain_runtime_version,
		maybe_key,
		should_create_id_graph: is_new_account_store,
		req_ext_hash,
	};

	let (vc_payload, vc_logs) =
		create_credential_str::<AuthorApi, StfEnclaveSigning, OCallApi, AssertionRepository>(
			&req,
			context.enclave_signer.clone(),
			context.enclave_account.clone(),
			context.ocall_api.clone(),
			context.data_provider_config.clone(),
			context.assertion_repository.clone(),
		)
		.map_err(|e| RequestVcErrorDetail::AssertionBuildFailed(Box::new(e)))?;

	let key = maybe_key.ok_or(RequestVcErrorDetail::MissingAesKey)?;

	let mutated_account_store =
		if is_new_account_store { member_identities } else { Default::default() };

	let vc_result = RequestVCResult {
		vc_payload: aes_encrypt_default(&key, &vc_payload),
		vc_logs: vc_logs.map(|log| aes_encrypt_default(&key, &log)),
		pre_mutated_account_store: aes_encrypt_default(&key, &mutated_account_store.encode()),
		omni_account: omni_account.clone(),
	};

	let on_vc_issued_call = OpaqueCall::from_tuple(&compose_call!(
		metadata,
		"VCManagement",
		"on_vc_issued",
		who.clone(),
		omni_account,
		req_ext_hash
	));

	let xt = context
		.extrinsic_factory
		.create_extrinsics(&[on_vc_issued_call], None)
		.map_err(|e| RequestVcErrorDetail::ExtrinsicConstructionFailed(e.to_string()))?;

	context
		.ocall_api
		.send_to_parentchain(xt, &ParentchainId::Litentry, None)
		.map_err(|e| RequestVcErrorDetail::ExtrinsicSendingFailed(e.to_string()))?;

	if let Err(e) = context
		.ocall_api
		.update_metric(EnclaveMetric::VCBuildTime(assertion.clone(), start_time.elapsed()))
	{
		log::warn!("Failed to update metric for vc build time: {:?}", e);
	}
	log::info!("Vc issued for {}, assertion: {:?}", who.to_did().unwrap_or_default(), assertion);

	Ok(vc_result)
}

fn extract_identity_from_member<Aes256KeyRepository>(
	key_repository: Arc<Aes256KeyRepository>,
	member_account: &MemberAccount,
) -> Result<Identity, &'static str>
where
	Aes256KeyRepository: AccessKey<KeyType = Aes256Key>,
{
	match member_account {
		MemberAccount::Public(identity) => Ok(identity.clone()),
		MemberAccount::Private(encrypted_identity, _hash) => {
			let aes_key = match key_repository.retrieve_key() {
				Ok(key) => Ok(key),
				Err(e) => {
					log::error!("Failed to retrieve aes key: {:?}", e);
					Err("Failed to retrieve aes key")
				},
			}?;
			let mut output: AesOutput = Decode::decode(&mut encrypted_identity.as_slice())
				.map_err(|_| "Failed to decode encrypted identity")?;
			let encoded_identity = aes_decrypt(&aes_key, &mut output).ok_or("Failed to decrypt")?;
			Identity::decode(&mut encoded_identity.as_slice()).map_err(|_| "Failed to decode")
		},
	}
}

fn get_elegible_identities(
	member_identities: &Vec<Identity>,
	assertion: &Assertion,
) -> Vec<IdentityNetworkTuple> {
	let supported_networks = assertion.get_supported_web3networks();
	let skip_filtering = assertion.skip_identity_filtering();
	member_identities
		.iter()
		.filter_map(|identity| {
			let mut networks = identity.default_web3networks();

			// if `skip_filtering` is true, we basically let assertion logic itself to handle those identities.
			if skip_filtering {
				return Some((identity.clone(), networks))
			}

			networks.retain(|n| supported_networks.contains(n));

			// differentiate between web2 and web3 assertions:
			// supported_web3networks.is_empty() means it's a web2 assertion,
			// otherwise web2 identities might survive to be unexpectedly "eligible" for web3 assertions.
			if supported_networks.is_empty() {
				if identity.is_web2() {
					Some((identity.clone(), networks))
				} else {
					None
				}
			} else {
				if identity.is_web3() && !networks.is_empty() {
					Some((identity.clone(), networks))
				} else {
					None
				}
			}
		})
		.collect()
}
