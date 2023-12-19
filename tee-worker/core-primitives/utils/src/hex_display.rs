// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Wrapper type for byte collections that outputs hex.
//!
//! Copied from sp-core and made purely no-std.

/// Simple wrapper to display hex representation of bytes.
pub struct HexDisplay<'a>(&'a [u8]);

impl<'a> HexDisplay<'a> {
	/// Create new instance that will display `d` as a hex string when displayed.
	pub fn from<R: AsBytesRef>(d: &'a R) -> Self {
		HexDisplay(d.as_bytes_ref())
	}
}

impl<'a> core::fmt::Display for HexDisplay<'a> {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
		if self.0.len() < 1027 {
			for byte in self.0 {
				f.write_fmt(format_args!("{:02x}", byte))?;
			}
		} else {
			for byte in &self.0[0..512] {
				f.write_fmt(format_args!("{:02x}", byte))?;
			}
			f.write_str("...")?;
			for byte in &self.0[self.0.len() - 512..] {
				f.write_fmt(format_args!("{:02x}", byte))?;
			}
		}
		Ok(())
	}
}

impl<'a> core::fmt::Debug for HexDisplay<'a> {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
		for byte in self.0 {
			f.write_fmt(format_args!("{:02x}", byte))?;
		}
		Ok(())
	}
}

/// Simple trait to transform various types to `&[u8]`
pub trait AsBytesRef {
	/// Transform `self` into `&[u8]`.
	fn as_bytes_ref(&self) -> &[u8];
}

impl AsBytesRef for &[u8] {
	fn as_bytes_ref(&self) -> &[u8] {
		self
	}
}

impl AsBytesRef for [u8] {
	fn as_bytes_ref(&self) -> &[u8] {
		self
	}
}

impl AsBytesRef for alloc::vec::Vec<u8> {
	fn as_bytes_ref(&self) -> &[u8] {
		self
	}
}

macro_rules! impl_non_endians {
	( $( $t:ty ),* ) => { $(
		impl AsBytesRef for $t {
			fn as_bytes_ref(&self) -> &[u8] { &self[..] }
		}
	)* }
}

impl_non_endians!(
	[u8; 1], [u8; 2], [u8; 3], [u8; 4], [u8; 5], [u8; 6], [u8; 7], [u8; 8], [u8; 10], [u8; 12],
	[u8; 14], [u8; 16], [u8; 20], [u8; 24], [u8; 28], [u8; 32], [u8; 40], [u8; 48], [u8; 56],
	[u8; 64], [u8; 65], [u8; 80], [u8; 96], [u8; 112], [u8; 128]
);