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

use crate::{Cli, CliResult, CliResultOk};

// Display the current system version detail
// usage example:
// ./litentry-cli system-version
#[derive(Parser)]
pub struct SystemVersionCommand {}

impl SystemVersionCommand {
	pub(crate) fn run(&self, _cli: &Cli) -> CliResult {
		itc_system_version::print_system_version();
		Ok(CliResultOk::Bytes { bytes: vec![] })
	}
}
