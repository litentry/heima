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

use log::error;
use std::fs;
use std::fs::File;
use std::io::Write;

/// Used for persisting Relayer's keys.
pub trait KeyStore<K> {
	fn generate_key() -> Result<K, ()>;
	fn serialize(k: &K) -> Result<Vec<u8>, ()>;
	fn deserialize(sealed: Vec<u8>) -> Result<K, ()>;
	fn path(&self) -> String;
	fn read(&self) -> Result<K, ()> {
		match fs::read(self.path()) {
			Ok(content) => Self::deserialize(content),
			Err(_) => Err(()),
		}
	}
	fn write(&self, k: &K) -> Result<(), ()> {
		match File::create(self.path()) {
			Ok(mut file) => {
				file.write(
					&Self::serialize(k).map_err(|e| error!("Error writing to file: {:?}", e))?,
				)
				.map_err(|_| ())?;
				Ok(())
			},
			Err(e) => {
				error!("Could not seal data to file {}: {:?}", self.path(), e);
				Err(())
			},
		}
	}
}
