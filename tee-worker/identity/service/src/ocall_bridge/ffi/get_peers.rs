use crate::ocall_bridge::bridge_api::{Bridge, SidechainBridge};
use itp_utils::write_slice_and_whitespace_pad;
use log::*;
use sgx_types::sgx_status_t;
use std::{slice, sync::Arc};

#[no_mangle]
pub unsafe extern "C" fn ocall_get_trusted_peers_urls(
	peers_ptr: *mut u8,
	peers_size: u32,
) -> sgx_status_t {
	get_trusted_peers_urls(peers_ptr, peers_size, Bridge::get_sidechain_api())
}

fn get_trusted_peers_urls(
	peers_ptr: *mut u8,
	peers_size: u32,
	sidechain_api: Arc<dyn SidechainBridge>,
) -> sgx_status_t {
	trace!("    Entering ocall_get_trusted_peers_urls");

	let peers_encoded = match sidechain_api.get_trusted_peers_urls() {
		Ok(r) => r,
		Err(e) => {
			error!("get peers failed: {:?}", e);
			return sgx_status_t::SGX_ERROR_UNEXPECTED;
		},
	};

	let peers_encoded_slice = unsafe { slice::from_raw_parts_mut(peers_ptr, peers_size as usize) };
	if let Err(e) = write_slice_and_whitespace_pad(peers_encoded_slice, peers_encoded) {
		error!("Failed to transfer encoded peers to o-call buffer: {:?}", e);
		return sgx_status_t::SGX_ERROR_UNEXPECTED;
	}

	sgx_status_t::SGX_SUCCESS
}
