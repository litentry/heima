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

use crate::{dynamic::repository::SmartContractByteCode, *};
use itp_ocall_api::EnclaveMetricsOCallApi;
use itp_types::Assertion;
use lc_credentials::{assertion_logic::AssertionLogic, Credential, IssuerRuntimeVersion};
use lc_dynamic_assertion::{AssertionExecutor, AssertionLogicRepository};
use lc_evm_dynamic_assertions::EvmAssertionExecutor;
use litentry_primitives::AssertionBuildRequest;
use log::error;
use primitive_types::H160;

pub mod repository;

pub fn build<
	SC: AssertionLogicRepository<Id = H160, Item = (SmartContractByteCode, Vec<String>)>,
	MetricsApi: EnclaveMetricsOCallApi,
>(
	req: &AssertionBuildRequest,
	params: DynamicParams,
	repository: Arc<SC>,
	metrics_api: Arc<MetricsApi>,
) -> Result<(Credential, Vec<String>)> {
	let executor = EvmAssertionExecutor { assertion_repository: repository, metrics_api };
	let execution_params = params.clone();
	let result = executor
		.execute(
			execution_params.smart_contract_id,
			execution_params.smart_contract_params.map(|v| v.into()).unwrap_or_default(),
			&req.identities,
		)
		.map_err(|e| {
			Error::RequestVCFailed(
				Assertion::Dynamic(params.clone()),
				ErrorDetail::StfError(ErrorString::truncate_from(e.into())),
			)
		})?;

	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};

	match Credential::new(&req.who, &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			let mut assertion_values: Vec<AssertionLogic> = vec![];
			for assertion in result.assertions {
				let logic: AssertionLogic = serde_json::from_str(&assertion).map_err(|e| {
					Error::RequestVCFailed(
						Assertion::Dynamic(params.clone()),
						ErrorDetail::StfError(ErrorString::truncate_from(format!("{}", e).into())),
					)
				})?;
				assertion_values.push(logic);
			}

			credential_unsigned.update_dynamic(
				result.description,
				result.assertion_type,
				assertion_values,
				result.schema_url,
				result.meet,
			);

			Ok((credential_unsigned, if params.return_log { result.contract_logs } else { vec![] }))
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(Assertion::Dynamic(params), e.into_error_detail()))
		},
	}
}

#[cfg(test)]
pub mod assertion_test {
	use crate::dynamic::{build, repository::InMemorySmartContractRepo};
	use itp_test::mock::metrics_ocall_mock::MetricsOCallMock;
	use itp_types::Assertion;
	use lc_mock_server::run;
	use litentry_hex_utils::decode_hex;
	use litentry_primitives::{
		AssertionBuildRequest, DynamicContractParams, DynamicParams, Identity, IdentityString,
		Web3Network,
	};
	use sp_core::{crypto::AccountId32, H160};
	use std::sync::Arc;

	#[test]
	pub fn test_a20_true() {
		let _ = env_logger::builder().is_test(true).try_init();
		run(19527).unwrap();
		// given
		let twitter_identity = Identity::Twitter(IdentityString::new(vec![]));

		let substrate_identity = Identity::Substrate(
			AccountId32::new([
				212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44,
				133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
			])
			.into(),
		);

		let dynamic_params = DynamicParams {
			smart_contract_id: hash(1),
			smart_contract_params: None,
			return_log: true,
		};
		let request = AssertionBuildRequest {
			shard: Default::default(),
			signer: AccountId32::new([0; 32]),
			who: Identity::Twitter(IdentityString::new(vec![])),
			assertion: Assertion::Dynamic(dynamic_params.clone()),
			identities: vec![(twitter_identity, vec![]), (substrate_identity, vec![])],
			top_hash: Default::default(),
			parachain_block_number: Default::default(),
			sidechain_block_number: Default::default(),
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			req_ext_hash: Default::default(),
			should_create_id_graph: Default::default(),
		};

		let repository = InMemorySmartContractRepo::new();
		let metrics_api = Arc::new(MetricsOCallMock::default());

		// when
		let (credential, vc_logs) =
			build(&request, dynamic_params, repository.into(), metrics_api).unwrap();

		for log in &vc_logs {
			println!("{}", log);
		}
		println!("Credential is: {:?}", credential);

		// then
		assert!(credential.credential_subject.values[0]);
		// assert!(vc_logs.len() == 0);
	}

	#[test]
	pub fn test_a1_true() {
		let _ = env_logger::builder().is_test(true).try_init();
		// given
		let twitter_identity = Identity::Twitter(IdentityString::new(vec![]));
		let substrate_identity = Identity::Substrate(AccountId32::new([0; 32]).into());

		let dynamic_params = DynamicParams {
			smart_contract_id: hash(0),
			smart_contract_params: None,
			return_log: false,
		};
		let request = AssertionBuildRequest {
			shard: Default::default(),
			signer: AccountId32::new([0; 32]),
			who: Identity::Twitter(IdentityString::new(vec![])),
			assertion: Assertion::Dynamic(dynamic_params.clone()),
			identities: vec![(twitter_identity, vec![]), (substrate_identity, vec![])],
			top_hash: Default::default(),
			parachain_block_number: Default::default(),
			sidechain_block_number: Default::default(),
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			req_ext_hash: Default::default(),
			should_create_id_graph: Default::default(),
		};

		let repository = InMemorySmartContractRepo::new();
		let metrics_api = Arc::new(MetricsOCallMock::default());

		// when
		let (credential, _) =
			build(&request, dynamic_params, repository.into(), metrics_api).unwrap();

		println!("Credential is: {:?}", credential);

		// then
		assert!(credential.credential_subject.values[0]);
	}

	#[test]
	pub fn test_a6_true() {
		let _ = env_logger::builder().is_test(true).try_init();
		run(19528).unwrap();
		// given
		let twitter_identity =
			Identity::Twitter(IdentityString::new("twitterdev".as_bytes().to_vec()));
		let substrate_identity = Identity::Substrate(AccountId32::new([0; 32]).into());

		let dynamic_params = DynamicParams {
			smart_contract_id: hash(2),
			smart_contract_params: None,
			return_log: false,
		};
		let request = AssertionBuildRequest {
			shard: Default::default(),
			signer: AccountId32::new([0; 32]),
			who: Identity::Twitter(IdentityString::new(vec![])),
			assertion: Assertion::Dynamic(dynamic_params.clone()),
			identities: vec![(twitter_identity, vec![]), (substrate_identity, vec![])],
			top_hash: Default::default(),
			parachain_block_number: Default::default(),
			sidechain_block_number: Default::default(),
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			req_ext_hash: Default::default(),
			should_create_id_graph: Default::default(),
		};

		let repository = InMemorySmartContractRepo::new();
		let metrics_api = Arc::new(MetricsOCallMock::default());

		// when
		let (credential, _) =
			build(&request, dynamic_params, repository.into(), metrics_api).unwrap();

		println!("Credential is: {:?}", credential);

		// then
		assert!(credential.credential_subject.values[0]);
	}

	#[test]
	pub fn test_a1_false() {
		let _ = env_logger::builder().is_test(true).try_init();
		// given
		let twitter_identity = Identity::Twitter(IdentityString::new(vec![]));

		let dynamic_params = DynamicParams {
			smart_contract_id: hash(0),
			smart_contract_params: None,
			return_log: false,
		};
		let request = AssertionBuildRequest {
			shard: Default::default(),
			signer: AccountId32::new([0; 32]),
			who: Identity::Twitter(IdentityString::new(vec![])),
			assertion: Assertion::Dynamic(dynamic_params.clone()),
			identities: vec![(twitter_identity, vec![])],
			top_hash: Default::default(),
			parachain_block_number: Default::default(),
			sidechain_block_number: Default::default(),
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			req_ext_hash: Default::default(),
			should_create_id_graph: Default::default(),
		};

		let repository = InMemorySmartContractRepo::new();
		let metrics_api = Arc::new(MetricsOCallMock::default());

		// when
		let (credential, _) =
			build(&request, dynamic_params, repository.into(), metrics_api).unwrap();

		// then
		assert!(!credential.credential_subject.values[0]);
	}

	#[test]
	pub fn test_token_holding_amount_ordi_true() {
		let _ = env_logger::builder().is_test(true).try_init();
		run(19529).unwrap();
		// given
		// bc1pgr5fw4p9gl9me0vzjklnlnap669caxc0gsk4j62gff2qktlw6naqm4m3d0
		let address = decode_hex(
			"0x02e8c39e82aaaa143c3def8d3c7084a539b227244ac9067c3f7fc86cb73a0b7aed".as_bytes(),
		)
		.unwrap()
		.as_slice()
		.try_into()
		.unwrap();

		let network = Web3Network::BitcoinP2tr;
		let identities = vec![(Identity::Bitcoin(address), vec![network])];

		let dynamic_params = DynamicParams {
			smart_contract_id: hash(3),
			smart_contract_params: Some(DynamicContractParams::truncate_from(ethabi::encode(&[
				ethabi::Token::String("ordi".into()),
			]))),
			return_log: false,
		};
		let request = AssertionBuildRequest {
			shard: Default::default(),
			signer: AccountId32::new([0; 32]),
			who: Identity::Substrate(AccountId32::new([0; 32]).into()),
			assertion: Assertion::Dynamic(dynamic_params.clone()),
			identities,
			top_hash: Default::default(),
			parachain_block_number: Default::default(),
			sidechain_block_number: Default::default(),
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			req_ext_hash: Default::default(),
			should_create_id_graph: Default::default(),
		};

		let repository = InMemorySmartContractRepo::new();
		let metrics_api = Arc::new(MetricsOCallMock::default());

		// when
		let (credential, _) =
			build(&request, dynamic_params, repository.into(), metrics_api).unwrap();

		println!("Credential is: {:?}", credential);

		// then
		assert!(credential.credential_subject.values[0]);
	}

	fn hash(a: u64) -> H160 {
		H160::from_low_u64_be(a)
	}
}
