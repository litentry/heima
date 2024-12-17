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

#[cfg(feature = "evm")]
use sp_core::{H160, U256};

#[cfg(feature = "evm")]
use crate::evm_helpers::{create_code_hash, evm_create2_address, evm_create_address};
use crate::{
	format,
	helpers::{enclave_signer_account, ensure_enclave_signer_account, ensure_self},
	trusted_call_result::{
		ActivateIdentityResult, DeactivateIdentityResult, SetIdentityNetworksResult,
		TrustedCallResult,
	},
	Arc, Getter, String, ToString, Vec,
};
use codec::{Decode, Encode};
use frame_support::{ensure, traits::UnfilteredDispatchable};
#[cfg(feature = "evm")]
use ita_sgx_runtime::{AddressMapping, HashedAddressMapping};
pub use ita_sgx_runtime::{
	Balance, IDGraph, Index, ParentchainInstanceLitentry, ParentchainInstanceTargetA,
	ParentchainInstanceTargetB, ParentchainLitentry, Runtime, System, VERSION as SIDECHAIN_VERSION,
};
use itp_node_api::metadata::{provider::AccessNodeMetadata, NodeMetadataTrait};
use itp_node_api_metadata::{pallet_imp::IMPCallIndexes, pallet_vcmp::VCMPCallIndexes};
use itp_stf_interface::ExecuteCall;
use itp_stf_primitives::{
	error::StfError,
	traits::{TrustedCallSigning, TrustedCallVerification},
	types::{AccountId, KeyPair, ShardIdentifier, TrustedOperation},
};
use itp_types::{
	parentchain::{ParentchainCall, ParentchainId},
	Moment, OpaqueCall, H256,
};
use itp_utils::stringify::account_id_to_string;
use litentry_hex_utils::hex_encode;
pub use litentry_primitives::{
	aes_encrypt_default, all_evm_web3networks, all_substrate_web3networks, AesOutput, Assertion,
	ErrorDetail, IMPError, Identity, Intent, LitentryMultiSignature, OmniAccountPermission,
	ParentchainBlockNumber, RequestAesKey, VCMPError, ValidationData, Web3Network,
};
use log::*;
use sp_core::{
	crypto::{AccountId32, UncheckedFrom},
	ed25519,
};
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::ConstU32, BoundedVec, MultiAddress};

pub type IMTCall = ita_sgx_runtime::IdentityManagementCall<Runtime>;
pub type IMT = ita_sgx_runtime::pallet_identity_management_tee::Pallet<Runtime>;
pub type MaxAssertionLength = ConstU32<128>;
pub type VecAssertion = BoundedVec<Assertion, MaxAssertionLength>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TrustedCall {
	/// litentry trusted calls
	/// the calls that should deliver a result other than `Empty` will need to include the parameter: `Option<RequestAesKey>`,
	/// it's a 32-byte AES key defined by the client. This key will be used to encrypt the user-sensitive result in the DI response,
	/// see `trusted_call_result.rs`.
	///
	/// It's an Option because for II call there's no need to define such a key.
	///
	/// Theoretically, this key **could** be different from what is used to encrypt the `AesRequest` payload, but in practice it's fine
	/// to simply use the same key.
	///
	/// Please note this key needs to be embeded in the trusted call itself because:
	/// - it needs to be passed around in async handling of trusted call
	/// - for multi-worker setup, the worker that processes the request can be differnet from the worker that receives the request, so
	///   we can't maintain something like a global mapping between trusted call and aes-key, which only resides in the memory of one worker.
	#[codec(index = 0)]
	link_identity(
		Identity,
		Identity,
		Identity,
		ValidationData,
		Vec<Web3Network>,
		Option<RequestAesKey>,
		H256,
	),
	#[codec(index = 1)]
	deactivate_identity(Identity, Identity, Identity, Option<RequestAesKey>, H256),
	#[codec(index = 2)]
	activate_identity(Identity, Identity, Identity, Option<RequestAesKey>, H256),
	#[codec(index = 3)]
	request_vc(Identity, Identity, Assertion, Option<RequestAesKey>, H256),
	#[codec(index = 4)]
	set_identity_networks(
		Identity,
		Identity,
		Identity,
		Vec<Web3Network>,
		Option<RequestAesKey>,
		H256,
	),
	#[cfg(feature = "development")]
	#[codec(index = 5)]
	remove_identity(Identity, Identity, Vec<Identity>),
	#[codec(index = 6)]
	request_batch_vc(Identity, Identity, VecAssertion, Option<RequestAesKey>, H256),
	// the following trusted calls should not be requested directly from external
	// they are guarded by the signature check (either root or enclave_signer_account)
	// starting from index 20 to leave some room for future "normal" trusted calls
	#[codec(index = 20)]
	link_identity_callback(
		Identity,
		Identity,
		Identity,
		Vec<Web3Network>,
		Option<RequestAesKey>,
		H256,
	),
	#[codec(index = 21)]
	handle_imp_error(Identity, Option<Identity>, IMPError, H256),
	#[codec(index = 22)]
	handle_vcmp_error(Identity, Option<Identity>, VCMPError, H256),
	#[codec(index = 23)]
	send_erroneous_parentchain_call(Identity),
	#[codec(index = 24)]
	maybe_create_id_graph(Identity, Identity),
	#[cfg(feature = "development")]
	#[codec(index = 25)]
	clean_id_graphs(Identity),
	#[codec(index = 26)]
	request_intent(Identity, Intent),
	#[codec(index = 27)]
	create_account_store(Identity),
	#[codec(index = 28)]
	add_account(Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>),
	#[codec(index = 29)]
	remove_accounts(Identity, Vec<Identity>),
	#[codec(index = 30)]
	publicize_account(Identity, Identity),
	#[codec(index = 31)]
	set_permissions(Identity, Identity, Vec<OmniAccountPermission>),

	// original integritee trusted calls, starting from index 50
	#[codec(index = 50)]
	noop(Identity),
	#[codec(index = 51)]
	balance_set_balance(Identity, AccountId, Balance, Balance),
	#[codec(index = 52)]
	balance_transfer(Identity, AccountId, Balance),
	#[codec(index = 55)]
	timestamp_set(Identity, Moment, ParentchainId),
	#[cfg(feature = "evm")]
	#[codec(index = 56)]
	evm_withdraw(Identity, H160, Balance), // (Origin, Address EVM Account, Value)
	// (Origin, Source, Target, Input, Value, Gas limit, Max fee per gas, Max priority fee per gas, Nonce, Access list)
	#[cfg(feature = "evm")]
	#[codec(index = 57)]
	evm_call(
		Identity,
		H160,
		H160,
		Vec<u8>,
		U256,
		u64,
		U256,
		Option<U256>,
		Option<U256>,
		Vec<(H160, Vec<H256>)>,
	),
	// (Origin, Source, Init, Value, Gas limit, Max fee per gas, Max priority fee per gas, Nonce, Access list)
	#[cfg(feature = "evm")]
	#[codec(index = 58)]
	evm_create(
		Identity,
		H160,
		Vec<u8>,
		U256,
		u64,
		U256,
		Option<U256>,
		Option<U256>,
		Vec<(H160, Vec<H256>)>,
	),
	// (Origin, Source, Init, Salt, Value, Gas limit, Max fee per gas, Max priority fee per gas, Nonce, Access list)
	#[cfg(feature = "evm")]
	#[codec(index = 59)]
	evm_create2(
		Identity,
		H160,
		Vec<u8>,
		H256,
		U256,
		u64,
		U256,
		Option<U256>,
		Option<U256>,
		Vec<(H160, Vec<H256>)>,
	),
}

impl TrustedCall {
	pub fn sender_identity(&self) -> &Identity {
		match self {
			Self::noop(sender_identity) => sender_identity,
			Self::balance_set_balance(sender_identity, ..) => sender_identity,
			Self::balance_transfer(sender_identity, ..) => sender_identity,
			Self::timestamp_set(sender_identity, ..) => sender_identity,
			#[cfg(feature = "evm")]
			Self::evm_withdraw(sender_identity, ..) => sender_identity,
			#[cfg(feature = "evm")]
			Self::evm_call(sender_identity, ..) => sender_identity,
			#[cfg(feature = "evm")]
			Self::evm_create(sender_identity, ..) => sender_identity,
			#[cfg(feature = "evm")]
			Self::evm_create2(sender_identity, ..) => sender_identity,
			// litentry
			Self::link_identity(sender_identity, ..) => sender_identity,
			Self::deactivate_identity(sender_identity, ..) => sender_identity,
			Self::activate_identity(sender_identity, ..) => sender_identity,
			Self::request_vc(sender_identity, ..) => sender_identity,
			Self::set_identity_networks(sender_identity, ..) => sender_identity,
			Self::link_identity_callback(sender_identity, ..) => sender_identity,
			Self::handle_imp_error(sender_identity, ..) => sender_identity,
			Self::handle_vcmp_error(sender_identity, ..) => sender_identity,
			Self::send_erroneous_parentchain_call(sender_identity) => sender_identity,
			Self::maybe_create_id_graph(sender_identity, ..) => sender_identity,
			#[cfg(feature = "development")]
			Self::remove_identity(sender_identity, ..) => sender_identity,
			Self::request_batch_vc(sender_identity, ..) => sender_identity,
			#[cfg(feature = "development")]
			Self::clean_id_graphs(sender_identity) => sender_identity,
			Self::request_intent(sender_identity, ..) => sender_identity,
			Self::create_account_store(sender_identity) => sender_identity,
			Self::add_account(sender_identity, ..) => sender_identity,
			Self::remove_accounts(sender_identity, ..) => sender_identity,
			Self::publicize_account(sender_identity, ..) => sender_identity,
			Self::set_permissions(sender_identity, ..) => sender_identity,
		}
	}

	pub fn metric_name(&self) -> &'static str {
		match self {
			Self::link_identity(..) => "link_identity",
			Self::request_vc(..) => "request_vc",
			Self::link_identity_callback(..) => "link_identity_callback",
			Self::handle_vcmp_error(..) => "handle_vcmp_error",
			Self::handle_imp_error(..) => "handle_imp_error",
			Self::deactivate_identity(..) => "deactivate_identity",
			Self::activate_identity(..) => "activate_identity",
			Self::maybe_create_id_graph(..) => "maybe_create_id_graph",
			Self::request_intent(..) => "request_intent",
			Self::create_account_store(..) => "create_account_store",
			Self::add_account(..) => "add_account",
			Self::remove_accounts(..) => "remove_account",
			Self::publicize_account(..) => "publicize_account",
			Self::set_permissions(..) => "set_permissions",
			_ => "unsupported_trusted_call",
		}
	}

	pub fn signature_message_prefix(&self) -> String {
		match self {
			Self::link_identity(..) => "By linking your identity to our platform, you're taking a step towards a more integrated experience. Please be assured, this process is safe and involves no transactions of your assets. Token: ".to_string(),
			Self::request_batch_vc(_, _, assertions, ..) =>  match assertions.len() {
				1 => "We are going to help you generate 1 secure credential. Please be assured, this process is safe and involves no transactions of your assets. Token: ".to_string(),
				n => format!("We are going to help you generate {n} secure credentials. Please be assured, this process is safe and involves no transactions of your assets. Token: "),
			},
			_ => "Token: ".to_string(),
		}
	}
}

impl TrustedCallSigning<TrustedCallSigned> for TrustedCall {
	fn sign(
		&self,
		pair: &KeyPair,
		nonce: Index,
		mrenclave: &[u8; 32],
		shard: &ShardIdentifier,
	) -> TrustedCallSigned {
		let mut payload = self.encode();
		payload.append(&mut nonce.encode());
		payload.append(&mut mrenclave.encode());
		payload.append(&mut shard.encode());

		// use blake2_256 hash to shorten the payload - see `verify_signature` below
		TrustedCallSigned { call: self.clone(), nonce, signature: pair.sign(&blake2_256(&payload)) }
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TrustedCallSigned {
	pub call: TrustedCall,
	pub nonce: Index,
	pub signature: LitentryMultiSignature,
}

impl TrustedCallSigned {
	pub fn new(call: TrustedCall, nonce: Index, signature: LitentryMultiSignature) -> Self {
		TrustedCallSigned { call, nonce, signature }
	}

	pub fn into_trusted_operation(
		self,
		direct: bool,
	) -> TrustedOperation<TrustedCallSigned, Getter> {
		match direct {
			true => TrustedOperation::direct_call(self),
			false => TrustedOperation::indirect_call(self),
		}
	}
}

impl Default for TrustedCallSigned {
	fn default() -> Self {
		Self {
			call: TrustedCall::noop(AccountId32::unchecked_from([0u8; 32].into()).into()),
			nonce: 0,
			signature: LitentryMultiSignature::Ed25519(ed25519::Signature::unchecked_from(
				[0u8; 64],
			)),
		}
	}
}
impl TrustedCallVerification for TrustedCallSigned {
	fn sender_identity(&self) -> &Identity {
		self.call.sender_identity()
	}

	fn nonce(&self) -> Index {
		self.nonce
	}

	fn verify_signature(&self, mrenclave: &[u8; 32], shard: &ShardIdentifier) -> bool {
		let mut payload = self.call.encode();
		payload.append(&mut self.nonce.encode());
		payload.append(&mut mrenclave.encode());
		payload.append(&mut shard.encode());

		// The signature should be valid in either case:
		// 1. blake2_256(payload)
		// 2. Signature Prefix + blake2_256(payload)

		let hashed = blake2_256(&payload);

		let prettified_msg_hash = self.call.signature_message_prefix() + &hex_encode(&hashed);
		let prettified_msg_hash = prettified_msg_hash.as_bytes();

		// Most common signatures variants by clients are verified first (4 and 2).
		self.signature.verify(prettified_msg_hash, self.call.sender_identity())
			|| self.signature.verify(&hashed, self.call.sender_identity())
	}

	fn metric_name(&self) -> &'static str {
		self.call.metric_name()
	}
}

impl<NodeMetadataRepository> ExecuteCall<NodeMetadataRepository> for TrustedCallSigned
where
	NodeMetadataRepository: AccessNodeMetadata,
	NodeMetadataRepository::MetadataType: NodeMetadataTrait,
{
	type Error = StfError;
	type Result = TrustedCallResult;

	// TODO(Kai@litentry):
	// If this function returns Err(), it will feed the executor with Ok(ExecutedOperation::failed()),
	// which will remove the failed op from its **own** top pool while preventing it from being included
	// in a sidechain block - see `execute_trusted_call_on_stf`.
	//
	// As a result, when other workers import sidechain blocks, they will treat the op as
	// "not yet executed" (before it's not recorded in the sidechain block) and try to execute it again from
	// its own top pool (if the op is added to the top pool upon e.g. parentchain block import).
	//
	// The execution will most likely fail again. However, the state could have been changed already by applying
	// the state diff from the imported sidechain block. This could cause an inconsistent/mismatching state,
	// for example, the nonce. See the nonce handling below: we increased the nonce no matter the STF is executed
	// successfully or not.
	//
	// This is probably the reason why the nonce-handling test in `demo_shielding_unshielding.sh` sometimes fails.
	//
	// Update:
	// see discussion in https://github.com/integritee-network/worker/issues/1232
	// my current thoughts are:
	// - we should return Err() if the STF execution fails, the parentchain effect will get applied regardless
	// - the failed top should be removed from the pool
	// - however, the failed top hash needs to be included in the sidechain block (still TODO)
	//
	// Almost every (Litentry) trusted call has a `H256` as parameter, this is used as the request identifier.
	// It should be generated by the client (requester), and checked against when getting the response.
	// It might seem redundant for direct invocation (DI) as the response is synchronous, however, we do need it
	// when the request is handled asynchronously interanlly, which leads to streamed responses. Without it, it's
	// impossible to pair the request and response. `top_hash` won't suffice as you can't know all hashes from
	// client side beforehand (e.g. those trusted calls signed by enclave signer).
	//
	// TODO:
	// - shall we add `req_ext_hash` in RpcReturnValue and use it to find streamed trustedCalls?
	// - show error details for "Invalid" synchronous responses
	fn execute(
		self,
		shard: &ShardIdentifier,
		top_hash: H256,
		calls: &mut Vec<ParentchainCall>,
		node_metadata_repo: Arc<NodeMetadataRepository>,
	) -> Result<Self::Result, Self::Error> {
		let sender = self.call.sender_identity().clone();
		let account_id: AccountId =
			sender.to_native_account().ok_or(Self::Error::InvalidAccount)?;
		let system_nonce = System::account_nonce(&account_id);
		ensure!(self.nonce == system_nonce, Self::Error::InvalidNonce(self.nonce, system_nonce));

		// Increment the nonce no matter if the call succeeds or fails.
		// We consider the call "valid" once it reaches here (= it entered the tx pool)
		System::inc_account_nonce(&account_id);

		// TODO: maybe we can further simplify this by effacing the duplicate code
		match self.call {
			TrustedCall::noop(who) => {
				debug!("noop called by {}", account_id_to_string(&who),);
				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::balance_set_balance(root, who, free_balance, reserved_balance) => {
				let root_account_id: AccountId =
					root.to_native_account().ok_or(Self::Error::InvalidAccount)?;
				ensure!(
					is_root::<Runtime, AccountId>(&root_account_id),
					Self::Error::MissingPrivileges(root_account_id)
				);
				debug!(
					"balance_set_balance({}, {}, {})",
					account_id_to_string(&who),
					free_balance,
					reserved_balance
				);
				ita_sgx_runtime::BalancesCall::<Runtime>::force_set_balance {
					who: MultiAddress::Id(who),
					new_free: free_balance,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::root())
				.map_err(|e| {
					Self::Error::Dispatch(format!("Balance Set Balance error: {:?}", e.error))
				})?;
				// This explicit Error type is somehow still needed, otherwise the compiler complains
				// 	multiple `impl`s satisfying `StfError: std::convert::From<_>`
				// 		note: and another `impl` found in the `core` crate: `impl<T> std::convert::From<T> for T;`
				// the impl From<..> for StfError conflicts with the standard convert
				//
				// Alternatively, removing the customised "impl From<..> for StfError" and use map_err directly
				// would also work
				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::balance_transfer(from, to, value) => {
				let origin = ita_sgx_runtime::RuntimeOrigin::signed(
					from.to_native_account().ok_or(Self::Error::InvalidAccount)?,
				);
				std::println!("⣿STF⣿ 🔄 balance_transfer from ⣿⣿⣿ to ⣿⣿⣿ amount ⣿⣿⣿");
				// endow fee to enclave (self)
				let fee_recipient: AccountId = enclave_signer_account();
				// fixme: apply fees through standard frame process and tune it
				let fee = crate::STF_TX_FEE;
				info!(
					"from {}, to {}, amount {}, fee {}",
					account_id_to_string(&from),
					account_id_to_string(&to),
					value,
					fee
				);
				ita_sgx_runtime::BalancesCall::<Runtime>::transfer {
					dest: MultiAddress::Id(fee_recipient),
					value: fee,
				}
				.dispatch_bypass_filter(origin.clone())
				.map_err(|e| {
					Self::Error::Dispatch(format!("Balance Transfer error: {:?}", e.error))
				})?;
				ita_sgx_runtime::BalancesCall::<Runtime>::transfer {
					dest: MultiAddress::Id(to),
					value,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!("Balance Transfer error: {:?}", e.error))
				})?;
				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::timestamp_set(enclave_account, now, parentchain_id) => {
				let account_id: AccountId32 =
					enclave_account.to_native_account().ok_or(Self::Error::InvalidAccount)?;
				ensure_enclave_signer_account(&account_id)?;
				// Litentry: we don't actually set the timestamp, see `BlockMetadata`
				warn!("unused timestamp_set({}, {:?})", now, parentchain_id);
				Ok(TrustedCallResult::Empty)
			},

			#[cfg(feature = "evm")]
			TrustedCall::evm_withdraw(from, address, value) => {
				debug!("evm_withdraw({}, {}, {})", account_id_to_string(&from), address, value);
				ita_sgx_runtime::EvmCall::<Runtime>::withdraw { address, value }
					.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::signed(
						from.to_native_account().ok_or(Self::Error::InvalidAccount)?,
					))
					.map_err(|e| {
						Self::Error::Dispatch(format!("Evm Withdraw error: {:?}", e.error))
					})?;
				Ok(TrustedCallResult::Empty)
			},
			#[cfg(feature = "evm")]
			TrustedCall::evm_call(
				from,
				source,
				target,
				input,
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
			) => {
				debug!(
					"evm_call(from: {}, source: {}, target: {})",
					account_id_to_string(&from),
					source,
					target
				);
				ita_sgx_runtime::EvmCall::<Runtime>::call {
					source,
					target,
					input,
					value,
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					access_list,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::signed(
					from.to_native_account().ok_or(Self::Error::InvalidAccount)?,
				))
				.map_err(|e| Self::Error::Dispatch(format!("Evm Call error: {:?}", e.error)))?;
				Ok(TrustedCallResult::Empty)
			},
			#[cfg(feature = "evm")]
			TrustedCall::evm_create(
				from,
				source,
				init,
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
			) => {
				debug!(
					"evm_create(from: {}, source: {}, value: {})",
					account_id_to_string(&from),
					source,
					value
				);
				let nonce_evm_account =
					System::account_nonce(&HashedAddressMapping::into_account_id(source));
				ita_sgx_runtime::EvmCall::<Runtime>::create {
					source,
					init,
					value,
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					access_list,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::signed(
					from.to_native_account().ok_or(Self::Error::InvalidAccount)?,
				))
				.map_err(|e| Self::Error::Dispatch(format!("Evm Create error: {:?}", e.error)))?;
				let contract_address = evm_create_address(source, nonce_evm_account);
				info!("Trying to create evm contract with address {:?}", contract_address);
				Ok(TrustedCallResult::Empty)
			},
			#[cfg(feature = "evm")]
			TrustedCall::evm_create2(
				from,
				source,
				init,
				salt,
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
			) => {
				debug!(
					"evm_create2(from: {}, source: {}, value: {})",
					account_id_to_string(&from),
					source,
					value
				);
				let code_hash = create_code_hash(&init);
				ita_sgx_runtime::EvmCall::<Runtime>::create2 {
					source,
					init,
					salt,
					value,
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					access_list,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::signed(
					from.to_native_account().ok_or(Self::Error::InvalidAccount)?,
				))
				.map_err(|e| Self::Error::Dispatch(format!("Evm Create2 error: {:?}", e.error)))?;
				let contract_address = evm_create2_address(source, salt, code_hash);
				info!("Trying to create evm contract with address {:?}", contract_address);
				Ok(TrustedCallResult::Empty)
			},
			// Litentry trusted calls
			// the reason that most calls have an internal handling fn is that we want to capture the error and
			// handle it here to be able to send error events to the parachain
			TrustedCall::link_identity(
				signer,
				who,
				identity,
				validation_data,
				web3networks,
				maybe_key,
				req_ext_hash,
			) => {
				debug!("link_identity, who: {}", account_id_to_string(&who));
				let verification_done = Self::link_identity_internal(
					shard,
					signer.to_native_account().ok_or(Self::Error::InvalidAccount)?,
					who.clone(),
					identity.clone(),
					validation_data,
					web3networks.clone(),
					top_hash,
					maybe_key,
					req_ext_hash,
				)
				.map_err(|e| {
					push_call_imp_some_error(
						calls,
						node_metadata_repo.clone(),
						Some(who.clone()),
						e.to_imp_error(),
						req_ext_hash,
					);
					e
				})?;

				if verification_done {
					Self::handle_link_identity_callback(
						calls,
						node_metadata_repo,
						enclave_signer_account::<AccountId>().into(),
						who,
						identity,
						web3networks,
						maybe_key,
						req_ext_hash,
					)
				} else {
					Ok(TrustedCallResult::Streamed)
				}
			},
			#[cfg(feature = "development")]
			TrustedCall::remove_identity(signer, who, identities) => {
				debug!("remove_identity, who: {}", account_id_to_string(&who));

				let account = signer.to_native_account().ok_or(Self::Error::InvalidAccount)?;
				use crate::helpers::ensure_enclave_signer_or_alice;
				ensure!(
					ensure_enclave_signer_or_alice(&account),
					StfError::RemoveIdentityFailed(ErrorDetail::UnauthorizedSigner)
				);

				IMTCall::remove_identity { who, identities }
					.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::root())
					.map_err(|e| StfError::RemoveIdentityFailed(e.into()))?;

				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::deactivate_identity(signer, who, identity, maybe_key, req_ext_hash) => {
				debug!("deactivate_identity, who: {}", account_id_to_string(&who));
				let call_index = node_metadata_repo
					.get_from_metadata(|m| m.identity_deactivated_call_indexes())??;

				let old_id_graph = IMT::id_graph(&who);
				Self::deactivate_identity_internal(
					signer.to_native_account().ok_or(Self::Error::InvalidAccount)?,
					who.clone(),
					identity,
				)
				.map_err(|e| {
					debug!("pushing error event ... error: {}", e);
					push_call_imp_some_error(
						calls,
						node_metadata_repo.clone(),
						Some(who.clone()),
						e.to_imp_error(),
						req_ext_hash,
					);
					e
				})?;

				let id_graph_hash: H256 = IMT::id_graph_hash(&who).ok_or(StfError::EmptyIDGraph)?;

				debug!("pushing identity_deactivated event ...");
				calls.push(ParentchainCall::Litentry(OpaqueCall::from_tuple(&(
					call_index,
					who.clone(),
					id_graph_hash,
					req_ext_hash,
				))));

				let mut mutated_id_graph = IMT::id_graph(&who);
				mutated_id_graph.retain(|i| !old_id_graph.contains(i));

				if let Some(key) = maybe_key {
					return Ok(TrustedCallResult::DeactivateIdentity(DeactivateIdentityResult {
						mutated_id_graph: aes_encrypt_default(&key, &mutated_id_graph.encode()),
						id_graph_hash,
					}))
				}

				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::activate_identity(signer, who, identity, maybe_key, req_ext_hash) => {
				debug!("activate_identity, who: {}", account_id_to_string(&who));
				let call_index = node_metadata_repo
					.get_from_metadata(|m| m.identity_activated_call_indexes())??;
				let old_id_graph = IMT::id_graph(&who);

				Self::activate_identity_internal(
					signer.to_native_account().ok_or(Self::Error::InvalidAccount)?,
					who.clone(),
					identity,
				)
				.map_err(|e| {
					debug!("pushing error event ... error: {}", e);
					push_call_imp_some_error(
						calls,
						node_metadata_repo.clone(),
						Some(who.clone()),
						e.to_imp_error(),
						req_ext_hash,
					);
					e
				})?;

				let id_graph_hash: H256 = IMT::id_graph_hash(&who).ok_or(StfError::EmptyIDGraph)?;

				debug!("pushing identity_activated event ...");
				calls.push(ParentchainCall::Litentry(OpaqueCall::from_tuple(&(
					call_index,
					who.clone(),
					id_graph_hash,
					req_ext_hash,
				))));

				let mut mutated_id_graph = IMT::id_graph(&who);
				mutated_id_graph.retain(|i| !old_id_graph.contains(i));

				if let Some(key) = maybe_key {
					return Ok(TrustedCallResult::ActivateIdentity(ActivateIdentityResult {
						mutated_id_graph: aes_encrypt_default(&key, &mutated_id_graph.encode()),
						id_graph_hash,
					}))
				}

				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::link_identity_callback(
				signer,
				who,
				identity,
				web3networks,
				maybe_key,
				req_ext_hash,
			) => Self::handle_link_identity_callback(
				calls,
				node_metadata_repo,
				signer,
				who,
				identity,
				web3networks,
				maybe_key,
				req_ext_hash,
			),
			TrustedCall::request_vc(_signer, _who, _assertion, _maybe_key, _req_ext_hash) => {
				error!("deprecated, please use author_requestVc instead");
				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::request_batch_vc(..) => {
				error!("deprecated, please use author_requestVc instead");
				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::set_identity_networks(
				signer,
				who,
				identity,
				web3networks,
				maybe_key,
				req_ext_hash,
			) => {
				debug!("set_identity_networks, networks: {:?}", web3networks);
				// only support DI requests from the signer but we leave the room for changes
				ensure!(
					ensure_self(&signer, &who),
					Self::Error::Dispatch("Unauthorized signer".to_string())
				);
				let call_index = node_metadata_repo
					.get_from_metadata(|m| m.identity_networks_set_call_indexes())??;
				let old_id_graph = IMT::id_graph(&who);

				IMTCall::set_identity_networks { who: who.clone(), identity, web3networks }
					.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::root())
					.map_err(|e| Self::Error::Dispatch(format!(" error: {:?}", e.error)))?;

				let id_graph_hash: H256 = IMT::id_graph_hash(&who).ok_or(StfError::EmptyIDGraph)?;

				debug!("pushing identity_networks_set event ...");
				calls.push(ParentchainCall::Litentry(OpaqueCall::from_tuple(&(
					call_index,
					who.clone(),
					id_graph_hash,
					req_ext_hash,
				))));

				let mut mutated_id_graph = IMT::id_graph(&who);
				mutated_id_graph.retain(|i| !old_id_graph.contains(i));

				if let Some(key) = maybe_key {
					return Ok(TrustedCallResult::SetIdentityNetworks(SetIdentityNetworksResult {
						mutated_id_graph: aes_encrypt_default(&key, &mutated_id_graph.encode()),
						id_graph_hash,
					}))
				}

				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::handle_imp_error(_enclave_account, identity, e, req_ext_hash) => {
				// checking of `_enclave_account` is not strictly needed, as this trusted call can
				// only be constructed internally
				push_call_imp_some_error(
					calls,
					node_metadata_repo,
					identity,
					e.clone(),
					req_ext_hash,
				);
				Err(e.into())
			},
			TrustedCall::handle_vcmp_error(_enclave_account, identity, e, req_ext_hash) => {
				// checking of `_enclave_account` is not strictly needed, as this trusted call can
				// only be constructed internally
				push_call_vcmp_some_error(
					calls,
					node_metadata_repo,
					identity,
					e.clone(),
					req_ext_hash,
				);
				Err(e.into())
			},
			TrustedCall::send_erroneous_parentchain_call(account) => {
				// intentionally send wrong parameters, only used in testing
				calls.push(ParentchainCall::Litentry(OpaqueCall::from_tuple(&(
					node_metadata_repo.get_from_metadata(|m| m.imp_some_error_call_indexes())??,
					"wrong_param".as_bytes(),
					account.encode(),
				))));
				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::maybe_create_id_graph(signer, who) => {
				debug!("maybe_create_id_graph, who: {:?}", who);
				let signer_account: AccountId32 =
					signer.to_native_account().ok_or(Self::Error::InvalidAccount)?;
				ensure_enclave_signer_account(&signer_account)?;

				// we only log the error
				match IMT::maybe_create_id_graph(&who) {
					Ok(()) => info!("maybe_create_id_graph OK"),
					Err(e) => warn!("maybe_create_id_graph NOK: {:?}", e),
				};

				Ok(TrustedCallResult::Empty)
			},
			#[cfg(feature = "development")]
			TrustedCall::clean_id_graphs(signer) => {
				debug!("clean_id_graphs");

				let account = signer.to_native_account().ok_or(Self::Error::InvalidAccount)?;
				use crate::helpers::ensure_enclave_signer_or_alice;
				ensure!(
					ensure_enclave_signer_or_alice(&account),
					StfError::CleanIDGraphsFailed(ErrorDetail::UnauthorizedSigner)
				);

				IMTCall::clean_id_graphs {}
					.dispatch_bypass_filter(ita_sgx_runtime::RuntimeOrigin::root())
					.map_err(|e| StfError::CleanIDGraphsFailed(e.into()))?;

				Ok(TrustedCallResult::Empty)
			},
			TrustedCall::request_intent(..)
			| TrustedCall::create_account_store(..)
			| TrustedCall::add_account(..)
			| TrustedCall::remove_accounts(..)
			| TrustedCall::publicize_account(..)
			| TrustedCall::set_permissions(..) => {
				error!("please use author_submitNativeRequest instead");
				Ok(TrustedCallResult::Empty)
			},
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		debug!("No storage updates needed...");
		Vec::new()
	}
}

pub(crate) fn is_root<Runtime, AccountId>(account: &AccountId) -> bool
where
	Runtime: frame_system::Config<AccountId = AccountId> + pallet_sudo::Config,
	AccountId: PartialEq,
{
	pallet_sudo::Pallet::<Runtime>::key().map_or(false, |k| account == &k)
}

pub fn push_call_imp_some_error<NodeMetadataRepository>(
	calls: &mut Vec<ParentchainCall>,
	node_metadata_repo: Arc<NodeMetadataRepository>,
	identity: Option<Identity>,
	e: IMPError,
	req_ext_hash: H256,
) where
	NodeMetadataRepository: AccessNodeMetadata,
	NodeMetadataRepository::MetadataType: NodeMetadataTrait,
{
	debug!("pushing IMP::some_error call ...");
	// TODO: anyway to simplify this? `and_then` won't be applicable here
	match node_metadata_repo.get_from_metadata(|m| m.imp_some_error_call_indexes()) {
		Ok(Ok(call_index)) => calls.push(ParentchainCall::Litentry(OpaqueCall::from_tuple(&(
			call_index,
			identity,
			e,
			req_ext_hash,
		)))),
		Ok(e) => warn!("error getting IMP::some_error call indexes: {:?}", e),
		Err(e) => warn!("error getting IMP::some_error call indexes: {:?}", e),
	}
}

pub fn push_call_vcmp_some_error<NodeMetadataRepository>(
	calls: &mut Vec<ParentchainCall>,
	node_metadata_repo: Arc<NodeMetadataRepository>,
	identity: Option<Identity>,
	e: VCMPError,
	req_ext_hash: H256,
) where
	NodeMetadataRepository: AccessNodeMetadata,
	NodeMetadataRepository::MetadataType: NodeMetadataTrait,
{
	debug!("pushing VCMP::some_error call ...");
	match node_metadata_repo.get_from_metadata(|m| m.vcmp_some_error_call_indexes()) {
		Ok(Ok(call_index)) => calls.push(ParentchainCall::Litentry(OpaqueCall::from_tuple(&(
			call_index,
			identity,
			e,
			req_ext_hash,
		)))),
		Ok(e) => warn!("error getting VCMP::some_error call indexes: {:?}", e),
		Err(e) => warn!("error getting VCMP::some_error call indexes: {:?}", e),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use itp_stf_primitives::types::KeyPair;
	use sp_keyring::AccountKeyring;

	#[test]
	fn verify_signature_works() {
		let nonce = 21;
		let mrenclave = [0u8; 32];
		let shard = ShardIdentifier::default();

		let call = TrustedCall::balance_set_balance(
			AccountKeyring::Alice.public().into(),
			AccountKeyring::Alice.public().into(),
			42,
			42,
		);
		let signed_call = call.sign(
			&KeyPair::Sr25519(Box::new(AccountKeyring::Alice.pair())),
			nonce,
			&mrenclave,
			&shard,
		);

		assert!(signed_call.verify_signature(&mrenclave, &shard));
	}
}
