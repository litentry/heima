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

use core::result;

use crate::*;
use lc_common::abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls};
use lc_credentials::{
	nodereal::amount_holding::evm_amount_holding::{
		EVMAmountHoldingAssertionUpdate, EVMTokenAddress,
	},
	Credential, IssuerRuntimeVersion,
};
use lc_data_providers::{
	nodereal_jsonrpc::{
		FungibleApiList, GetTokenBalance20Param, NoderealChain, NoderealJsonrpcClient,
	},
	DataProviderConfig, Error as DataProviderError,
};
use litentry_primitives::EVMTokenType;

fn get_holding_balance(
	token_type: EVMTokenType,
	addresses: Vec<(Web3Network, String)>,
	data_provider_config: &DataProviderConfig,
) -> result::Result<f64, DataProviderError> {
	let mut eth_client = NoderealJsonrpcClient::new(NoderealChain::Eth, data_provider_config);
	let mut bsc_client = NoderealJsonrpcClient::new(NoderealChain::Bsc, data_provider_config);
	let mut total_balance = 0_u128;

	let decimals = token_type.token_decimals();

	loop_with_abort_strategy(
		addresses,
		|(network, address)| {
			let param = GetTokenBalance20Param {
				contract_address: token_type.get_address(*network).unwrap_or_default().into(),
				address: address.clone(),
				block_number: "latest".into(),
			};
			match network {
				Web3Network::Bsc => match bsc_client.get_token_balance_20(&param, false) {
					Ok(balance) => {
						total_balance += balance;
						Ok(LoopControls::Continue)
					},
					Err(err) => Err(err),
				},
				Web3Network::Ethereum => match eth_client.get_token_balance_20(&param, false) {
					Ok(balance) => {
						total_balance += balance;
						Ok(LoopControls::Continue)
					},
					Err(err) => Err(err),
				},
				_ => Ok(LoopControls::Continue),
			}
		},
		AbortStrategy::FailFast::<fn(&_) -> bool>,
	)
	.map_err(|errors| errors[0].clone())?;

	Ok((total_balance / decimals as u128) as f64
		+ ((total_balance % decimals as u128) as f64 / decimals))
}

pub fn build(
	req: &AssertionBuildRequest,
	token_type: EVMTokenType,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	debug!("evm amount holding: {:?}", token_type);

	let identities: Vec<(Web3Network, Vec<String>)> = transpose_identity(&req.identities);
	let addresses = identities
		.into_iter()
		.filter(|(newtwork_type, _)| newtwork_type.is_evm())
		.flat_map(|(newtwork_type, addresses)| {
			addresses.into_iter().map(move |address| (newtwork_type, address))
		})
		.collect::<Vec<(Web3Network, String)>>();

	let result =
		get_holding_balance(token_type.clone(), addresses, data_provider_config).map_err(|e| {
			Error::RequestVCFailed(
				Assertion::EVMAmountHolding(token_type.clone()),
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
					credential_unsigned.update_evm_amount_holding_assertion(token_type, value);
					Ok(credential_unsigned)
				},
				Err(e) => {
					error!("Generate unsigned credential failed {:?}", e);
					Err(Error::RequestVCFailed(
						Assertion::EVMAmountHolding(token_type),
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
	use lc_credentials::assertion_logic::{AssertionLogic, Op};
	use lc_mock_server::run;
	use litentry_primitives::decode_hex;

	fn create_ton_token_assertion_logic() -> Box<AssertionLogic> {
		Box::new(AssertionLogic::Item { src: "$token".into(), op: Op::Equal, dst: "TON".into() })
	}

	fn create_ton_network_assertion_logic() -> Box<AssertionLogic> {
		Box::new(AssertionLogic::Or {
			items: vec![
				Box::new(AssertionLogic::And {
					items: vec![
						Box::new(AssertionLogic::Item {
							src: "$network".into(),
							op: Op::Equal,
							dst: "ethereum".into(),
						}),
						Box::new(AssertionLogic::Item {
							src: "$address".into(),
							op: Op::Equal,
							dst: "0x582d872a1b094fc48f5de31d3b73f2d9be47def1".into(),
						}),
					],
				}),
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
							dst: "0x76a797a59ba2c17726896976b7b3747bfd1d220f".into(),
						}),
					],
				}),
			],
		})
	}

	fn init() -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap() + "/nodereal_jsonrpc/";
		let mut data_provider_config = DataProviderConfig::new().unwrap();

		data_provider_config.set_nodereal_api_key("d416f55179dbd0e45b1a8ed030e3".into());
		data_provider_config.set_nodereal_api_chain_network_url(url).unwrap();
		data_provider_config
	}

	#[test]
	fn build_evm_amount_holding_works() {
		let data_provider_config = init();
		let identities: Vec<IdentityNetworkTuple> = vec![
			(Identity::Evm([0; 20].into()), vec![Web3Network::Ethereum]),
			(Identity::Evm([0; 20].into()), vec![Web3Network::Ethereum, Web3Network::Bsc]),
		];

		let req: AssertionBuildRequest = AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::EVMAmountHolding(EVMTokenType::Ton),
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

		match build(&req, EVMTokenType::Ton, &data_provider_config) {
			Ok(credential) => {
				log::info!("build EVMAmount holding done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_ton_token_assertion_logic(),
							create_ton_network_assertion_logic(),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterThan,
								dst: "0".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "1".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build EVMAmount holding failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_evm_amount_holding_lt_min_works() {
		let data_provider_config = init();
		let address = decode_hex("0x85be4e2ccc9c85be8783798b6e8a101bdac6467f".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];

		let req: AssertionBuildRequest = AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::EVMAmountHolding(EVMTokenType::Ton),
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

		match build(&req, EVMTokenType::Ton, &data_provider_config) {
			Ok(credential) => {
				log::info!("build EVMAmount holding done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_ton_token_assertion_logic(),
							create_ton_network_assertion_logic(),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "100".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "200".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build EVMAmount holding failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_evm_amount_holding_gte_max_works() {
		let data_provider_config = init();
		let address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e3c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];

		let req: AssertionBuildRequest = AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::EVMAmountHolding(EVMTokenType::Ton),
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

		match build(&req, EVMTokenType::Ton, &data_provider_config) {
			Ok(credential) => {
				log::info!("build EVMAmount holding done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_ton_token_assertion_logic(),
							create_ton_network_assertion_logic(),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "3000".into()
							}),
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build EVMAmount holding failed with error {:?}", e);
			},
		}
	}
}
