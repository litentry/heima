// Copyright 2020-2023 Litentry Technologies GmbH.
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

use crate::Result;
use itp_stf_primitives::types::ShardIdentifier;
use itp_types::AccountId;
use lc_credentials::Credential;
use lc_data_providers::graphql::{GraphQLClient, VerifiedCredentialsTotalTxs, VerifiedCredentialsNetwork};
use litentry_primitives::{Assertion, Identity, ParentchainBlockNumber, ASSERTION_NETWORKS, AssertionNetworks, Network, SubstrateNetwork, EvmNetwork};
use log::*;
use parachain_core_primitives::VCMPError;
use std::{str::from_utf8, string::ToString, vec, vec::Vec, collections::HashSet};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NETWORK_HASHSET: HashSet<VerifiedCredentialsNetwork> = {
        let mut m = HashSet::new();
        
		let litentry = VerifiedCredentialsNetwork::from(SubstrateNetwork::Litentry);
		let litmus = VerifiedCredentialsNetwork::from(SubstrateNetwork::Litmus);
		let polkadot = VerifiedCredentialsNetwork::from(SubstrateNetwork::Polkadot);
		let kusama = VerifiedCredentialsNetwork::from(SubstrateNetwork::Kusama);
		let khala = VerifiedCredentialsNetwork::from(SubstrateNetwork::Litentry);
		let ethereum = VerifiedCredentialsNetwork::from(EvmNetwork::Ethereum);

		m.insert(litentry);
		m.insert(litmus);
		m.insert(polkadot);
		m.insert(kusama);
		m.insert(khala);
		m.insert(ethereum);

        m
    };
}

fn assertion_networks_to_vc_networks(networks: AssertionNetworks) -> HashSet<VerifiedCredentialsNetwork> {
	let mut set: HashSet<VerifiedCredentialsNetwork> = HashSet::new();

	if networks.is_empty() { 
		return NETWORK_HASHSET;
	 } else { 
		for network in networks {
			let ret = from_utf8(network.as_ref());
			match ret {
				Ok(network) => {
					let network = network.to_ascii_lowercase().as_str();
					if ASSERTION_NETWORKS.contains(&network) {
						debug!("	[AssertionBuild-A8] available networks: {}", network);

						match network {
							"litentry" => {
								let litentry = VerifiedCredentialsNetwork::from(SubstrateNetwork::Litentry);
								set.insert(litentry);
							},
							"litmus" => {
								let litmus = VerifiedCredentialsNetwork::from(SubstrateNetwork::Litmus);
								set.insert(litmus);
							},
							"polkadot" => {
								let polkadot = VerifiedCredentialsNetwork::from(SubstrateNetwork::Polkadot);
								set.insert(polkadot);
							},
							"kusama" => {
								let kusama = VerifiedCredentialsNetwork::from(SubstrateNetwork::Kusama);
								set.insert(kusama);						
							},
							"khala" => {
								let khala = VerifiedCredentialsNetwork::from(SubstrateNetwork::Litentry);
								set.insert(khala);
							},
							"ethereum" => {
								let ethereum = VerifiedCredentialsNetwork::from(EvmNetwork::Ethereum);
								set.insert(ethereum);
							},
							_ => {
								info!("		[AssertionBuild-A8] Wrong Network!");
							}
						} 
					}
					else {
						continue
					}
				},
				Err(_) => continue
			}
		}

		if set.is_empty() {
			return NETWORK_HASHSET;
		} else {
			return set;
		}
	};
}

fn vc_network_to_vec(networks: HashSet<VerifiedCredentialsNetwork>) -> Vec<&'static str> {
	let mut rets = Vec::<&str>::new();
	for n in networks {
		match n {
			VerifiedCredentialsNetwork::Litentry => {
				rets.push("litentry");
			},
			VerifiedCredentialsNetwork::Litmus => {
				rets.push("litmus");
			},
			VerifiedCredentialsNetwork::Polkadot => {
				rets.push("polkadot");
			},
			VerifiedCredentialsNetwork::Kusama => {
				rets.push("kusama");
			},
			VerifiedCredentialsNetwork::Khala => {
				rets.push("khala");
			},
			VerifiedCredentialsNetwork::Ethereum => {
				rets.push("ethereum");
			},
		}
	}

	rets
}

pub fn build(
	identities: Vec<Identity>,
	networks: AssertionNetworks,
	shard: &ShardIdentifier,
	who: &AccountId,
	bn: ParentchainBlockNumber,
) -> Result<Credential> {
	log::debug!("	[AssertionBuild] A8 networks: {:?}", networks);

	let mut client = GraphQLClient::new();
	let mut total_txs: u64 = 0;
	let target_networks = assertion_networks_to_vc_networks(networks);

	for identity in identities {
		let query = match identity {
			Identity::Substrate { network, address } =>
				if target_networks.contains(&network.into()) {
					from_utf8(address.as_ref()).map_or(None, |addr| {
						Some(VerifiedCredentialsTotalTxs::new(
							vec![addr.to_string()],
							vec![network.into()],
						))
					})
				} else {
					None
				},
			Identity::Evm { network, address } =>
				if target_networks.contains(&network.into()) {
					from_utf8(address.as_ref()).map_or(None, |addr| {
						Some(VerifiedCredentialsTotalTxs::new(
							vec![addr.to_string()],
							vec![network.into()],
						))
					})
				} else {
					None
				}
			_ => {
				debug!("ignore identity: {:?}", identity);
				None
			},
		};
		if let Some(query) = query {
			if let Ok(result) = client.query_total_transactions(query) {
				total_txs += result.iter().map(|v| v.total_transactions).sum::<u64>();
			}
		}
	}	
	debug!("total_transactions: {}", total_txs);

	let min: u64;
	let max: u64;

	match total_txs {
		0 | 1 => {
			min = 0;
			max = 1;
		},
		2..=10 => {
			min = 1;
			max = 10;
		},
		11..=100 => {
			min = 10;
			max = 100;
		},
		101..=1000 => {
			min = 100;
			max = 1000
		},
		1001..=10000 => {
			min = 1000;
			max = 10000;
		},
		10001..=u64::MAX => {
			min = 10000;
			max = u64::MAX;
		},
	}

	let a8 = Assertion::A8(networks);
	match Credential::generate_unsigned_credential(&a8, who, &shard.clone(), bn) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.add_assertion_a8(vc_network_to_vec(target_networks), min, max);
			credential_unsigned.credential_subject.values.push(true);
			return Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
		},
	}

	Err(VCMPError::Assertion8Failed)
}
