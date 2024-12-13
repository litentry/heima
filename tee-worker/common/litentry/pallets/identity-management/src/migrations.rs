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

use frame_support::{
	traits::{GetStorageVersion, PalletInfoAccess, StorageVersion},
	weights::Weight,
};

// This is just an example of how to write a custom migration
pub fn migrate_to_v1<T: crate::Config, P: GetStorageVersion + PalletInfoAccess>() -> Weight {
	let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();
	if on_chain_storage_version < 1 {
		log::info!("Doing migrations now for IMT version {:?}", on_chain_storage_version);
		StorageVersion::new(1).put::<P>();
	}
	Weight::zero()
}

pub fn migrate_to_v2<T: crate::Config, P: GetStorageVersion + PalletInfoAccess>() -> Weight {
	let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();
	if on_chain_storage_version < 2 {
		log::info!("Doing migrations now for IMT version {:?}", on_chain_storage_version);
		StorageVersion::new(2).put::<P>();

		// '//Charlie' -> 0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22
		let mut charlie = [0; 32];
		hex::decode_to_slice(
			"90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22",
			&mut charlie,
		)
		.unwrap();

		// Update after P-174:
		// the following code snippet only serves as an example based on the old codebase, it doesn't
		// work now as we don't have the storage item anymore, thus commented out.
		//
		// set Charlie's shielding key to [2u8; 32]
		// verifiable via:
		// ```
		// ./bin/litentry-cli trusted --mrenclave $mrenclave get-storage IdentityManagement UserShieldingKeys 90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22
		// ```
		// migration::put_storage_value::<RequestAesKey>(
		// 	"IdentityManagement".as_bytes(),
		// 	"UserShieldingKeys".as_bytes(),
		// 	&charlie.blake2_128_concat(),
		// 	[2u8; 32],
		// );
	}
	Weight::zero()
}
