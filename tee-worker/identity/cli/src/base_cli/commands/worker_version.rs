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

mod built_info {
	include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// Dispnay the current worker version detail
// usage example:
// ./litentry-cli worker-version
#[derive(Parser)]
pub struct WorkerVersionCommand {}

impl WorkerVersionCommand {
	pub(crate) fn run(&self, cli: &Cli) -> CliResult {
		println!(
			r#"
Version Information:
------------------
Package version: {}
Target: {}
Rustc version: {}

Git Information:
--------------
Version: {}
Commit Hash: {}

Build Time:
---------
{}
            "#,
			built_info::PKG_VERSION,
			built_info::TARGET,
			built_info::RUSTC_VERSION,
			built_info::GIT_VERSION.unwrap_or("unknown"),
			built_info::GIT_COMMIT_HASH.unwrap_or("unknown"),
			built::util::strptime(built_info::BUILT_TIME_UTC),
		);
		Ok(CliResultOk::Bytes { bytes: () })
	}
}
