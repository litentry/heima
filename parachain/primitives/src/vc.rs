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

#[cfg(feature = "std")]
extern crate std;

use sp_runtime::{traits::ConstU32, BoundedVec};

// vc schema
pub const SCHEMA_ID_LEN: u32 = 512;
pub const SCHEMA_CONTENT_LEN: u32 = 2048;

/// An index of a schema. Just a `u64`.
pub type SchemaIndex = u64;

pub type MaxIdLength = ConstU32<SCHEMA_ID_LEN>;
pub type SchemaIdString = BoundedVec<u8, MaxIdLength>;

pub type MaxContentLength = ConstU32<SCHEMA_CONTENT_LEN>;
pub type SchemaContentString = BoundedVec<u8, MaxContentLength>;
