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

use std::process::Command;

fn main() {
	// Generate build-time information
	built::write_built_file().expect("Failed to acquire build-time information");

	// Example value: ce3abe05a53fcdf103af58b46b3974cb24309543
	if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
		if output.status.success() {
			if let Ok(hash) = String::from_utf8(output.stdout) {
				println!("cargo:rustc-env=GIT_COMMIT_HASH={}", hash.trim());
			}
		}
	}

	// Example value: v0.9.20-07-51-gce3abe05
	if let Ok(output) = Command::new("git").args(["describe", "--tags", "--long"]).output() {
		if output.status.success() {
			if let Ok(version) = String::from_utf8(output.stdout) {
				println!("cargo:rustc-env=GIT_VERSION={}", version.trim());
			}
		}
	}
}
