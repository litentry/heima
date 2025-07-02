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

// TODO: maybe use macros to simplify this
use crate::{error::Result, NodeMetadata};

const VCMP: &str = "VCManagement";

pub trait VCMPCallIndexes {
	fn request_vc_call_indexes(&self) -> Result<[u8; 2]>;

	fn vc_issued_call_indexes(&self) -> Result<[u8; 2]>;

	fn vcmp_some_error_call_indexes(&self) -> Result<[u8; 2]>;
}

impl VCMPCallIndexes for NodeMetadata {
	fn request_vc_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(VCMP, "request_vc")
	}

	fn vc_issued_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(VCMP, "vc_issued")
	}

	fn vcmp_some_error_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(VCMP, "some_error")
	}
}
