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

use codec::{Decode, Encode};

use sp_std::vec::Vec;

pub use ita_sgx_runtime::{Balance, Index};
use ita_stf::{Getter, TrustedCall, TrustedCallSigned};
use itc_parentchain_indirect_calls_executor::error::Error;
use itp_stf_primitives::{traits::IndirectExecutor, types::TrustedOperation};
use itp_types::parentchain::{
	events::{BalanceTransfer, LinkIdentityRequested},
	AccountId, FilterEvents, HandleParentchainEvents, ParentchainError, ParentchainId,
};
use litentry_hex_utils::hex_encode;
use litentry_primitives::{Identity, ValidationData, Web3Network};
use log::*;

pub struct ParentchainEventHandler {}

impl ParentchainEventHandler {
	fn shield_funds<Executor: IndirectExecutor<TrustedCallSigned, Error>>(
		executor: &Executor,
		account: &AccountId,
		amount: Balance,
	) -> Result<(), Error> {
		log::info!("shielding for {:?} amount {}", account, amount,);
		let shard = executor.get_default_shard();
		// todo: ensure this parentchain is assigned for the shard vault!
		let trusted_call = TrustedCall::balance_shield(
			executor.get_enclave_account()?.into(),
			account.clone(),
			amount,
			ParentchainId::Litentry,
		);
		let signed_trusted_call = executor.sign_call_with_self(&trusted_call, &shard)?;
		let trusted_operation =
			TrustedOperation::<TrustedCallSigned, Getter>::indirect_call(signed_trusted_call);

		let encrypted_trusted_call = executor.encrypt(&trusted_operation.encode())?;
		executor.submit_trusted_call(shard, encrypted_trusted_call);

		Ok(())
	}

	fn link_identity<Executor: IndirectExecutor<TrustedCallSigned, Error>>(
		executor: &Executor,
		account: &AccountId,
		encrypted_identity: Vec<u8>,
		encrypted_validation_data: Vec<u8>,
		encrypted_web3networks: Vec<u8>,
	) -> Result<(), Error> {
		let shard = executor.get_default_shard();
		let enclave_account_id = executor.get_enclave_account().expect("no enclave account");

		let identity: Identity =
			Identity::decode(&mut executor.decrypt(&encrypted_identity)?.as_slice())?;
		let validation_data =
			ValidationData::decode(&mut executor.decrypt(&encrypted_validation_data)?.as_slice())?;
		let web3networks: Vec<Web3Network> =
			Decode::decode(&mut executor.decrypt(&encrypted_web3networks)?.as_slice())?;

		let trusted_call = TrustedCall::link_identity(
			enclave_account_id.into(),
			account.clone().into(),
			identity,
			validation_data,
			web3networks,
			None,
			Default::default(),
		);
		let signed_trusted_call = executor.sign_call_with_self(&trusted_call, &shard)?;
		let trusted_operation =
			TrustedOperation::<TrustedCallSigned, Getter>::indirect_call(signed_trusted_call);

		let encrypted_trusted_call = executor.encrypt(&trusted_operation.encode())?;
		executor.submit_trusted_call(shard, encrypted_trusted_call);

		Ok(())
	}
}

impl<Executor> HandleParentchainEvents<Executor, TrustedCallSigned, Error>
	for ParentchainEventHandler
where
	Executor: IndirectExecutor<TrustedCallSigned, Error>,
{
	fn handle_events(
		executor: &Executor,
		events: impl FilterEvents,
		vault_account: &AccountId,
	) -> Result<(), Error> {
		if let Ok(events) = events.get_events::<BalanceTransfer>() {
			trace!(
				"filtering transfer events to shard vault account: {}",
				hex_encode(vault_account.encode().as_slice())
			);
			events
				.iter()
				.filter(|&event| event.to == *vault_account)
				.try_for_each(|event| {
					info!("found transfer_event to vault account: {}", event);
					//debug!("shielding from Integritee suppressed");
					Self::shield_funds(executor, &event.from, event.amount)
					//Err(ParentchainError::FunctionalityDisabled)
				})
				.map_err(|_| ParentchainError::ShieldFundsFailure)?;
		}

		if let Ok(events) = events.get_events::<LinkIdentityRequested>() {
			debug!("Handling link_identity events");
			events
				.iter()
				.try_for_each(|event| {
					info!("found link_identity_event: {}", event);
					Self::link_identity(
						executor,
						&event.account,
						event.encrypted_identity.clone(),
						event.encrypted_validation_data.clone(),
						event.encrypted_web3networks.clone(),
					)
				})
				.map_err(|_| ParentchainError::LinkIdentityFailure)?;
		}

		Ok(())
	}
}
