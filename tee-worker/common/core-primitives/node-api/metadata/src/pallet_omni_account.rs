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

const OMNIACCOUNT: &str = "OmniAccount";

pub trait OmniAccountCallIndexes {
	fn dispatch_as_omni_account_call_indexes(&self) -> Result<[u8; 2]>;
	fn dispatch_as_signed_call_indexes(&self) -> Result<[u8; 2]>;
	fn create_account_store_call_indexes(&self) -> Result<[u8; 2]>;
	fn update_account_store_by_one_call_indexes(&self) -> Result<[u8; 2]>;
}

impl OmniAccountCallIndexes for NodeMetadata {
	fn dispatch_as_omni_account_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(OMNIACCOUNT, "dispatch_as_omni_account")
	}

	fn dispatch_as_signed_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(OMNIACCOUNT, "dispatch_as_signed")
	}

	fn create_account_store_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(OMNIACCOUNT, "create_account_store")
	}

	fn update_account_store_by_one_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(OMNIACCOUNT, "update_account_store_by_one")
	}
}
