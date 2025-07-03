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

use crate::{error::Result, NodeMetadata};

/// Pallet' name:
const UTIL: &str = "Utility";

pub trait UtilityCallIndexes {
	fn batch_call_indexes(&self) -> Result<[u8; 2]>;
	fn as_derivative_call_indexes(&self) -> Result<[u8; 2]>;
	fn batch_all_call_indexes(&self) -> Result<[u8; 2]>;
	fn dispatch_as_call_indexes(&self) -> Result<[u8; 2]>;
	fn force_batch_call_indexes(&self) -> Result<[u8; 2]>;
}

impl UtilityCallIndexes for NodeMetadata {
	fn batch_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(UTIL, "batch")
	}

	fn as_derivative_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(UTIL, "as_derivative")
	}

	fn batch_all_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(UTIL, "batch_all")
	}

	fn dispatch_as_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(UTIL, "dispatch_as")
	}

	fn force_batch_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(UTIL, "force_batch")
	}
}
