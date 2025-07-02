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

use itp_storage::{storage_map_key, StorageHasher};
use itp_types::WorkerType;
use sp_std::prelude::Vec;

pub struct TeebagStorage;

pub trait StoragePrefix {
	fn prefix() -> &'static str;
}

impl StoragePrefix for TeebagStorage {
	fn prefix() -> &'static str {
		"Teebag"
	}
}

pub trait TeebagStorageKeys {
	fn enclave_identifier(worker_type: WorkerType) -> Vec<u8>;
}

impl<S: StoragePrefix> TeebagStorageKeys for S {
	fn enclave_identifier(worker_type: WorkerType) -> Vec<u8> {
		storage_map_key(
			Self::prefix(),
			"EnclaveIdentifier",
			&worker_type,
			&StorageHasher::Blake2_128Concat,
		)
	}
}
