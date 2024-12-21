/*
	Copyright 2021 Integritee AG

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
//! Various way to filter Parentchain events

use itp_api_client_types::{EventDetails, Events};
use itp_node_api::api_client::StaticEvent;
use itp_types::{
	parentchain::{events::*, FilterEvents},
	H256,
};
use std::{vec, vec::Vec};

#[derive(Clone)]
pub struct FilterableEvents {
	events: Vec<EventDetails<H256>>,
}

impl FilterableEvents {
	pub fn new(events: Events<H256>) -> Self {
		let mut interesting_events = vec![];
		events.iter().flatten().for_each(|ev| {
			// lets keep only events worker is interested in
			if ev.pallet_name() == LinkIdentityRequested::PALLET
				&& ev.variant_name() == LinkIdentityRequested::EVENT
				|| ev.pallet_name() == VCRequested::PALLET
					&& ev.variant_name() == VCRequested::EVENT
				|| ev.pallet_name() == DeactivateIdentityRequested::PALLET
					&& ev.variant_name() == DeactivateIdentityRequested::EVENT
				|| ev.pallet_name() == ActivateIdentityRequested::PALLET
					&& ev.variant_name() == ActivateIdentityRequested::EVENT
				|| ev.pallet_name() == EnclaveUnauthorized::PALLET
					&& ev.variant_name() == EnclaveUnauthorized::EVENT
				|| ev.pallet_name() == OpaqueTaskPosted::PALLET
					&& ev.variant_name() == OpaqueTaskPosted::EVENT
				|| ev.pallet_name() == AssertionCreated::PALLET
					&& ev.variant_name() == AssertionCreated::EVENT
				|| ev.pallet_name() == ParentchainBlockProcessed::PALLET
					&& ev.variant_name() == ParentchainBlockProcessed::EVENT
				|| ev.pallet_name() == EnclaveAdded::PALLET
					&& ev.variant_name() == EnclaveAdded::EVENT
				|| ev.pallet_name() == EnclaveRemoved::PALLET
					&& ev.variant_name() == EnclaveRemoved::EVENT
				|| ev.pallet_name() == AccountStoreUpdated::PALLET
					&& ev.variant_name() == AccountStoreUpdated::EVENT
			{
				interesting_events.push(ev)
			}
		});

		Self { events: interesting_events }
	}

	fn filter<T: StaticEvent, E>(&mut self) -> Result<Vec<T>, E> {
		Ok(self
			.events
			.iter()
			.filter_map(|ev| match ev.as_event::<T>() {
				Ok(maybe_event) => maybe_event,
				Err(e) => {
					log::error!("Could not decode event: {:?}", e);
					None
				},
			})
			.collect())
	}
}

impl From<Events<H256>> for FilterableEvents {
	fn from(ev: Events<H256>) -> Self {
		Self::new(ev)
	}
}

impl FilterEvents for FilterableEvents {
	type Error = itc_parentchain_indirect_calls_executor::Error;

	fn get_link_identity_events(&mut self) -> Result<Vec<LinkIdentityRequested>, Self::Error> {
		self.filter()
	}

	fn get_vc_requested_events(&mut self) -> Result<Vec<VCRequested>, Self::Error> {
		self.filter()
	}

	fn get_deactivate_identity_events(
		&mut self,
	) -> Result<Vec<DeactivateIdentityRequested>, Self::Error> {
		self.filter()
	}

	fn get_activate_identity_events(
		&mut self,
	) -> Result<Vec<ActivateIdentityRequested>, Self::Error> {
		self.filter()
	}

	fn get_enclave_unauthorized_events(&mut self) -> Result<Vec<EnclaveUnauthorized>, Self::Error> {
		self.filter()
	}

	fn get_opaque_task_posted_events(&mut self) -> Result<Vec<OpaqueTaskPosted>, Self::Error> {
		self.filter()
	}

	fn get_assertion_created_events(&mut self) -> Result<Vec<AssertionCreated>, Self::Error> {
		self.filter()
	}

	fn get_parentchain_block_proccessed_events(
		&mut self,
	) -> Result<Vec<ParentchainBlockProcessed>, Self::Error> {
		self.filter()
	}

	fn get_enclave_added_events(&mut self) -> Result<Vec<EnclaveAdded>, Self::Error> {
		self.filter()
	}

	fn get_enclave_removed_events(&mut self) -> Result<Vec<EnclaveRemoved>, Self::Error> {
		self.filter()
	}

	fn get_account_store_updated_events(
		&mut self,
	) -> Result<Vec<AccountStoreUpdated>, Self::Error> {
		self.filter()
	}
}
