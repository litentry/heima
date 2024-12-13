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

use lc_credentials_v2::{
	token_holding_amount::TokenHoldingAmountAssertionUpdate, Credential, IssuerRuntimeVersion,
};
use lc_service::web3_token::token_balance::get_token_balance;
use litentry_primitives::{AssertionBuildRequest, Web3Network, Web3TokenType};
use log::debug;

use crate::*;

pub fn build(
	req: &AssertionBuildRequest,
	token_type: Web3TokenType,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	debug!("token holding amount: {:?}", token_type);

	let identities: Vec<(Web3Network, Vec<String>)> = transpose_identity(&req.identities);
	let addresses = identities
		.into_iter()
		.flat_map(|(network_type, addresses)| {
			addresses.into_iter().map(move |address| (network_type, address))
		})
		.collect::<Vec<(Web3Network, String)>>();

	let result =
		get_token_balance(token_type.clone(), addresses, data_provider_config).map_err(|e| {
			Error::RequestVCFailed(Assertion::TokenHoldingAmount(token_type.clone()), e)
		})?;

	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};

	match Credential::new(&req.who, &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.update_token_holding_amount_assertion(token_type, result);
			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(
				Assertion::TokenHoldingAmount(token_type),
				e.into_error_detail(),
			))
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use base58::FromBase58;
	use itp_stf_primitives::types::ShardIdentifier;
	use itp_types::AccountId;
	use lc_common::{
		web3_network_to_chain,
		web3_token::{TokenAddress, TokenName},
	};
	use lc_credentials_v2::assertion_logic::{AssertionLogic, Op};
	use lc_mock_server::run;
	use litentry_hex_utils::decode_hex;
	use litentry_primitives::{Identity, IdentityNetworkTuple};

	fn crate_assertion_build_request(
		token_type: Web3TokenType,
		identities: Vec<IdentityNetworkTuple>,
	) -> AssertionBuildRequest {
		AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::TokenHoldingAmount(token_type),
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

	fn create_token_assertion_logic(token_type: Web3TokenType) -> Box<AssertionLogic> {
		Box::new(AssertionLogic::Item {
			src: "$token".into(),
			op: Op::Equal,
			dst: token_type.get_token_name().into(),
		})
	}

	fn create_network_address_assertion_logic(
		token_type: Web3TokenType,
		network: Web3Network,
	) -> AssertionLogic {
		let mut assertion = AssertionLogic::new_and();
		assertion = assertion.add_item(AssertionLogic::new_item(
			"$network",
			Op::Equal,
			web3_network_to_chain(&network),
		));
		if let Some(address) = token_type.get_token_address(network) {
			assertion =
				assertion.add_item(AssertionLogic::new_item("$address", Op::Equal, address));
		}
		assertion
	}

	fn create_network_address_assertion_logics(token_type: Web3TokenType) -> Box<AssertionLogic> {
		let assertion_logic = token_type.get_supported_networks().into_iter().fold(
			AssertionLogic::new_or(),
			|assertion, network| {
				assertion
					.add_item(create_network_address_assertion_logic(token_type.clone(), network))
			},
		);
		Box::new(assertion_logic)
	}

	fn init() -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap();
		let mut data_provider_config = DataProviderConfig::new().unwrap();

		data_provider_config.set_nodereal_api_key("d416f55179dbd0e45b1a8ed030e3".into());
		data_provider_config.set_moralis_api_key("d416f55179dbd0e45b1a8ed030e2".into());
		data_provider_config
			.set_nodereal_api_chain_network_url(url.clone() + "/nodereal_jsonrpc/")
			.unwrap();
		data_provider_config
			.set_blockchain_info_api_url(url.clone() + "/blockchain_info/")
			.unwrap();
		data_provider_config.set_achainable_url(url.clone()).unwrap();
		data_provider_config.set_moralis_api_url(url.clone() + "/moralis/").unwrap();
		data_provider_config
			.set_moralis_solana_api_url(url + "/moralis_solana/")
			.unwrap();
		data_provider_config
	}

	#[test]
	fn build_bnb_holding_amount_works() {
		let data_provider_config = init();
		let address = decode_hex("0x45cdb67696802b9d01ed156b883269dbdb9c6239".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Bsc, Web3Network::Ethereum])];

		let req = crate_assertion_build_request(Web3TokenType::Bnb, identities);

		match build(&req, Web3TokenType::Bnb, &data_provider_config) {
			Ok(credential) => {
				log::info!("build bnb TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Bnb),
							create_network_address_assertion_logics(Web3TokenType::Bnb),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "50".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "100".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build bnb TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_eth_holding_amount_works() {
		let data_provider_config = init();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm([0; 20].into()), vec![Web3Network::Ethereum])];

		let req = crate_assertion_build_request(Web3TokenType::Eth, identities);

		match build(&req, Web3TokenType::Eth, &data_provider_config) {
			Ok(credential) => {
				log::info!("build eth TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Eth),
							create_network_address_assertion_logics(Web3TokenType::Eth),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "0.6".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "1.2".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build eth TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_evm_holding_amount_works() {
		let data_provider_config = init();
		let address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e7c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Bsc, Web3Network::Ethereum])];

		let req = crate_assertion_build_request(Web3TokenType::SpaceId, identities);

		match build(&req, Web3TokenType::SpaceId, &data_provider_config) {
			Ok(credential) => {
				log::info!("build evm TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::SpaceId),
							create_network_address_assertion_logics(Web3TokenType::SpaceId),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "800".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "1200".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build evm TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_ethereum_holding_amount_works() {
		let data_provider_config = init();
		let address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e7c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];

		let req = crate_assertion_build_request(Web3TokenType::Amp, identities);

		match build(&req, Web3TokenType::Amp, &data_provider_config) {
			Ok(credential) => {
				log::info!("build ethereum TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Amp),
							create_network_address_assertion_logics(Web3TokenType::Amp),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "200".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "500".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build ethereum TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_lit_holding_amount_works() {
		let data_provider_config = init();
		let address = decode_hex("0xba359c153ad11aa17c3122b05a4db8b46bb3191b".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Ethereum, Web3Network::Litentry])];

		let req = crate_assertion_build_request(Web3TokenType::Lit, identities);

		match build(&req, Web3TokenType::Lit, &data_provider_config) {
			Ok(credential) => {
				log::info!("build lit TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Lit),
							create_network_address_assertion_logics(Web3TokenType::Lit),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "1600".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "3000".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build lit TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_sol_holding_amount_works() {
		let data_provider_config = init();
		let address1 = "EJpLyTeE8XHG9CeREeHd6pr6hNhaRnTRJx4Z5DPhEJJ6"
			.from_base58()
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let address2 = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e0c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> = vec![
			(Identity::Solana(address1), vec![Web3Network::Solana]),
			(Identity::Evm(address2), vec![Web3Network::Ethereum]),
		];

		let req = crate_assertion_build_request(Web3TokenType::Sol, identities);

		match build(&req, Web3TokenType::Sol, &data_provider_config) {
			Ok(credential) => {
				log::info!("build sol TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Sol),
							create_network_address_assertion_logics(Web3TokenType::Sol),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "1".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "50".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build sol TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_nfp_holding_amount_works() {
		let data_provider_config = init();
		let mut address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e0c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let mut identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Evm(address), vec![Web3Network::Bsc])];
		let mut req = crate_assertion_build_request(Web3TokenType::Nfp, identities);
		match build(&req, Web3TokenType::Nfp, &data_provider_config) {
			Ok(credential) => {
				log::info!("build nfp TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Nfp),
							create_network_address_assertion_logics(Web3TokenType::Nfp),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterThan,
								dst: "0".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), false);
			},
			Err(e) => {
				panic!("build nfp TokenHoldingAmount failed with error {:?}", e);
			},
		};

		address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e1c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc])];
		req = crate_assertion_build_request(Web3TokenType::Nfp, identities);
		match build(&req, Web3TokenType::Nfp, &data_provider_config) {
			Ok(credential) => {
				log::info!("build nfp TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Nfp),
							create_network_address_assertion_logics(Web3TokenType::Nfp),
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
				panic!("build nfp TokenHoldingAmount failed with error {:?}", e);
			},
		};

		address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e2c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc])];
		req = crate_assertion_build_request(Web3TokenType::Nfp, identities);
		match build(&req, Web3TokenType::Nfp, &data_provider_config) {
			Ok(credential) => {
				log::info!("build nfp TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Nfp),
							create_network_address_assertion_logics(Web3TokenType::Nfp),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "1".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "50".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build nfp TokenHoldingAmount failed with error {:?}", e);
			},
		};

		address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e3c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc])];
		req = crate_assertion_build_request(Web3TokenType::Nfp, identities);
		match build(&req, Web3TokenType::Nfp, &data_provider_config) {
			Ok(credential) => {
				log::info!("build nfp TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Nfp),
							create_network_address_assertion_logics(Web3TokenType::Nfp),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "3000".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build nfp TokenHoldingAmount failed with error {:?}", e);
			},
		};
	}

	#[test]
	fn build_mcrt_holding_amount_works() {
		let data_provider_config = init();
		let address = "EJpLyTeE8XHG9CeREeHd6pr6hNhaRnTRJx4Z5DPhEJJ6"
			.from_base58()
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Solana(address), vec![Web3Network::Solana])];

		let req = crate_assertion_build_request(Web3TokenType::Mcrt, identities);

		match build(&req, Web3TokenType::Mcrt, &data_provider_config) {
			Ok(credential) => {
				log::info!("build mcrt TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Mcrt),
							create_network_address_assertion_logics(Web3TokenType::Mcrt),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "150000".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "500000".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build mcrt TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_btc_holding_amount_works() {
		let data_provider_config: DataProviderConfig = init();
		// bc1pgr5fw4p9gl9me0vzjklnlnap669caxc0gsk4j62gff2qktlw6naqm4m3d0
		let address = decode_hex(
			"0x02e8c39e82aaaa143c3def8d3c7084a539b227244ac9067c3f7fc86cb73a0b7aed".as_bytes(),
		)
		.unwrap()
		.as_slice()
		.try_into()
		.unwrap();
		let identities: Vec<IdentityNetworkTuple> =
			vec![(Identity::Bitcoin(address), vec![Web3Network::BitcoinP2tr])];

		let req = crate_assertion_build_request(Web3TokenType::Btc, identities);

		match build(&req, Web3TokenType::Btc, &data_provider_config) {
			Ok(credential) => {
				log::info!("build btc TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Btc),
							create_network_address_assertion_logics(Web3TokenType::Btc),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "50".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build btc TokenHoldingAmount failed with error {:?}", e);
			},
		}
	}

	#[test]
	fn build_bean_holding_amount_works() {
		let data_provider_config: DataProviderConfig = init();
		let address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e3c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc])];
		let req = crate_assertion_build_request(Web3TokenType::Bean, identities);
		match build(&req, Web3TokenType::Bean, &data_provider_config) {
			Ok(credential) => {
				log::info!("build bean TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Bean),
							create_network_address_assertion_logics(Web3TokenType::Bean),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "5000".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "10000".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build bean TokenHoldingAmount failed with error {:?}", e);
			},
		};
	}

	#[test]
	fn build_an_holding_amount_works() {
		let data_provider_config: DataProviderConfig = init();
		let address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e4c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities = vec![(Identity::Evm(address), vec![Web3Network::Bsc])];
		let req = crate_assertion_build_request(Web3TokenType::An, identities);
		match build(&req, Web3TokenType::An, &data_provider_config) {
			Ok(credential) => {
				log::info!("build an TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::An),
							create_network_address_assertion_logics(Web3TokenType::An),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "100000".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "900000".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build an TokenHoldingAmount failed with error {:?}", e);
			},
		};
	}

	#[test]
	fn build_tuna_holding_amount_works() {
		let data_provider_config: DataProviderConfig = init();
		let address = decode_hex("0x75438d34c9125839c8b08d21b7f3167281659e5c".as_bytes())
			.unwrap()
			.as_slice()
			.try_into()
			.unwrap();
		let identities = vec![(Identity::Evm(address), vec![Web3Network::Ethereum])];
		let req = crate_assertion_build_request(Web3TokenType::Tuna, identities);
		match build(&req, Web3TokenType::Tuna, &data_provider_config) {
			Ok(credential) => {
				log::info!("build tuna TokenHoldingAmount done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![
							create_token_assertion_logic(Web3TokenType::Tuna),
							create_network_address_assertion_logics(Web3TokenType::Tuna),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::GreaterEq,
								dst: "1000".into()
							}),
							Box::new(AssertionLogic::Item {
								src: "$holding_amount".into(),
								op: Op::LessThan,
								dst: "2000".into()
							})
						]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build tuna TokenHoldingAmount failed with error {:?}", e);
			},
		};
	}
}
