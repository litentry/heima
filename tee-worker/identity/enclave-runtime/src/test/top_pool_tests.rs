/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
	Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

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

use crate::test::{
	fixtures::{
		components::{
			create_ocall_api, create_top_pool, encrypt_trusted_operation, sign_trusted_call,
		},
		initialize_test_state::init_state,
	},
	mocks::types::{
		TestShieldingKey, TestShieldingKeyRepo, TestSigner, TestStateHandler, TestTopPoolAuthor,
	},
};
use ita_stf::{
	test_genesis::{endowed_account, unendowed_account},
	Getter, TrustedCall, TrustedCallSigned,
};
use itc_parentchain_test::parentchain_header_builder::ParentchainHeaderBuilder;
use itp_ocall_api::EnclaveAttestationOCallApi;
use itp_sgx_crypto::ShieldingCryptoEncrypt;

use itp_stf_primitives::types::TrustedOperation;
use itp_test::mock::metrics_ocall_mock::MetricsOCallMock;
use itp_top_pool_author::{
	top_filter::{AllowAllTopsFilter, DirectCallsOnlyFilter},
	traits::AuthorApi,
};
use itp_types::{RsaRequest, ShardIdentifier};
use jsonrpc_core::futures::executor;
use litentry_primitives::Identity;
use log::*;
use sgx_crypto_helper::RsaKeyPair;
use sp_core::Pair;
use std::{sync::Arc, vec::Vec};
pub fn process_indirect_call_in_top_pool() {
	let _ = env_logger::builder().is_test(true).try_init();
	info!("Setting up test.");

	let signer = TestSigner::from_seed(b"42315678901234567890123456789012");
	let shielding_key = TestShieldingKey::new().unwrap();
	let shielding_key_repo = Arc::new(TestShieldingKeyRepo::new(shielding_key));
	let header = ParentchainHeaderBuilder::default().build();

	let ocall_api = create_ocall_api(&header, &signer);

	let state_handler = Arc::new(TestStateHandler::default());
	let (_, shard_id) = init_state(state_handler.as_ref(), signer.public().into());

	let top_pool = create_top_pool();
	let (sender, _receiver) = std::sync::mpsc::sync_channel(1000);

	let top_pool_author = Arc::new(TestTopPoolAuthor::new(
		top_pool,
		AllowAllTopsFilter::<TrustedCallSigned, Getter>::new(),
		DirectCallsOnlyFilter::<TrustedCallSigned, Getter>::new(),
		state_handler,
		shielding_key_repo,
		Arc::new(MetricsOCallMock::default()),
		Arc::new(sender),
	));

	let encrypted_indirect_call =
		encrypted_indirect_call(ocall_api.as_ref(), &shard_id, &shielding_key);

	executor::block_on(
		top_pool_author.submit_top(RsaRequest::new(shard_id, encrypted_indirect_call)),
	)
	.unwrap();

	assert_eq!(1, top_pool_author.get_pending_trusted_calls(shard_id).len());
}

/*

// TODO: use our trusted call for testing - see P-494

pub fn submit_shielding_call_to_top_pool() {
	let _ = env_logger::builder().is_test(true).try_init();

	let signer = TestSigner::from_seed(b"42315678901234567890123456789012");
	let shielding_key = TestShieldingKey::new().unwrap();
	let shielding_key_repo = Arc::new(TestShieldingKeyRepo::new(shielding_key));
	let header = ParentchainHeaderBuilder::default().build();

	let ocall_api = create_ocall_api(&header, &signer);
	let mr_enclave = ocall_api.get_mrenclave_of_self().unwrap();

	let state_handler = Arc::new(TestStateHandler::default());
	let (state, shard_id) = init_state(state_handler.as_ref(), signer.public().into());
	let state_observer = Arc::new(ObserveStateMock::new(state));

	let top_pool = create_top_pool();
	let (sender, _receiver) = std::sync::mpsc::sync_channel(1000);

	let top_pool_author = Arc::new(TestTopPoolAuthor::new(
		top_pool,
		AllowAllTopsFilter::<TrustedCallSigned, Getter>::new(),
		DirectCallsOnlyFilter::<TrustedCallSigned, Getter>::new(),
		state_handler,
		shielding_key_repo.clone(),
		Arc::new(MetricsOCallMock::default()),
		Arc::new(sender),
	));

	let enclave_signer =
		Arc::new(StfEnclaveSigner::<_, _, _, TestStf, _, TrustedCallSigned, Getter>::new(
			state_observer,
			ocall_api,
			shielding_key_repo.clone(),
			top_pool_author.clone(),
		));
	let node_meta_data_repository = Arc::new(NodeMetadataRepository::default());
	node_meta_data_repository.set_metadata(NodeMetadataMock::new());
	let indirect_calls_executor = IndirectCallsExecutor::<
		_,
		_,
		_,
		_,
		integritee::ExtrinsicFilter,
		TestEventCreator,
		integritee::ParentchainEventHandler,
		TrustedCallSigned,
		Getter,
	>::new(
		shielding_key_repo,
		enclave_signer,
		top_pool_author.clone(),
		node_meta_data_repository,
		ParentchainId::Litentry,
	);

	let block_with_shielding_call = create_opaque_call_extrinsic(shard_id, &shielding_key);

	let _ = indirect_calls_executor
		.execute_indirect_calls_in_extrinsics(&block_with_shielding_call, &Vec::new())
		.unwrap();

	assert_eq!(1, top_pool_author.get_pending_trusted_calls(shard_id).len());
	let trusted_operation =
		top_pool_author.get_pending_trusted_calls(shard_id).first().cloned().unwrap();
	let trusted_call = trusted_operation.to_call().unwrap();
	assert!(trusted_call.verify_signature(&mr_enclave.m, &shard_id));
}
*/

fn encrypted_indirect_call<
	AttestationApi: EnclaveAttestationOCallApi,
	ShieldingKey: ShieldingCryptoEncrypt,
>(
	attestation_api: &AttestationApi,
	shard_id: &ShardIdentifier,
	shielding_key: &ShieldingKey,
) -> Vec<u8> {
	let sender = endowed_account();
	let receiver = unendowed_account();

	let call = TrustedCall::balance_transfer(
		Identity::Substrate(sender.public().into()),
		receiver.public().into(),
		10000u128,
	);
	let call_signed = sign_trusted_call(&call, attestation_api, shard_id, sender);
	let trusted_operation =
		TrustedOperation::<TrustedCallSigned, Getter>::indirect_call(call_signed);
	encrypt_trusted_operation(shielding_key, &trusted_operation)
}

/*
fn create_opaque_call_extrinsic<ShieldingKey: ShieldingCryptoEncrypt>(
	_shard: ShardIdentifier,
	_shielding_key: &ShieldingKey,
) -> Block {
	let test_signer = ed25519::Pair::from_seed(b"33345678901234567890123456789012");
	let signature = test_signer.sign(&[0u8]);

	let default_extra_for_test = ParentchainExtrinsicParams::new(
		0,
		0,
		0,
		H256::default(),
		ParentchainAdditionalParams::default(),
	);

	let dummy_node_metadata = NodeMetadataMock::new();

	let call_index = dummy_node_metadata.post_opaque_task_call_indexes().unwrap();
	let opaque_extrinsic = OpaqueExtrinsic::from_bytes(
		ParentchainUncheckedExtrinsic::<CallWorkerFn>::new_signed(
			(call_index, RsaRequest::default()),
			Address::Address32([1u8; 32]),
			MultiSignature::Ed25519(signature),
			default_extra_for_test.signed_extra(),
		)
		.encode()
		.as_slice(),
	)
	.unwrap();

	ParentchainBlockBuilder::default()
		.with_extrinsics(vec![opaque_extrinsic])
		.build()
}

*/
