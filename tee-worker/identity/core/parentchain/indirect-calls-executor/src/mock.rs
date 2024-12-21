use crate::{error::Error, filter_metadata::EventsFromMetadata};
use itp_stf_primitives::traits::IndirectExecutor;
use itp_test::mock::stf_mock::TrustedCallSignedMock;
use itp_types::{
	parentchain::{events::*, FilterEvents, HandleParentchainEvents, ProcessedEventsArtifacts},
	RsaRequest, H256,
};
use sp_core::H160;
use sp_runtime::traits::{Block as ParentchainBlock, Header as ParentchainHeader};
use std::vec::Vec;

pub struct TestEventCreator;

impl<NodeMetadata> EventsFromMetadata<NodeMetadata> for TestEventCreator {
	type Output = MockEvents;

	fn create_from_metadata(
		_metadata: NodeMetadata,
		_block_hash: H256,
		_events: &[u8],
	) -> Option<Self::Output> {
		Some(MockEvents)
	}
}

pub struct MockEvents;

impl FilterEvents for MockEvents {
	type Error = ();

	fn get_opaque_task_posted_events(&mut self) -> Result<Vec<OpaqueTaskPosted>, Self::Error> {
		let opaque_task_posted_event =
			OpaqueTaskPosted { request: RsaRequest::new(H256::default(), Vec::from([0u8; 32])) };
		Ok(Vec::from([opaque_task_posted_event]))
	}

	fn get_link_identity_events(&mut self) -> Result<Vec<LinkIdentityRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_vc_requested_events(&mut self) -> Result<Vec<VCRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_deactivate_identity_events(
		&mut self,
	) -> Result<Vec<DeactivateIdentityRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_activate_identity_events(
		&mut self,
	) -> Result<Vec<ActivateIdentityRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_enclave_unauthorized_events(&mut self) -> Result<Vec<EnclaveUnauthorized>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_assertion_created_events(&mut self) -> Result<Vec<AssertionCreated>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_parentchain_block_proccessed_events(
		&mut self,
	) -> Result<Vec<ParentchainBlockProcessed>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_enclave_added_events(&mut self) -> Result<Vec<EnclaveAdded>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_enclave_removed_events(&mut self) -> Result<Vec<EnclaveRemoved>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_account_store_updated_events(
		&mut self,
	) -> Result<Vec<AccountStoreUpdated>, Self::Error> {
		Ok(Vec::new())
	}
}

pub struct MockParentchainEventHandler {}

impl<Executor> HandleParentchainEvents<Executor, TrustedCallSignedMock, Error, (), (), ()>
	for MockParentchainEventHandler
where
	Executor: IndirectExecutor<TrustedCallSignedMock, Error, (), (), ()>,
{
	type Output = ProcessedEventsArtifacts;
	fn handle_events<Block>(
		&self,
		_: &Executor,
		_: &mut impl FilterEvents,
		_block_number: <<Block as ParentchainBlock>::Header as ParentchainHeader>::Number,
	) -> Result<ProcessedEventsArtifacts, Error>
	where
		Block: ParentchainBlock,
	{
		Ok((
			Vec::from([H256::default()]),
			Vec::from([H160::default()]),
			Vec::from([H160::default()]),
		))
	}
}
