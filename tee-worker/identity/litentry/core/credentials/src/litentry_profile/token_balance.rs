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

use super::{BalanceRange, BalanceRangeIndex};
use crate::{
	assertion_logic::{AssertionLogic, Op},
	Credential,
};
use lc_data_providers::ETokenAddress;
use std::vec::Vec;

// [LIT, USDC, USDT, WBTC]
// (type, description)
const VC_TOKEN_BALANCE_INFOS: [(&str, &str); 1] =
	[("Token holding amount", "The amount of a particular token you are holding")];

const USDT_C_TOKEN_BALANCE_RANGE: [f64; 10] =
	[0.0, 5.0, 10.0, 50.0, 100.0, 150.0, 300.0, 500.0, 800.0, 1200.0];
const LIT_TOKEN_BALANCE_RANGE: [f64; 8] = [0.0, 100.0, 200.0, 500.0, 800.0, 1200.0, 1600.0, 3000.0];
const WBTC_TOKEN_BALANCE_RANGE: [f64; 10] =
	[0.0, 0.001, 0.05, 0.1, 0.5, 10.0, 50.0, 100.0, 500.0, 1000.0];
const CRV_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0, 1800.0];
const MATIC_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0, 1800.0];
const DYDX_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 20.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0];
const AMP_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 2000.0, 5000.0, 10000.0, 18000.0, 30000.0, 50000.0, 80000.0, 120000.0];
const CVX_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 20.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0];
const TUSD_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 20.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0];
const USDD_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 20.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0];
const GUSD_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 20.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0];
const LINK_TOKEN_BALANCE_RANGE: [f64; 9] = [0.0, 1.0, 5.0, 20.0, 40.0, 80.0, 150.0, 240.0, 400.0];
const GRT_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0, 1800.0];
const COMP_TOKEN_BALANCE_RANGE: [f64; 11] =
	[0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 80.0, 150.0, 250.0, 400.0];
const PEOPLE_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 200.0, 500.0, 1000.0, 1800.0, 3000.0, 5000.0, 8000.0, 12000.0];
const GTC_TOKEN_BALANCE_RANGE: [f64; 9] =
	[0.0, 1.0, 20.0, 50.0, 100.0, 200.0, 500.0, 800.0, 1200.0];

pub trait TokenBalanceInfo {
	fn update_token_balance(&mut self, token: ETokenAddress, balance: f64);
}

impl TokenBalanceInfo for Credential {
	fn update_token_balance(&mut self, token: ETokenAddress, balance: f64) {
		let info = get_token_info(&token);
		self.add_subject_info(info.1, info.0);

		update_assertion(token, balance, self);
	}
}

fn update_assertion(token: ETokenAddress, balance: f64, credential: &mut Credential) {
	let mut assertion = AssertionLogic::new_and();

	let content = get_assertion_content(&token);
	let range = get_balance_range(&token);
	let index = BalanceRange::index(&range, balance);
	match index {
		Some(index) => {
			let min = format!("{}", range[index]);
			let max = format!("{}", range[index + 1]);
			let min_item = AssertionLogic::new_item(
				content,
				if index == 0 { Op::GreaterThan } else { Op::GreaterEq },
				&min,
			);

			assertion = assertion.add_item(min_item);
			if balance > 0_f64 {
				let max_item = AssertionLogic::new_item(content, Op::LessThan, &max);
				assertion = assertion.add_item(max_item);
			}

			credential.credential_subject.values.push(index != 0 || balance > 0_f64);
		},
		None => {
			let min_item = AssertionLogic::new_item(
				content,
				Op::GreaterEq,
				&format!("{}", get_token_range_last(&token)),
			);
			assertion = assertion.add_item(min_item);

			credential.credential_subject.values.push(true);
		},
	}

	credential.credential_subject.assertions.push(assertion);
}

fn get_token_info(token: &ETokenAddress) -> (&'static str, &'static str) {
	match token {
		ETokenAddress::WBTC
		| ETokenAddress::LIT
		| ETokenAddress::USDC
		| ETokenAddress::USDT
		| ETokenAddress::CRV
		| ETokenAddress::MATIC
		| ETokenAddress::DYDX
		| ETokenAddress::AMP
		| ETokenAddress::CVX
		| ETokenAddress::TUSD
		| ETokenAddress::USDD
		| ETokenAddress::GUSD
		| ETokenAddress::LINK
		| ETokenAddress::GRT
		| ETokenAddress::COMP
		| ETokenAddress::PEOPLE
		| ETokenAddress::GTC => VC_TOKEN_BALANCE_INFOS[0],
		_ => ("UnknownType", ("UnkonwDescription")),
	}
}

fn get_assertion_content(token: &ETokenAddress) -> &'static str {
	match token {
		ETokenAddress::WBTC => "$wbtc_holding_amount",
		ETokenAddress::LIT => "$lit_holding_amount",
		ETokenAddress::USDC => "$usdc_holding_amount",
		ETokenAddress::USDT => "$usdt_holding_amount",
		ETokenAddress::CRV => "$crv_holding_amount",
		ETokenAddress::MATIC => "$matic_holding_amount",
		ETokenAddress::DYDX => "$dydx_holding_amount",
		ETokenAddress::AMP => "$amp_holding_amount",
		ETokenAddress::CVX => "$cvx_holding_amount",
		ETokenAddress::TUSD => "$tusd_holding_amount",
		ETokenAddress::USDD => "$usdd_holding_amount",
		ETokenAddress::GUSD => "$gusd_holding_amount",
		ETokenAddress::LINK => "$link_holding_amount",
		ETokenAddress::GRT => "$grt_holding_amount",
		ETokenAddress::COMP => "$comp_holding_amount",
		ETokenAddress::PEOPLE => "$people_holding_amount",
		ETokenAddress::GTC => "$gtc_holding_amount",
		_ => "Unknown",
	}
}

fn get_balance_range(token: &ETokenAddress) -> Vec<f64> {
	match token {
		ETokenAddress::WBTC => WBTC_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::LIT => LIT_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::USDC | ETokenAddress::USDT => USDT_C_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::CRV => CRV_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::MATIC => MATIC_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::DYDX => DYDX_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::AMP => AMP_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::CVX => CVX_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::TUSD => TUSD_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::USDD => USDD_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::GUSD => GUSD_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::LINK => LINK_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::GRT => GRT_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::COMP => COMP_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::PEOPLE => PEOPLE_TOKEN_BALANCE_RANGE.to_vec(),
		ETokenAddress::GTC => GTC_TOKEN_BALANCE_RANGE.to_vec(),
		_ => {
			vec![]
		},
	}
}

fn get_token_range_last(token: &ETokenAddress) -> f64 {
	match token {
		ETokenAddress::WBTC => *WBTC_TOKEN_BALANCE_RANGE.last().unwrap_or(&1000.0),
		ETokenAddress::LIT => *LIT_TOKEN_BALANCE_RANGE.last().unwrap_or(&3000.0),
		ETokenAddress::USDC | ETokenAddress::USDT => {
			*USDT_C_TOKEN_BALANCE_RANGE.last().unwrap_or(&1200.0)
		},
		ETokenAddress::CRV => *CRV_TOKEN_BALANCE_RANGE.last().unwrap_or(&1800.0),
		ETokenAddress::MATIC => *MATIC_TOKEN_BALANCE_RANGE.last().unwrap_or(&1800.0),
		ETokenAddress::DYDX => *DYDX_TOKEN_BALANCE_RANGE.last().unwrap_or(&1200.0),
		ETokenAddress::AMP => *AMP_TOKEN_BALANCE_RANGE.last().unwrap_or(&1200.0),
		ETokenAddress::CVX => *CVX_TOKEN_BALANCE_RANGE.last().unwrap_or(&12000.0),
		ETokenAddress::TUSD => *TUSD_TOKEN_BALANCE_RANGE.last().unwrap_or(&1200.0),
		ETokenAddress::USDD => *USDD_TOKEN_BALANCE_RANGE.last().unwrap_or(&1200.0),
		ETokenAddress::GUSD => *GUSD_TOKEN_BALANCE_RANGE.last().unwrap_or(&1200.0),
		ETokenAddress::LINK => *LINK_TOKEN_BALANCE_RANGE.last().unwrap_or(&400.0),
		ETokenAddress::GRT => *GRT_TOKEN_BALANCE_RANGE.last().unwrap_or(&1800.0),
		ETokenAddress::COMP => *COMP_TOKEN_BALANCE_RANGE.last().unwrap_or(&400.0),
		ETokenAddress::PEOPLE => *PEOPLE_TOKEN_BALANCE_RANGE.last().unwrap_or(&12000.0),
		ETokenAddress::GTC => *GTC_TOKEN_BALANCE_RANGE.last().unwrap_or(&1200.0),
		_ => 0.0,
	}
}
