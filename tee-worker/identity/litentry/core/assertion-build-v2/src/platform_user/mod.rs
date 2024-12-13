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
// along with Litentry. If not, see <https://www.gnu.org/licenses/>.

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use std::collections::HashSet;

use lc_credentials_v2::{
	platform_user::PlatformUserAssertionUpdate, Credential, IssuerRuntimeVersion,
};
use lc_service::platform_user::is_user;
use litentry_primitives::{AssertionBuildRequest, PlatformUserType, Web3Network};
use log::debug;

use crate::*;

pub fn build(
	req: &AssertionBuildRequest,
	platform_user_type: PlatformUserType,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	debug!("platform user: {:?}", platform_user_type);

	let identities: Vec<(Web3Network, Vec<String>)> = transpose_identity(&req.identities);
	let addresses: Vec<String> = identities
		.into_iter()
		.flat_map(|(_, addresses)| addresses.into_iter())
		.collect::<HashSet<String>>()
		.into_iter()
		.collect();

	let result =
		is_user(platform_user_type.clone(), addresses, data_provider_config).map_err(|e| {
			Error::RequestVCFailed(
				Assertion::PlatformUser(platform_user_type.clone()),
				ErrorDetail::DataProviderError(ErrorString::truncate_from(
					format!("{e:?}").as_bytes().to_vec(),
				)),
			)
		});

	match result {
		Ok(value) => {
			let runtime_version = IssuerRuntimeVersion {
				parachain: req.parachain_runtime_version,
				sidechain: req.sidechain_runtime_version,
			};

			match Credential::new(&req.who, &req.shard, &runtime_version) {
				Ok(mut credential_unsigned) => {
					credential_unsigned.update_platform_user_assertion(platform_user_type, value);
					Ok(credential_unsigned)
				},
				Err(e) => {
					error!("Generate unsigned credential failed {:?}", e);
					Err(Error::RequestVCFailed(
						Assertion::PlatformUser(platform_user_type),
						e.into_error_detail(),
					))
				},
			}
		},
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use itp_stf_primitives::types::ShardIdentifier;
	use itp_types::AccountId;
	use lc_common::platform_user::PlatformName;
	use lc_credentials_v2::assertion_logic::{AssertionLogic, Op};
	use lc_mock_server::run;
	use litentry_hex_utils::decode_hex;
	use litentry_primitives::{Identity, IdentityNetworkTuple};

	fn init(platform_user_type: PlatformUserType) -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();

		let mut config = DataProviderConfig::new().unwrap();

		match platform_user_type {
			PlatformUserType::KaratDao => {
				let url = run(0).unwrap() + "/karat_dao/";
				config.set_karat_dao_api_url(url).unwrap();
			},
			PlatformUserType::MagicCraftStaking => {
				let url = run(0).unwrap() + "/magic_craft/";
				config.set_magic_craft_api_url(url).unwrap();
			},
			PlatformUserType::DarenMarket => {
				let url = run(0).unwrap() + "/daren_market/";
				config.set_daren_market_api_url(url).unwrap();
			},
		};

		config
	}

	fn crate_assertion_build_request(
		platform_user_type: PlatformUserType,
		identities: Vec<IdentityNetworkTuple>,
	) -> AssertionBuildRequest {
		AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::PlatformUser(platform_user_type),
			identities,
			top_hash: Default::default(),
			parachain_block_number: 0u32,
			sidechain_block_number: 0u32,
			parachain_runtime_version: 0u32,
			sidechain_runtime_version: 0u32,
			maybe_key: None,
			should_create_id_graph: false,
			req_ext_hash: Default::default(),
		}
	}

	fn build_and_assert_result(
		identities: Vec<IdentityNetworkTuple>,
		platform_user_type: PlatformUserType,
		assertion_value: bool,
		data_provider_config: &DataProviderConfig,
	) {
		let req = crate_assertion_build_request(platform_user_type.clone(), identities);

		match build(&req, platform_user_type.clone(), data_provider_config) {
			Ok(credential) => {
				log::info!("build platform user: {:?} done", platform_user_type);
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![Box::new(AssertionLogic::Item {
							src: "$platform".into(),
							op: Op::Equal,
							dst: platform_user_type.get_platform_name().into()
						})]
					}
				);
				assert_eq!(
					*(credential.credential_subject.values.first().unwrap()),
					assertion_value
				);
			},
			Err(e) => {
				panic!("build platform user: {:?} failed with error {:?}", platform_user_type, e);
			},
		}
	}

	#[test]
	fn build_karat_dao_user_works() {
		let data_provider_config = init(PlatformUserType::KaratDao);

		let mut address = decode_hex("0x49ad262c49c7aa708cc2df262ed53b64a17dd5ee".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let mut identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];

		build_and_assert_result(
			identities,
			PlatformUserType::KaratDao,
			true,
			&data_provider_config,
		);

		address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e7c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc, Web3Network::Ethereum])];

		build_and_assert_result(
			identities,
			PlatformUserType::KaratDao,
			false,
			&data_provider_config,
		);
	}

	#[test]
	fn build_magic_craft_staking_user_works() {
		let data_provider_config = init(PlatformUserType::MagicCraftStaking);

		let mut address = decode_hex("0x49ad262c49c7aa708cc2df262ed53b64a17dd5ee".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let mut identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];

		build_and_assert_result(
			identities,
			PlatformUserType::MagicCraftStaking,
			true,
			&data_provider_config,
		);

		address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e7c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc, Web3Network::Ethereum])];

		build_and_assert_result(
			identities,
			PlatformUserType::MagicCraftStaking,
			false,
			&data_provider_config,
		);
	}

	#[test]
	fn build_daren_market_user_works() {
		let data_provider_config = init(PlatformUserType::DarenMarket);

		let mut address = decode_hex("0x49ad262c49c7aa708cc2df262ed53b64a17dd5ee".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let mut identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];

		build_and_assert_result(
			identities,
			PlatformUserType::DarenMarket,
			true,
			&data_provider_config,
		);

		address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e7c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc, Web3Network::Ethereum])];

		build_and_assert_result(
			identities,
			PlatformUserType::DarenMarket,
			false,
			&data_provider_config,
		);
	}
}
