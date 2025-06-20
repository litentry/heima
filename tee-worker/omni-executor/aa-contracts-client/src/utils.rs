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

use alloy::primitives::{Address, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};

/// Build a transaction request for a contract call (no value transfer)
pub fn build_call_transaction(to: Address, call_data: Vec<u8>) -> TransactionRequest {
	TransactionRequest {
		to: Some(TxKind::Call(to)),
		input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
		..Default::default()
	}
}

/// Build a transaction request for a payable contract call (with value transfer)
pub fn build_payable_transaction(
	to: Address,
	call_data: Vec<u8>,
	value: U256,
) -> TransactionRequest {
	TransactionRequest {
		to: Some(TxKind::Call(to)),
		input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
		value: Some(value),
		..Default::default()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloy::primitives::address;

	#[test]
	fn test_build_call_transaction() {
		let address = address!("0x1234567890123456789012345678901234567890");
		let call_data = vec![0x12, 0x34, 0x56];

		let tx = build_call_transaction(address, call_data.clone());

		assert_eq!(tx.to, Some(TxKind::Call(address)));
		assert_eq!(tx.input.data, Some(call_data.into()));
		assert_eq!(tx.value, None);
	}

	#[test]
	fn test_build_payable_transaction() {
		let address = address!("0x1234567890123456789012345678901234567890");
		let call_data = vec![0x12, 0x34, 0x56];
		let value = U256::from(1000);

		let tx = build_payable_transaction(address, call_data.clone(), value);

		assert_eq!(tx.to, Some(TxKind::Call(address)));
		assert_eq!(tx.input.data, Some(call_data.into()));
		assert_eq!(tx.value, Some(value));
	}
}
