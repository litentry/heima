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

pub mod assertion;
pub mod identity_verification;

use ita_stf::TrustedCall;
use itp_types::{ShardIdentifier, H256};
use std::sync::mpsc::Sender;

pub trait TaskHandler {
	type Error;
	type Result;
	fn start(&self, sender: Sender<(ShardIdentifier, H256, TrustedCall)>) {
		match self.on_process() {
			Ok(r) => self.on_success(r, sender),
			Err(e) => self.on_failure(e, sender),
		}
	}
	fn on_process(&self) -> Result<Self::Result, Self::Error>;
	fn on_success(&self, r: Self::Result, sender: Sender<(ShardIdentifier, H256, TrustedCall)>);
	fn on_failure(&self, e: Self::Error, sender: Sender<(ShardIdentifier, H256, TrustedCall)>);
}
