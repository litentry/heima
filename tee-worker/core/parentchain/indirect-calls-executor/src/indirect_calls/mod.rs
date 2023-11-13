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

mod invoke;
mod litentry;
mod shield_funds;
mod transfer_to_alice_shields_funds;

pub use invoke::InvokeArgs;
pub use litentry::{
	activate_identity::ActivateIdentityArgs,
	deactivate_identity::DeactivateIdentityArgs,
	link_identity::LinkIdentityArgs,
	request_vc::RequestVCArgs,
	scheduled_enclave::{RemoveScheduledEnclaveArgs, UpdateScheduledEnclaveArgs},
};
pub use shield_funds::ShieldFundsArgs;
pub use transfer_to_alice_shields_funds::{TransferToAliceShieldsFundsArgs, ALICE_ACCOUNT_ID};
