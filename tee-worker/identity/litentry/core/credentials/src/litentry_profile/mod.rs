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

pub mod holding_amount;
pub mod lit_staking;
pub mod mirror;
pub mod token_balance;

#[derive(Debug)]
pub struct BalanceRange;
pub trait BalanceRangeIndex {
	fn index<T>(source: &[T], balance: T) -> Option<usize>
	where
		T: Into<f64> + std::fmt::Debug + std::cmp::PartialOrd<T>;
}

impl BalanceRangeIndex for BalanceRange {
	fn index<T>(source: &[T], balance: T) -> Option<usize>
	where
		T: Into<f64> + std::fmt::Debug + std::cmp::PartialOrd<T>,
	{
		for (index, item) in source.iter().enumerate() {
			if balance < *item {
				return Some(index - 1);
			}
		}

		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::litentry_profile::holding_amount::ETH_HOLDING_AMOUNT_RANGE;

	#[test]
	fn balance_index_eth_holding_false_works() {
		let balance = 0.0;
		let index = BalanceRange::index(&ETH_HOLDING_AMOUNT_RANGE, balance);
		assert_eq!(index.unwrap(), 0);
	}

	#[test]
	fn balance_index_eth_holding_true_works() {
		let balance = 10.0;
		let index = BalanceRange::index(&ETH_HOLDING_AMOUNT_RANGE, balance);
		assert_eq!(index.unwrap(), 5);
	}

	#[test]
	fn balance_index_eth_holding_max_works() {
		let balance = 1000000.0;
		let index = BalanceRange::index(&ETH_HOLDING_AMOUNT_RANGE, balance);
		assert_eq!(index, None);
	}
}
