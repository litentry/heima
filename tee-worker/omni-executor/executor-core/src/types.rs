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
	pub sender: [u8; 20],               // Address as byte array
	pub nonce: [u8; 32],                // U256 as byte array
	pub init_code: Vec<u8>,             // Bytes as Vec<u8>
	pub call_data: Vec<u8>,             // Bytes as Vec<u8>
	pub account_gas_limits: [u8; 32],   // FixedBytes<32> as byte array
	pub pre_verification_gas: [u8; 32], // U256 as byte array
	pub gas_fees: [u8; 32],             // FixedBytes<32> as byte array
	pub paymaster_and_data: Vec<u8>,    // Bytes as Vec<u8>
	pub signature: Vec<u8>,             // Bytes as Vec<u8>
}
