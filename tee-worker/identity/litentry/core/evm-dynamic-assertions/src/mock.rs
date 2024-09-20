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

use crate::sealing::{SealingError, UnsealedAssertions};
pub use itp_sgx_io::SealedIO;
pub use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct AssertionsSealMock {}

impl AssertionsSealMock {
	pub fn new() -> Self {
		Self {}
	}
}

impl SealedIO for AssertionsSealMock {
	type Error = SealingError;
	type Unsealed = UnsealedAssertions;

	fn unseal(&self) -> Result<Self::Unsealed, Self::Error> {
		Ok(UnsealedAssertions::default())
	}

	fn seal(&self, _unsealed: &Self::Unsealed) -> Result<(), Self::Error> {
		Ok(())
	}
}
