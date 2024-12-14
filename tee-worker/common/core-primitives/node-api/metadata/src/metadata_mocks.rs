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

use crate::{
	error::Result, pallet_balances::BalancesCallIndexes,
	pallet_evm_assertion::EvmAssertionsCallIndexes, pallet_imp::IMPCallIndexes,
	pallet_omni_account::OmniAccountCallIndexes, pallet_proxy::ProxyCallIndexes,
	pallet_system::SystemConstants, pallet_teebag::TeebagCallIndexes,
	pallet_timestamp::TimestampCallIndexes, pallet_utility::UtilityCallIndexes,
	pallet_vcmp::VCMPCallIndexes, runtime_call::RuntimeCall,
};
use codec::{Decode, Encode};

use itp_api_client_types::Metadata;
use sp_version::RuntimeVersion;

impl TryFrom<NodeMetadataMock> for Metadata {
	type Error = ();

	fn try_from(_: NodeMetadataMock) -> core::result::Result<Self, Self::Error> {
		Err(())
	}
}

#[derive(Default, Encode, Decode, Debug, Clone)]
pub struct NodeMetadataMock {
	// litentry
	// teebag
	teebag_module: u8,
	force_add_authorized_enclave: u8,
	force_remove_authorized_enclave: u8,
	register_enclave: u8,
	unregister_enclave: u8,
	register_quoting_enclave: u8,
	register_tcb_info: u8,
	post_opaque_task: u8,
	parentchain_block_processed: u8,
	sidechain_block_imported: u8,
	// IMP
	imp_module: u8,
	imp_link_identity: u8,
	imp_deactivate_identity: u8,
	imp_activate_identity: u8,
	imp_update_id_graph_hash: u8,
	imp_identity_linked: u8,
	imp_identity_deactivated: u8,
	imp_identity_activated: u8,
	imp_identity_networks_set: u8,
	imp_some_error: u8,
	// VCMP
	vcmp_module: u8,
	vcmp_request_vc: u8,
	vcmp_vc_issued: u8,
	vcmp_some_error: u8,
	// EVM Assertion
	evm_assertions_module: u8,
	evm_assertions_store_assertion: u8,
	evm_assertions_void_assertion: u8,

	utility_module: u8,
	utility_batch: u8,
	utility_as_derivative: u8,
	utility_batch_all: u8,
	utility_dispatch_as: u8,
	utility_force_batch: u8,

	proxy_module: u8,
	add_proxy: u8,
	proxy: u8,
	balances_module: u8,
	transfer: u8,
	transfer_keep_alive: u8,
	transfer_allow_death: u8,
	timestamp_module: u8,
	timestamp_set: u8,
	runtime_spec_version: u32,
	runtime_transaction_version: u32,

	omni_account_module: u8,
	dispatch_as_omni_account: u8,
	dispatch_as_signed: u8,
	create_account_store: u8,
	update_account_store_by_one: u8,
}

impl NodeMetadataMock {
	pub fn new() -> Self {
		NodeMetadataMock {
			// litentry
			teebag_module: 50u8,
			force_add_authorized_enclave: 0u8,
			force_remove_authorized_enclave: 1u8,
			register_enclave: 2u8,
			unregister_enclave: 3u8,
			register_quoting_enclave: 4u8,
			register_tcb_info: 5u8,
			post_opaque_task: 6u8,
			parentchain_block_processed: 7u8,
			sidechain_block_imported: 8u8,

			imp_module: 64u8,
			imp_link_identity: 1u8,
			imp_deactivate_identity: 2u8,
			imp_activate_identity: 3u8,
			imp_update_id_graph_hash: 4u8,
			imp_identity_linked: 6u8,
			imp_identity_deactivated: 7u8,
			imp_identity_activated: 8u8,
			imp_identity_networks_set: 9u8,
			imp_some_error: 10u8,

			vcmp_module: 66u8,
			vcmp_request_vc: 0u8,
			vcmp_vc_issued: 3u8,
			vcmp_some_error: 9u8,

			evm_assertions_module: 76u8,
			evm_assertions_store_assertion: 77u8,
			evm_assertions_void_assertion: 78u8,

			utility_module: 80u8,
			utility_batch: 0u8,
			utility_as_derivative: 1u8,
			utility_batch_all: 2u8,
			utility_dispatch_as: 3u8,
			utility_force_batch: 4u8,

			proxy_module: 7u8,
			add_proxy: 1u8,
			proxy: 0u8,
			balances_module: 10u8,
			transfer: 7u8,
			transfer_keep_alive: 3u8,
			transfer_allow_death: 0u8,
			timestamp_module: 3,
			timestamp_set: 0,
			runtime_spec_version: 25,
			runtime_transaction_version: 4,

			omni_account_module: 70u8,
			dispatch_as_omni_account: 0u8,
			dispatch_as_signed: 1u8,
			create_account_store: 2u8,
			update_account_store_by_one: 3u8,
		}
	}
}

impl TeebagCallIndexes for NodeMetadataMock {
	fn force_add_authorized_enclave_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.force_add_authorized_enclave])
	}
	fn force_remove_authorized_enclave_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.force_remove_authorized_enclave])
	}
	fn register_enclave_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.register_enclave])
	}
	fn unregister_enclave_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.unregister_enclave])
	}
	fn register_quoting_enclave_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.register_quoting_enclave])
	}
	fn register_tcb_info_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.register_tcb_info])
	}
	fn post_opaque_task_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.post_opaque_task])
	}
	fn parentchain_block_processed_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.parentchain_block_processed])
	}
	fn sidechain_block_imported_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.teebag_module, self.sidechain_block_imported])
	}
}

impl IMPCallIndexes for NodeMetadataMock {
	fn link_identity_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_link_identity])
	}

	fn deactivate_identity_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_deactivate_identity])
	}

	fn activate_identity_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_activate_identity])
	}

	fn update_id_graph_hash_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_update_id_graph_hash])
	}

	fn identity_linked_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_identity_linked])
	}

	fn identity_deactivated_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_identity_deactivated])
	}

	fn identity_activated_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_identity_activated])
	}

	fn identity_networks_set_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_identity_networks_set])
	}

	fn imp_some_error_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.imp_module, self.imp_some_error])
	}
}

impl VCMPCallIndexes for NodeMetadataMock {
	fn request_vc_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.vcmp_module, self.vcmp_request_vc])
	}

	fn vc_issued_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.vcmp_module, self.vcmp_vc_issued])
	}

	fn vcmp_some_error_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.vcmp_module, self.vcmp_some_error])
	}
}

impl UtilityCallIndexes for NodeMetadataMock {
	fn batch_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.utility_module, self.utility_batch])
	}

	fn as_derivative_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.utility_module, self.utility_as_derivative])
	}

	fn batch_all_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.utility_module, self.utility_batch_all])
	}

	fn dispatch_as_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.utility_module, self.utility_dispatch_as])
	}

	fn force_batch_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.utility_module, self.utility_force_batch])
	}
}

impl RuntimeCall for NodeMetadataMock {
	fn retrieve(&self) -> Result<u32> {
		Err(crate::Error::MetadataNotSet)
	}
}

impl SystemConstants for NodeMetadataMock {
	fn system_ss58_prefix(&self) -> Result<u16> {
		Ok(131)
	}

	fn system_version(&self) -> Result<RuntimeVersion> {
		Ok(RuntimeVersion {
			spec_version: self.runtime_spec_version,
			transaction_version: self.runtime_transaction_version,
			..Default::default()
		})
	}
}

impl ProxyCallIndexes for NodeMetadataMock {
	fn add_proxy_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.proxy_module, self.add_proxy])
	}

	fn proxy_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.proxy_module, self.proxy])
	}
}

impl BalancesCallIndexes for NodeMetadataMock {
	fn transfer_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.balances_module, self.transfer])
	}

	fn transfer_keep_alive_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.balances_module, self.transfer_keep_alive])
	}

	fn transfer_allow_death_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.balances_module, self.transfer_allow_death])
	}
}

impl TimestampCallIndexes for NodeMetadataMock {
	fn timestamp_set_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.timestamp_module, self.timestamp_set])
	}
}

impl EvmAssertionsCallIndexes for NodeMetadataMock {
	fn store_assertion_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.evm_assertions_module, self.evm_assertions_store_assertion])
	}

	fn void_assertion_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.evm_assertions_module, self.evm_assertions_void_assertion])
	}
}

impl OmniAccountCallIndexes for NodeMetadataMock {
	fn dispatch_as_omni_account_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.omni_account_module, self.dispatch_as_omni_account])
	}

	fn dispatch_as_signed_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.omni_account_module, self.dispatch_as_signed])
	}

	fn create_account_store_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.omni_account_module, self.create_account_store])
	}

	fn update_account_store_by_one_call_indexes(&self) -> Result<[u8; 2]> {
		Ok([self.omni_account_module, self.update_account_store_by_one])
	}
}
