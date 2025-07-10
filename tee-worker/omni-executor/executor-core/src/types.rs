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

use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Serializable version of PackedUserOperation for use in NativeTask
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct SerializablePackedUserOperation {
	pub sender: String,             // Address as hex string (e.g., "0x1234...")
	pub nonce: u128,                // U256 as u128 integer
	pub init_code: String,          // Bytes as hex string (e.g., "0xabc...")
	pub call_data: String,          // Bytes as hex string (e.g., "0xdef...")
	pub account_gas_limits: String, // FixedBytes<32> as hex string (e.g., "0x123...")
	pub pre_verification_gas: u128, // U256 as u128 integer
	pub gas_fees: String,           // FixedBytes<32> as hex string (e.g., "0x456...")
	pub paymaster_and_data: String, // Bytes as hex string (e.g., "0x789...")
	pub signature: String,          // Bytes as hex string (e.g., "0xabc...")
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn test_serializable_packed_user_operation_deserialization() {
		let json_data = r#"{
			"sender": "0x1234567890123456789012345678901234567890",
			"nonce": 42,
			"init_code": "0xdeadbeef",
			"call_data": "0xcafebabe",
			"account_gas_limits": "0x0000000000000000000000000000000000000000000000000000000000030d40000000000000000000000000000000000000000000000000000000000000c350",
			"pre_verification_gas": 21000,
			"gas_fees": "0x000000000000000000000000000000000000000000000000000000003b9aca00000000000000000000000000000000000000000000000000000000000b2d05e0",
			"paymaster_and_data": "0x",
			"signature": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1c"
		}"#;

		let user_op: SerializablePackedUserOperation = serde_json::from_str(json_data)
			.expect("Failed to deserialize SerializablePackedUserOperation");

		// Verify all fields are correctly deserialized
		assert_eq!(user_op.sender, "0x1234567890123456789012345678901234567890");
		assert_eq!(user_op.nonce, 42);
		assert_eq!(user_op.init_code, "0xdeadbeef");
		assert_eq!(user_op.call_data, "0xcafebabe");
		assert_eq!(user_op.account_gas_limits, "0x0000000000000000000000000000000000000000000000000000000000030d40000000000000000000000000000000000000000000000000000000000000c350");
		assert_eq!(user_op.pre_verification_gas, 21000);
		assert_eq!(user_op.gas_fees, "0x000000000000000000000000000000000000000000000000000000003b9aca00000000000000000000000000000000000000000000000000000000000b2d05e0");
		assert_eq!(user_op.paymaster_and_data, "0x");
		assert_eq!(user_op.signature, "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1c");
	}

	#[test]
	fn test_serializable_packed_user_operation_serialization() {
		let user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 123,
			init_code: "0xabcd".to_string(),
			call_data: "0xef01".to_string(),
			account_gas_limits: "0x0000000000000000000000000000000000000000000000000000000000030d40000000000000000000000000000000000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 25000,
			gas_fees: "0x000000000000000000000000000000000000000000000000000000003b9aca00000000000000000000000000000000000000000000000000000000000b2d05e0".to_string(),
			paymaster_and_data: "0x456789".to_string(),
			signature: "0x987654321".to_string(),
		};

		let json_string = serde_json::to_string(&user_op)
			.expect("Failed to serialize SerializablePackedUserOperation");

		// Verify it can be deserialized back
		let deserialized: SerializablePackedUserOperation =
			serde_json::from_str(&json_string).expect("Failed to deserialize back");

		assert_eq!(user_op, deserialized);
	}
}
