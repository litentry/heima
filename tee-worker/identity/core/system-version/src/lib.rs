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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use std::{format, println, string::String};

mod built_info {
	include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const DEFAULT_VALUE: &str = "unknown";

pub fn print_system_version() {
	println!(
		r#"Version Information:
------------------
Target: {}
Rustc version: {}

Git Information:
--------------
Version: {}
Commit Hash: {}

Build Time:
---------
{}"#,
		built_info::TARGET,
		built_info::RUSTC_VERSION,
		option_env!("GIT_VERSION").unwrap_or(DEFAULT_VALUE),
		option_env!("GIT_COMMIT_HASH").unwrap_or(DEFAULT_VALUE),
		built_info::BUILT_TIME_UTC,
	)
}

pub fn get_system_version() -> String {
	format!(
		r#"{{"target":"{}","rustc_version":"{}","git_version":"{}","git_commit_hash":"{}","build_time":"{}"}}"#,
		built_info::TARGET,
		built_info::RUSTC_VERSION,
		option_env!("GIT_VERSION").unwrap_or(DEFAULT_VALUE),
		option_env!("GIT_COMMIT_HASH").unwrap_or(DEFAULT_VALUE),
		built_info::BUILT_TIME_UTC
	)
}
