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

use lc_data_providers::achainable::web3_network_to_chain;
use litentry_primitives::{all_evm_web3networks, EVMTokenType, Web3Network};

use crate::{
	assertion_logic::{AssertionLogic, Op},
	litentry_profile::{BalanceRange, BalanceRangeIndex},
	Credential,
};
pub trait EVMTokenAddress {
	fn get_address(&self, network: Web3Network) -> Option<&'static str>;
}

impl EVMTokenAddress for EVMTokenType {
	fn get_address(&self, network: Web3Network) -> Option<&'static str> {
		match (self, network) {
			(EVMTokenType::Ton, Web3Network::Bsc) => {
				Some("0x76a797a59ba2c17726896976b7b3747bfd1d220f")
			},
			(EVMTokenType::Ton, Web3Network::Ethereum) => {
				Some("0x582d872a1b094fc48f5de31d3b73f2d9be47def1")
			},
			(EVMTokenType::Trx, Web3Network::Bsc) => {
				Some("0xCE7de646e7208a4Ef112cb6ed5038FA6cC6b12e3")
			},
			(EVMTokenType::Trx, Web3Network::Ethereum) => {
				Some("0x50327c6c5a14dcade707abad2e27eb517df87ab5")
			},
			_ => None,
		}
	}
}

const EVM_HOLDING_AMOUNT_RANGE: [f64; 10] =
	[0.0, 1.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0, 1600.0, 3000.0];

const TYPE: &str = "Token Holding Amount";
const DESCRIPTION: &str = "The amount of a particular token you are holding";

struct AssertionKeys {
	token: &'static str,
	network: &'static str,
	address: &'static str,
	holding_amount: &'static str,
}

const ASSERTION_KEYS: AssertionKeys = AssertionKeys {
	token: "$token",
	network: "$network",
	address: "$address",
	holding_amount: "$holding_amount",
};

trait AssertionTokenName {
	fn get_name(&self) -> &'static str;
}

impl AssertionTokenName for EVMTokenType {
	fn get_name(&self) -> &'static str {
		match self {
			EVMTokenType::Ton => "TON",
			EVMTokenType::Trx => "TRX",
		}
	}
}

pub trait EVMAmountHoldingAssertionUpdate {
	fn update_evm_amount_holding_assertion(&mut self, token_type: EVMTokenType, amount: f64);
}

impl EVMAmountHoldingAssertionUpdate for Credential {
	fn update_evm_amount_holding_assertion(&mut self, token_type: EVMTokenType, amount: f64) {
		self.add_subject_info(DESCRIPTION, TYPE);

		update_assertion(token_type, amount, self);
	}
}

fn update_assertion(token_type: EVMTokenType, balance: f64, credential: &mut Credential) {
	let mut assertion = AssertionLogic::new_and();

	assertion = assertion.add_item(AssertionLogic::new_item(
		ASSERTION_KEYS.token,
		Op::Equal,
		token_type.get_name(),
	));

	let mut network_assertion = AssertionLogic::new_or();
	for newtork in all_evm_web3networks() {
		match create_network_assertion_logic(newtork, token_type.clone()) {
			Some(network_assertion_item) => {
				network_assertion = network_assertion.add_item(network_assertion_item);
			},
			None => continue,
		}
	}

	assertion = assertion.add_item(network_assertion);

	let index = BalanceRange::index(&EVM_HOLDING_AMOUNT_RANGE, balance);
	match index {
		Some(index) => {
			let min = format!("{}", &EVM_HOLDING_AMOUNT_RANGE[index]);
			let max = format!("{}", &EVM_HOLDING_AMOUNT_RANGE[index + 1]);
			let min_item = AssertionLogic::new_item(
				ASSERTION_KEYS.holding_amount,
				if index == 0 { Op::GreaterThan } else { Op::GreaterEq },
				&min,
			);

			assertion = assertion.add_item(min_item);
			if balance > 0_f64 {
				let max_item =
					AssertionLogic::new_item(ASSERTION_KEYS.holding_amount, Op::LessThan, &max);
				assertion = assertion.add_item(max_item);
			}

			credential.credential_subject.values.push(index != 0 || balance > 0_f64);
		},
		None => {
			let min_item = AssertionLogic::new_item(
				ASSERTION_KEYS.holding_amount,
				Op::GreaterEq,
				&format!("{}", &EVM_HOLDING_AMOUNT_RANGE.last().unwrap()),
			);
			assertion = assertion.add_item(min_item);

			credential.credential_subject.values.push(true);
		},
	}

	credential.credential_subject.assertions.push(assertion);
}

fn create_network_assertion_logic(
	network: Web3Network,
	token_type: EVMTokenType,
) -> Option<AssertionLogic> {
	let mut assertion = AssertionLogic::new_and();
	assertion = assertion.add_item(AssertionLogic::new_item(
		ASSERTION_KEYS.network,
		Op::Equal,
		web3_network_to_chain(&network).as_str(),
	));
	assertion = assertion.add_item(AssertionLogic::new_item(
		ASSERTION_KEYS.address,
		Op::Equal,
		token_type.get_address(network)?,
	));
	Some(assertion)
}
