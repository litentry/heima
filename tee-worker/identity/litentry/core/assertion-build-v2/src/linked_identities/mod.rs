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
	linked_identities::LinkedIdentitiesAssertionUpdate, Credential, IssuerRuntimeVersion,
};
use litentry_primitives::AssertionBuildRequest;

use crate::*;

pub fn build(req: &AssertionBuildRequest) -> Result<Credential> {
	let identities = req
		.identities
		.iter()
		.filter_map(|identity| identity.0.to_did().ok())
		.collect::<Vec<_>>();

	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};

	match Credential::new(&req.who, &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.update_linked_identities_assertion(identities);
			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(Assertion::LinkedIdentities, e.into_error_detail()))
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use itp_stf_primitives::types::ShardIdentifier;
	use itp_types::AccountId;
	use lc_credentials_v2::assertion_logic::{AssertionLogic, Op};
	use litentry_primitives::{Identity, IdentityNetworkTuple, IdentityString};

	#[test]
	fn build_linked_identities_works() {
		let mut identities: Vec<IdentityNetworkTuple> = vec![
			(Identity::Substrate([0; 32].into()), vec![]),
			(Identity::Evm([0; 20].into()), vec![]),
			(Identity::Bitcoin([0; 33].into()), vec![]),
			(Identity::Solana([0; 32].into()), vec![]),
			(Identity::Discord(IdentityString::new("discord_handle".as_bytes().to_vec())), vec![]),
			(Identity::Twitter(IdentityString::new("twitter_handle".as_bytes().to_vec())), vec![]),
			(Identity::Github(IdentityString::new("github_handle".as_bytes().to_vec())), vec![]),
		];

		let req = AssertionBuildRequest {
			shard: ShardIdentifier::default(),
			signer: AccountId::from([0; 32]),
			who: AccountId::from([0; 32]).into(),
			assertion: Assertion::LinkedIdentities,
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

		match build(&req) {
			Ok(credential) => {
				log::info!("build linked_identities done");
				assert_eq!(
					*(credential.credential_subject.assertions.first().unwrap()),
					AssertionLogic::And {
						items: vec![Box::new(AssertionLogic::Item {
							src: "$identities".into(),
							op: Op::Equal,
							dst: "[\"did:litentry:substrate:0x0000000000000000000000000000000000000000000000000000000000000000\",\"did:litentry:evm:0x0000000000000000000000000000000000000000\",\"did:litentry:bitcoin:0x000000000000000000000000000000000000000000000000000000000000000000\",\"did:litentry:solana:11111111111111111111111111111111\",\"did:litentry:discord:discord_handle\",\"did:litentry:twitter:twitter_handle\",\"did:litentry:github:github_handle\"]".into()
						})]
					}
				);
				assert_eq!(*(credential.credential_subject.values.first().unwrap()), true);
			},
			Err(e) => {
				panic!("build linked_identities failed with error {:?}", e);
			},
		}
	}
}
