/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/
use crate::mock::*;
use frame_support::{assert_err, assert_ok};
use sp_core::H256;
use sp_keyring::AccountKeyring;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, Header as HeaderT},
	DispatchError::BadOrigin,
};

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

#[test]
fn verify_storage_works() {
	let block_number = 3;
	let parent_hash = H256::from_low_u64_be(420);

	let header: Header = HeaderT::new(
		block_number,
		Default::default(),
		Default::default(),
		parent_hash,
		Default::default(),
	);
	let hash = header.hash();

	new_test_ext().execute_with(|| {
		assert_ok!(Parentchain::set_block(RuntimeOrigin::root(), header));
		assert_eq!(Parentchain::block_number(), block_number);
		assert_eq!(Parentchain::parent_hash(), parent_hash);
		assert_eq!(Parentchain::block_hash(), hash);
	})
}

#[test]
fn non_root_account_errs() {
	let header = HeaderT::new(
		1,
		Default::default(),
		Default::default(),
		[69; 32].into(),
		Default::default(),
	);

	new_test_ext().execute_with(|| {
		let root = AccountKeyring::Ferdie.to_account_id();
		assert_err!(Parentchain::set_block(RuntimeOrigin::signed(root), header), BadOrigin);
	})
}
