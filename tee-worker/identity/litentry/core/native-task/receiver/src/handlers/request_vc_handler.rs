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
use codec::Decode;
use frame_support::ensure;
use ita_sgx_runtime::VERSION as SIDECHAIN_VERSION;
use ita_stf::{
	helpers::ensure_self, trusted_call_result::RequestVcErrorDetail, Getter, TrustedCall,
	TrustedCallSigned,
};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::metadata::{
	pallet_system::SystemConstants, provider::AccessNodeMetadata, NodeMetadata,
};
use itp_ocall_api::{EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{
	aes256::Aes256Key, aes_decrypt, key_repository::AccessKey, AesOutput, ShieldingCryptoDecrypt,
	ShieldingCryptoEncrypt,
};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_stf_state_handler::handle_state::HandleState;
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use itp_types::parentchain::ParachainHeader;
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use lc_omni_account::InMemoryStore as OmniAccountStore;
use lc_stf_task_receiver::handler::assertion::create_credential_str;
use litentry_macros::if_development_or;
use litentry_primitives::{
	Assertion, AssertionBuildRequest, Identity, IdentityNetworkTuple, MemberAccount, RequestAesKey,
	Web3Network,
};
use sp_core::{H160, H256 as Hash};

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
) -> Result<Vec<u8>, RequestVcErrorDetail>
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
	debug!(
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

	let mut account_store = match OmniAccountStore::get_omni_account(who.hash()) {
		Ok(Some(account_id)) => match OmniAccountStore::get_member_accounts(&account_id) {
			Ok(Some(member_accounts)) => member_accounts,
			_ => Vec::new(),
		},
		_ => Vec::new(),
	};

	if account_store.is_empty() {
		// TODO: create AccountStore on chain
		let member_account = MemberAccount::Public(who.clone());
		account_store.push(member_account);
	}

	let member_identities = account_store
		.iter()
		.map(|m| extract_identity_from_member(context.aes256_key_repository.clone(), m))
		.collect::<Result<Vec<Identity>, &'static str>>()
		.map_err(|e| RequestVcErrorDetail::ExtractingMemberIdentityFailed)?;
	let identities = get_elegible_identities(member_identities, &assertion);

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
		should_create_id_graph: false,
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

	// TODO: create new vc_issued call in the pallet

	todo!()
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
			let encoded_identity = aes_decrypt(&key, &mut output).ok_or("Failed to decrypt")?;
			Identity::decode(&mut encoded_identity.as_slice()).map_err(|_| "Failed to decode")
		},
	}
}

fn get_elegible_identities(
	member_identities: Vec<Identity>,
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
