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

use crate::{BlockNumberOf, Config, IDGraph, Web3Network};
use codec::{Decode, Encode};
use core::cmp::Ordering;
use litentry_primitives::{Identity, IdentityNetworkTuple};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(Clone, Eq, PartialEq, Default, Debug, Encode, Decode, TypeInfo)]
pub enum IdentityStatus {
	#[default]
	#[codec(index = 0)]
	Active,
	#[codec(index = 1)]
	Inactive,
}

// The context associated with the (litentry-account, did) pair
#[derive(Clone, Eq, PartialEq, Default, Debug, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct IdentityContext<T: Config> {
	// the sidechain block number at which the identity is linked
	pub link_block: BlockNumberOf<T>,
	// a list of web3 networks on which the identity should be used
	pub web3networks: Vec<Web3Network>,
	// the identity status
	pub status: IdentityStatus,
}

impl<T: Config> IdentityContext<T> {
	pub fn new(link_block: BlockNumberOf<T>, web3networks: Vec<Web3Network>) -> Self {
		Self { link_block, web3networks: Self::dedup(web3networks), status: IdentityStatus::Active }
	}

	pub fn set_web3networks(&mut self, web3networks: Vec<Web3Network>) {
		self.web3networks = Self::dedup(web3networks);
	}

	pub fn deactivate(&mut self) {
		self.status = IdentityStatus::Inactive
	}

	pub fn activate(&mut self) {
		self.status = IdentityStatus::Active
	}

	pub fn is_active(&self) -> bool {
		self.status == IdentityStatus::Active
	}

	// a small helper fn to apply mutable changes
	fn dedup(mut web3networks: Vec<Web3Network>) -> Vec<Web3Network> {
		web3networks.sort();
		web3networks.dedup();
		web3networks
	}
}

pub fn sort_id_graph<T: Config>(id_graph: &mut [(Identity, IdentityContext<T>)]) {
	id_graph.sort_by(|a, b| {
		let order = Ord::cmp(&a.1.link_block, &b.1.link_block);
		if order == Ordering::Equal {
			// Compare identities by their did formated string
			Ord::cmp(&a.0.to_did().ok(), &b.0.to_did().ok())
		} else {
			order
		}
	});
}

// get the active identities in the `id_graph` whose web3networks match the `desired_web3networks`,
// return a `Vec<(Identity, Vec<Web3Network>)` with retained web3networks
//
// if `skip_filtering` is true, the **active** identities will be passed through regardless of the value
// of `desired_web3networks`, which basically let assertion logic itself to handle those identities.
#[allow(clippy::collapsible_else_if)]
pub fn get_eligible_identities<T: Config>(
	id_graph: &IDGraph<T>,
	desired_web3networks: Vec<Web3Network>,
	skip_filtering: bool,
) -> Vec<IdentityNetworkTuple> {
	id_graph
		.iter()
		.filter_map(|item| {
			if item.1.is_active() {
				let mut networks = item.0.default_web3networks();

				if skip_filtering {
					return Some((item.0.clone(), networks))
				}
				// filter out identities whose web3networks are not supported by this specific `assertion`.
				// We do it here before every request sending because:
				// - it's a common step for all assertion buildings, for those assertions which only
				//   care about web2 identities, this step will empty `IdentityContext.web3networks`
				// - it helps to reduce the request size a bit
				networks.retain(|n| desired_web3networks.contains(n));

				// differentiate between web2 and web3 assertions:
				// desired_web3networks.is_empty() means it's a web2 assertion,
				// otherwise web2 identities might survive to be unexpectedly "eligible" for web3 assertions.
				if desired_web3networks.is_empty() {
					if item.0.is_web2() {
						Some((item.0.clone(), networks))
					} else {
						None
					}
				} else {
					if item.0.is_web3() && !networks.is_empty() {
						Some((item.0.clone(), networks))
					} else {
						None
					}
				}
			} else {
				None
			}
		})
		.collect()
}
