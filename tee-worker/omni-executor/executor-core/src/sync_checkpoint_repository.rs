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

use parity_scale_codec::{Decode, Encode};
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::Path;

/// Represents the point in chain. It can be a whole block or a more precise unit, for example
/// in case of substrate chain it is BLOCK_NUM::EVENT_NUM
pub trait Checkpoint {
	// determines whether checkpoint is a whole block or not
	fn just_block_num(&self) -> bool;
	fn get_block_num(&self) -> u64;
}

/// Used for saving and reading `Checkpoint`
pub trait CheckpointRepository<Checkpoint> {
	fn get(&self) -> Result<Option<Checkpoint>, ()>;
	fn save(&mut self, checkpoint: Checkpoint) -> Result<(), ()>;
}

/// Simple `CheckpointRepository`. Checkpoints are not persisted across restarts.
pub struct InMemoryCheckpointRepository<Checkpoint> {
	last: Option<Checkpoint>,
}

impl<Checkpoint> InMemoryCheckpointRepository<Checkpoint> {
	pub fn new(last: Option<Checkpoint>) -> Self {
		Self { last }
	}
}

impl<Checkpoint> CheckpointRepository<Checkpoint> for InMemoryCheckpointRepository<Checkpoint>
where
	Checkpoint: Clone,
{
	fn get(&self) -> Result<Option<Checkpoint>, ()> {
		Ok(self.last.clone())
	}

	fn save(&mut self, checkpoint: Checkpoint) -> Result<(), ()> {
		self.last = Some(checkpoint);
		Ok(())
	}
}

/// File based `CheckpointRepository`. Used to persist checkpoints across restarts.
pub struct FileCheckpointRepository {
	file_name: String,
}

impl FileCheckpointRepository {
	pub fn new(file_name: &str) -> Self {
		let file_name_regex = regex::Regex::new(r"^[a-zA-Z0-9_\-\.\/]+$").expect("Invalid regex");
		if !file_name_regex.is_match(file_name) {
			panic!("Invalid file name: {}", file_name);
		}

		if let Some(parent) = Path::new(file_name).parent() {
			fs::create_dir_all(parent).expect("Failed to create directories");
		}

		Self { file_name: file_name.to_owned() }
	}
}

impl<Checkpoint> CheckpointRepository<Checkpoint> for FileCheckpointRepository
where
	Checkpoint: Encode + Decode + Debug,
{
	fn get(&self) -> Result<Option<Checkpoint>, ()> {
		match fs::read(&self.file_name) {
			Ok(content) => {
				let checkpoint: Checkpoint =
					Checkpoint::decode(&mut content.as_slice()).map_err(|e| {
						log::error!("Could not decode last processed log: {:?}", e);
					})?;
				Ok(Some(checkpoint))
			},
			Err(e) => match e.kind() {
				ErrorKind::NotFound => Ok(None),
				_ => {
					log::error!("Could not open file {:?}", e);
					Err(())
				},
			},
		}
	}

	fn save(&mut self, checkpoint: Checkpoint) -> Result<(), ()> {
		log::trace!("Saving checkpoint: {:?}", checkpoint);
		let content = checkpoint.encode();
		if let Ok(mut file) = File::create(&self.file_name) {
			file.write(content.as_slice()).map_err(|_| ())?;
			Ok(())
		} else {
			Err(())
		}
	}
}
