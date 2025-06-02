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

// VC Schema
// According to https://w3c-ccg.github.io/vc-json-schemas/, it defines JSON Schema for W3C Verifiable Credential.

use crate::Config;
use heima_primitives::{SchemaContentString, SchemaIdString};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Status {
	#[codec(index = 0)]
	Active,
	#[codec(index = 1)]
	Disabled,
	// Revoked, // commented out for now, we can delete the VC entry when revoked
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct VCSchema<T: Config> {
	// the schema id
	pub id: SchemaIdString,
	// the schema author
	pub author: T::AccountId,
	// schema content
	pub content: SchemaContentString,
	// status of the Schema
	pub status: Status,
}

impl<T: Config> VCSchema<T> {
	pub fn new(sid: Vec<u8>, author: T::AccountId, scontent: Vec<u8>) -> Self {
		let id: SchemaIdString = sid.try_into().expect("error convert to BoundedVec");
		let content: SchemaContentString =
			scontent.try_into().expect("error convert to BoundedVec");

		Self { id, author, content, status: Status::Active }
	}
}
