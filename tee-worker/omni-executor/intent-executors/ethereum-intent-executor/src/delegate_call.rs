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

use alloy::primitives::{Address, Uint};
use alloy::sol_types::SolValue;
use alloy::{hex, sol};


sol!(
    #[sol(rpc)]
    "artifacts/SimpleDelegateContract.sol"
);

pub fn prepare_delegate_call_data(address: Address, input: Vec<u8>) -> Vec<u8> {
	let call = SimpleDelegateContract::Call {
		data: alloy::primitives::Bytes::from(input),
		to: address,
		value: Uint::from_str_radix("0", 10).unwrap(),
	};

	let calls = vec![call];
	let encoded_calls = calls.abi_encode();
	let method_hash = "a6d0ad61";

	// input = method hash + encoded calls
	let mut input = hex::decode(method_hash).unwrap();
	input.extend(encoded_calls);

	input
}
