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

use itp_types::parentchain::ParentchainId;

/// Interface trait of the parentchain pallet.
pub trait ParentchainPalletInstancesInterface<State, ParentchainHeader> {
	type Error;

	/// Updates the block number, block hash and parent hash of the parentchain block.
	fn update_parentchain_litentry_block(
		state: &mut State,
		header: ParentchainHeader,
	) -> Result<(), Self::Error>;

	/// Updates the block number, block hash and parent hash of the parentchain block.
	fn update_parentchain_target_a_block(
		state: &mut State,
		header: ParentchainHeader,
	) -> Result<(), Self::Error>;

	/// Updates the block number, block hash and parent hash of the parentchain block.
	fn update_parentchain_target_b_block(
		state: &mut State,
		header: ParentchainHeader,
	) -> Result<(), Self::Error>;

	fn set_creation_block(
		state: &mut State,
		header: ParentchainHeader,
		parentchain_id: ParentchainId,
	) -> Result<(), Self::Error>;
}
