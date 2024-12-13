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

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use crate::*;
use lc_common::abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls};
use lc_credentials::{
	generic_discord_role::GenericDiscordRoleAssertionUpdate, Credential, IssuerRuntimeVersion,
};
use lc_data_providers::{
	discord_litentry::DiscordLitentryClient, DataProviderConfig, Error as DataProviderError,
};
use litentry_primitives::{
	AssertionBuildRequest, ContestType, GenericDiscordRoleType, SoraQuizType,
};
use std::string::ToString;

pub fn build(
	req: &AssertionBuildRequest,
	rtype: GenericDiscordRoleType,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	let role_id =
		get_generic_discord_role_id(&rtype, data_provider_config).map_err(|error_detail| {
			Error::RequestVCFailed(Assertion::GenericDiscordRole(rtype.clone()), error_detail)
		})?;

	let identities = req
		.identities
		.iter()
		.map(|(identity, _)| identity.clone())
		.collect::<Vec<Identity>>();

	let mut has_role_value = false;
	let mut client =
		DiscordLitentryClient::new(&data_provider_config.litentry_discord_microservice_url);

	loop_with_abort_strategy::<fn(&_) -> bool, Identity, DataProviderError>(
		identities,
		|identity| {
			if let Identity::Discord(address) = &identity {
				match client.has_role(role_id.clone(), address.inner_ref().to_vec()) {
					Ok(resp) => {
						debug!("Litentry & Discord user has role response: {:?}", resp);
						// data is true if the user has the specified role, otherwise, it is false.
						if resp.data {
							has_role_value = true;
							Ok(LoopControls::Break)
						} else {
							Ok(LoopControls::Continue)
						}
					},
					Err(err) => Err(err),
				}
			} else {
				Ok(LoopControls::Continue)
			}
		},
		AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
	)
	.map_err(|errors| {
		Error::RequestVCFailed(
			Assertion::GenericDiscordRole(rtype.clone()),
			errors[0].clone().into_error_detail(),
		)
	})?;

	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};

	match Credential::new(&req.who, &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.update_generic_discord_role_assertion(rtype, has_role_value);
			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential GenericDiscordRole failed {:?}", e);
			Err(Error::RequestVCFailed(Assertion::GenericDiscordRole(rtype), e.into_error_detail()))
		},
	}
}

fn get_generic_discord_role_id(
	rtype: &GenericDiscordRoleType,
	data_provider_config: &DataProviderConfig,
) -> core::result::Result<String, ErrorDetail> {
	match rtype {
		GenericDiscordRoleType::Contest(ctype) => match ctype {
			ContestType::Legend =>
				Ok(data_provider_config.contest_legend_discord_role_id.to_string()),
			ContestType::Popularity =>
				Ok(data_provider_config.contest_popularity_discord_role_id.to_string()),
			ContestType::Participant =>
				Ok(data_provider_config.contest_participant_discord_role_id.to_string()),
		},
		GenericDiscordRoleType::SoraQuiz(qtype) => match qtype {
			SoraQuizType::Attendee => Ok(data_provider_config.sora_quiz_attendee_id.to_string()),
			SoraQuizType::Master => Ok(data_provider_config.sora_quiz_master_id.to_string()),
		},
	}
}

#[cfg(test)]
mod tests {
	use crate::{generic_discord_role::build, AccountId, AssertionBuildRequest};
	use itp_stf_primitives::types::ShardIdentifier;
	use lc_credentials::assertion_logic::{AssertionLogic, Op};
	use lc_data_providers::DataProviderConfig;
	use lc_mock_server::run;
	use litentry_primitives::{
		Assertion, ContestType, GenericDiscordRoleType, Identity, IdentityNetworkTuple,
		IdentityString, SoraQuizType,
	};
	use log;
	use std::{vec, vec::Vec};

	fn init() -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap();
		let mut data_provider_conifg = DataProviderConfig::new().unwrap();

		data_provider_conifg.set_litentry_discord_microservice_url(url).unwrap();
		data_provider_conifg.set_contest_legend_discord_role_id("1034083718425493544".to_string());
		data_provider_conifg.set_sora_quiz_attendee_id("1034083718425493544".to_string());
		data_provider_conifg
	}

	#[test]
	fn build_contest_role_works() {
		let data_provider_config = init();

		let handler_vec: Vec<u8> = "againstwar".to_string().as_bytes().to_vec();

		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Discord(IdentityString::new(handler_vec)), vec![])];

		let req: AssertionBuildRequest = AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::GenericDiscordRole(GenericDiscordRoleType::Contest(
				ContestType::Legend,
			)),
			identities,
			top_hash: Default::default(),
			parachain_block_number: 0u32,
			sidechain_block_number: 0u32,
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			should_create_id_graph: false,
			req_ext_hash: Default::default(),
		};

		match build(
			&req,
			GenericDiscordRoleType::Contest(ContestType::Legend),
			&data_provider_config,
		) {
			Ok(credential) => {
				log::info!("build GenericDiscordRole Contest done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::Item {
						src: String::from("$is_contest_legend"),
						op: Op::Equal,
						dst: String::from("true")
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build GenericDiscordRole Contest failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_sora_quiz_role_works() {
		let data_provider_config = init();

		let handler_vec: Vec<u8> = "ericzhang.eth".to_string().as_bytes().to_vec();

		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Discord(IdentityString::new(handler_vec)), vec![])];

		let req: AssertionBuildRequest = AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::GenericDiscordRole(GenericDiscordRoleType::SoraQuiz(
				SoraQuizType::Attendee,
			)),
			identities,
			top_hash: Default::default(),
			parachain_block_number: 0u32,
			sidechain_block_number: 0u32,
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			should_create_id_graph: false,
			req_ext_hash: Default::default(),
		};

		match build(
			&req,
			GenericDiscordRoleType::SoraQuiz(SoraQuizType::Attendee),
			&data_provider_config,
		) {
			Ok(credential) => {
				log::info!("build GenericDiscordRole SoraQuiz done");
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), false);
			},
			Err(e) => {
				panic!("build GenericDiscordRole SoraQuiz failed with error {:?}", e);
			},
		}
	}
}
