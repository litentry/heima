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

use lc_credentials_v2::{nft_holder::NFTHolderAssertionUpdate, Credential, IssuerRuntimeVersion};
use lc_service::web3_nft::nft_holder::has_nft;
use litentry_primitives::{AssertionBuildRequest, Web3Network, Web3NftType};
use log::debug;

use crate::*;

pub fn build(
	req: &AssertionBuildRequest,
	nft_type: Web3NftType,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	debug!("nft holder: {:?}", nft_type);

	let identities: Vec<(Web3Network, Vec<String>)> = transpose_identity(&req.identities);
	let addresses = identities
		.into_iter()
		.flat_map(|(network_type, addresses)| {
			addresses.into_iter().map(move |address| (network_type, address))
		})
		.collect::<Vec<(Web3Network, String)>>();

	let result = has_nft(nft_type.clone(), addresses, data_provider_config).map_err(|e| {
		Error::RequestVCFailed(
			Assertion::NftHolder(nft_type.clone()),
			ErrorDetail::DataProviderError(ErrorString::truncate_from(
				format!("{e:?}").as_bytes().to_vec(),
			)),
		)
	});

	match result {
		Ok(has_nft) => {
			let runtime_version = IssuerRuntimeVersion {
				parachain: req.parachain_runtime_version,
				sidechain: req.sidechain_runtime_version,
			};

			match Credential::new(&req.who, &req.shard, &runtime_version) {
				Ok(mut credential_unsigned) => {
					credential_unsigned.update_nft_holder_assertion(nft_type, has_nft);
					Ok(credential_unsigned)
				},
				Err(e) => {
					error!("Generate unsigned credential failed {:?}", e);
					Err(Error::RequestVCFailed(
						Assertion::NftHolder(nft_type),
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
	use lc_common::web3_nft::{NftAddress, NftName};
	use lc_credentials_v2::assertion_logic::{AssertionLogic, Op};
	use lc_mock_server::run;
	use litentry_hex_utils::decode_hex;
	use litentry_primitives::{Identity, IdentityNetworkTuple};

	fn crate_assertion_build_request(
		nft_type: Web3NftType,
		identities: Vec<IdentityNetworkTuple>,
	) -> AssertionBuildRequest {
		AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::NftHolder(nft_type),
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

	fn create_token_assertion_logic(nft_type: Web3NftType) -> Box<AssertionLogic> {
		Box::new(AssertionLogic::Item {
			src: "$nft".into(),
			op: Op::Equal,
			dst: nft_type.get_nft_name().into(),
		})
	}

	fn create_werido_ghost_gang_assertion_logic() -> Box<AssertionLogic> {
		Box::new(AssertionLogic::Or {
			items: vec![Box::new(AssertionLogic::And {
				items: vec![
					Box::new(AssertionLogic::Item {
						src: "$network".into(),
						op: Op::Equal,
						dst: "ethereum".into(),
					}),
					Box::new(AssertionLogic::Item {
						src: "$address".into(),
						op: Op::Equal,
						dst: Web3NftType::WeirdoGhostGang
							.get_nft_address(Web3Network::Ethereum)
							.unwrap()
							.into(),
					}),
				],
			})],
		})
	}

	fn create_club3_sbt_assertion_logic() -> Box<AssertionLogic> {
		Box::new(AssertionLogic::Or {
			items: vec![
				Box::new(AssertionLogic::And {
					items: vec![
						Box::new(AssertionLogic::Item {
							src: "$network".into(),
							op: Op::Equal,
							dst: "bsc".into(),
						}),
						Box::new(AssertionLogic::Item {
							src: "$address".into(),
							op: Op::Equal,
							dst: Web3NftType::Club3Sbt
								.get_nft_address(Web3Network::Bsc)
								.unwrap()
								.into(),
						}),
					],
				}),
				Box::new(AssertionLogic::And {
					items: vec![
						Box::new(AssertionLogic::Item {
							src: "$network".into(),
							op: Op::Equal,
							dst: "polygon".into(),
						}),
						Box::new(AssertionLogic::Item {
							src: "$address".into(),
							op: Op::Equal,
							dst: Web3NftType::Club3Sbt
								.get_nft_address(Web3Network::Polygon)
								.unwrap()
								.into(),
						}),
					],
				}),
				Box::new(AssertionLogic::And {
					items: vec![
						Box::new(AssertionLogic::Item {
							src: "$network".into(),
							op: Op::Equal,
							dst: "arbitrum".into(),
						}),
						Box::new(AssertionLogic::Item {
							src: "$address".into(),
							op: Op::Equal,
							dst: Web3NftType::Club3Sbt
								.get_nft_address(Web3Network::Arbitrum)
								.unwrap()
								.into(),
						}),
					],
				}),
			],
		})
	}

	fn create_mfan_assertion_logic() -> Box<AssertionLogic> {
		Box::new(AssertionLogic::Or {
			items: vec![Box::new(AssertionLogic::And {
				items: vec![
					Box::new(AssertionLogic::Item {
						src: "$network".into(),
						op: Op::Equal,
						dst: "polygon".into(),
					}),
					Box::new(AssertionLogic::Item {
						src: "$address".into(),
						op: Op::Equal,
						dst: Web3NftType::MFan
							.get_nft_address(Web3Network::Polygon)
							.unwrap()
							.into(),
					}),
				],
			})],
		})
	}

	fn init() -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap();

		let mut data_provider_config = DataProviderConfig::new().unwrap();

		data_provider_config.set_nodereal_api_key("d416f55179dbd0e45b1a8ed030e3".into());
		data_provider_config
			.set_nodereal_api_chain_network_url(url.clone() + "/nodereal_jsonrpc/")
			.unwrap();
		data_provider_config.set_moralis_api_key("d416f55179dbd0e45b1a8ed030e3".into());
		data_provider_config.set_moralis_api_url(url + "/moralis/").unwrap();
		data_provider_config
	}

	#[test]
	fn build_werido_ghost_gang_holder_works() {
		let data_provider_config = init();
		let address = decode_hex("0x45cdb67696802b9d01ed156b883269dbdb9c6239".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];

		let req = crate_assertion_build_request(Web3NftType::WeirdoGhostGang, identities);

		match build(&req, Web3NftType::WeirdoGhostGang, &data_provider_config) {
			Ok(credential) => {
				log::info!("build WeirdoGhostGang holder done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3NftType::WeirdoGhostGang),
							create_werido_ghost_gang_assertion_logic(),
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build WeirdoGhostGang holder failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_club3_sbt_holder_works() {
		let data_provider_config = init();
		let mut address = decode_hex("0x49ad262c49c7aa708cc2df262ed53b64a17dd5ee".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let mut identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Bsc, Web3Network::Polygon])];

		let mut req = crate_assertion_build_request(Web3NftType::Club3Sbt, identities);
		match build(&req, Web3NftType::Club3Sbt, &data_provider_config) {
			Ok(credential) => {
				log::info!("build Club3Sbt holder done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3NftType::Club3Sbt),
							create_club3_sbt_assertion_logic(),
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build Club3Sbt holder failed with error {:?}", e);
			},
		}

		address = decode_hex("0x45cdb67696802b9d01ed156b883269dbdb9c6239".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(
			Identity::Evm(address),
			vec![Web3Network::Bsc, Web3Network::Polygon, Web3Network::Arbitrum],
		)];

		req = crate_assertion_build_request(Web3NftType::Club3Sbt, identities);
		match build(&req, Web3NftType::Club3Sbt, &data_provider_config) {
			Ok(credential) => {
				log::info!("build Club3Sbt holder done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3NftType::Club3Sbt),
							create_club3_sbt_assertion_logic(),
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), false);
			},
			Err(e) => {
				panic!("build Club3Sbt holder failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_mfan_holder_works() {
		let data_provider_config = init();
		let mut address = decode_hex("0x49ad262c49c7aa708cc2df262ed53b64a17dd5ee".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let mut identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Polygon])];

		let mut req = crate_assertion_build_request(Web3NftType::MFan, identities);
		match build(&req, Web3NftType::MFan, &data_provider_config) {
			Ok(credential) => {
				log::info!("build MFan holder done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3NftType::MFan),
							create_mfan_assertion_logic(),
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build MFan holder failed with error {:?}", e);
			},
		}

		address = decode_hex("0x45cdb67696802b9d01ed156b883269dbdb9c6239".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(Identity::Evm(address), vec![Web3Network::Polygon])];

		req = crate_assertion_build_request(Web3NftType::MFan, identities);
		match build(&req, Web3NftType::MFan, &data_provider_config) {
			Ok(credential) => {
				log::info!("build MFan holder done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3NftType::MFan),
							create_mfan_assertion_logic(),
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), false);
			},
			Err(e) => {
				panic!("build MFan holder failed with error {:?}", e);
			},
		}
	}
}
