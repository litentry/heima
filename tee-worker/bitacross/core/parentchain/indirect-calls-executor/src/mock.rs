use crate::{
	error::{Error, Result as ICResult},
	filter_metadata::EventsFromMetadata,
	IndirectDispatch,
};
use bc_relayer_registry::RelayerRegistry;
use bc_signer_registry::SignerRegistry;
use codec::{Decode, Encode};
use core::marker::PhantomData;
use litentry_primitives::DecryptableRequest;

use bc_enclave_registry::EnclaveRegistry;
use itp_node_api::api_client::{CallIndex, PairSignature, UncheckedExtrinsicV4};
use itp_sgx_runtime_primitives::types::{AccountId, Balance};
use itp_stf_primitives::{traits::IndirectExecutor, types::Signature};
use itp_test::mock::stf_mock::{GetterMock, TrustedCallMock, TrustedCallSignedMock};
use itp_types::{
	parentchain::{events::*, FilterEvents, HandleParentchainEvents},
	Address, RsaRequest, ShardIdentifier, H256,
};
use log::*;
use sp_runtime::traits::{Block as ParentchainBlock, Header as ParentchainHeader};
use std::vec::Vec;

pub struct ExtrinsicParser<SignedExtra> {
	_phantom: PhantomData<SignedExtra>,
}
use itp_api_client_types::ParentchainSignedExtra;
use itp_stf_primitives::types::TrustedOperation;

/// Parses the extrinsics corresponding to the parentchain.
pub type MockParentchainExtrinsicParser = ExtrinsicParser<ParentchainSignedExtra>;

/// Partially interpreted extrinsic containing the `signature` and the `call_index` whereas
/// the `call_args` remain in encoded form.
///
/// Intended for usage, where the actual `call_args` form is unknown.
pub struct SemiOpaqueExtrinsic<'a> {
	/// Signature of the Extrinsic.
	pub signature: Signature,
	/// Call index of the dispatchable.
	pub call_index: CallIndex,
	/// Encoded arguments of the dispatchable corresponding to the `call_index`.
	pub call_args: &'a [u8],
}

/// Trait to extract signature and call indexes of an encoded [UncheckedExtrinsicV4].
pub trait ParseExtrinsic {
	/// Signed extra of the extrinsic.
	type SignedExtra;

	fn parse(encoded_call: &[u8]) -> Result<SemiOpaqueExtrinsic, codec::Error>;
}

impl<SignedExtra> ParseExtrinsic for ExtrinsicParser<SignedExtra>
where
	SignedExtra: Decode + Encode,
{
	type SignedExtra = SignedExtra;

	/// Extract a call index of an encoded call.
	fn parse(encoded_call: &[u8]) -> Result<SemiOpaqueExtrinsic, codec::Error> {
		let call_mut = &mut &encoded_call[..];

		// `()` is a trick to stop decoding after the call index. So the remaining bytes
		//  of `call` after decoding only contain the parentchain's dispatchable's arguments.
		let xt = UncheckedExtrinsicV4::<
            Address,
            (CallIndex, ()),
            PairSignature,
            Self::SignedExtra,
        >::decode(call_mut)?;

		Ok(SemiOpaqueExtrinsic {
			signature: xt.signature.unwrap().1,
			call_index: xt.function.0,
			call_args: call_mut,
		})
	}
}
/// The default indirect call (extrinsic-triggered) of the Integritee-Parachain.
#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq)]
pub enum IndirectCall {
	ShieldFunds(ShieldFundsArgs),
	Invoke(InvokeArgs),
}

impl<
		Executor: IndirectExecutor<
			TrustedCallSignedMock,
			Error,
			RelayerRegistry,
			SignerRegistry,
			EnclaveRegistry,
		>,
	>
	IndirectDispatch<
		Executor,
		TrustedCallSignedMock,
		RelayerRegistry,
		SignerRegistry,
		EnclaveRegistry,
	> for IndirectCall
{
	type Args = ();
	fn dispatch(&self, executor: &Executor, args: Self::Args) -> ICResult<()> {
		trace!("dispatching indirect call {:?}", self);
		match self {
			IndirectCall::ShieldFunds(shieldfunds_args) =>
				shieldfunds_args.dispatch(executor, args),
			IndirectCall::Invoke(invoke_args) => invoke_args.dispatch(executor, args),
		}
	}
}

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

	fn get_link_identity_events(&self) -> Result<Vec<LinkIdentityRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_vc_requested_events(&self) -> Result<Vec<VCRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_deactivate_identity_events(
		&self,
	) -> Result<Vec<DeactivateIdentityRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_activate_identity_events(&self) -> Result<Vec<ActivateIdentityRequested>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_enclave_unauthorized_events(&self) -> Result<Vec<EnclaveUnauthorized>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_opaque_task_posted_events(&self) -> Result<Vec<OpaqueTaskPosted>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_assertion_created_events(&self) -> Result<Vec<AssertionCreated>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_parentchain_block_proccessed_events(
		&self,
	) -> Result<Vec<ParentchainBlockProcessed>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_relayer_added_events(
		&self,
	) -> Result<Vec<itp_types::parentchain::events::RelayerAdded>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_relayers_removed_events(
		&self,
	) -> Result<Vec<itp_types::parentchain::events::RelayerRemoved>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_enclave_added_events(
		&self,
	) -> Result<Vec<itp_types::parentchain::events::EnclaveAdded>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_enclave_removed_events(
		&self,
	) -> Result<Vec<itp_types::parentchain::events::EnclaveRemoved>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_btc_wallet_generated_events(
		&self,
	) -> Result<Vec<itp_types::parentchain::events::BtcWalletGenerated>, Self::Error> {
		Ok(Vec::new())
	}

	fn get_account_store_updated_events(&self) -> Result<Vec<AccountStoreUpdated>, Self::Error> {
		Ok(Vec::new())
	}
}

pub struct MockParentchainEventHandler {}

impl<Executor>
	HandleParentchainEvents<
		Executor,
		TrustedCallSignedMock,
		Error,
		RelayerRegistry,
		SignerRegistry,
		EnclaveRegistry,
	> for MockParentchainEventHandler
where
	Executor: IndirectExecutor<
		TrustedCallSignedMock,
		Error,
		RelayerRegistry,
		SignerRegistry,
		EnclaveRegistry,
	>,
{
	type Output = Vec<H256>;

	fn handle_events<Block>(
		&self,
		_: &Executor,
		_: impl itp_types::parentchain::FilterEvents,
		_block_number: <<Block as ParentchainBlock>::Header as ParentchainHeader>::Number,
	) -> core::result::Result<Vec<H256>, Error>
	where
		Block: ParentchainBlock,
	{
		Ok(Vec::from([H256::default()]))
	}
}

/// Arguments of the Integritee-Parachain's shield fund dispatchable.
#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq)]
pub struct ShieldFundsArgs {
	account_encrypted: Vec<u8>,
	amount: Balance,
	shard: ShardIdentifier,
}

impl<
		Executor: IndirectExecutor<
			TrustedCallSignedMock,
			Error,
			RelayerRegistry,
			SignerRegistry,
			EnclaveRegistry,
		>,
	>
	IndirectDispatch<
		Executor,
		TrustedCallSignedMock,
		RelayerRegistry,
		SignerRegistry,
		EnclaveRegistry,
	> for ShieldFundsArgs
{
	type Args = ();
	fn dispatch(&self, executor: &Executor, _args: Self::Args) -> ICResult<()> {
		info!("Found ShieldFunds extrinsic in block: \nAccount Encrypted {:?} \nAmount: {} \nShard: {}",
        	self.account_encrypted, self.amount, bs58::encode(self.shard.encode()).into_string());

		debug!("decrypt the account id");
		let account_vec = executor.decrypt(&self.account_encrypted)?;
		let _account = AccountId::decode(&mut account_vec.as_slice())?;

		let enclave_account_id = executor.get_enclave_account()?;
		let trusted_call = TrustedCallMock::noop(enclave_account_id.into());
		let signed_trusted_call = executor.sign_call_with_self(&trusted_call, &self.shard)?;
		let trusted_operation =
			TrustedOperation::<TrustedCallSignedMock, GetterMock>::indirect_call(
				signed_trusted_call,
			);

		let encrypted_trusted_call = executor.encrypt(&trusted_operation.encode())?;
		executor.submit_trusted_call(self.shard, encrypted_trusted_call);
		Ok(())
	}
}

#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq)]
pub struct InvokeArgs {
	request: RsaRequest,
}

impl<
		Executor: IndirectExecutor<
			TrustedCallSignedMock,
			Error,
			RelayerRegistry,
			SignerRegistry,
			EnclaveRegistry,
		>,
	>
	IndirectDispatch<
		Executor,
		TrustedCallSignedMock,
		RelayerRegistry,
		SignerRegistry,
		EnclaveRegistry,
	> for InvokeArgs
{
	type Args = ();
	fn dispatch(&self, executor: &Executor, _args: Self::Args) -> ICResult<()> {
		log::debug!("Found trusted call extrinsic, submitting it to the top pool");
		executor.submit_trusted_call(self.request.shard(), self.request.payload().to_vec());
		Ok(())
	}
}
