#![no_main]

use libfuzzer_sys::fuzz_target;
use sgx_verify::collateral::{EnclaveIdentity, TcbInfo};

fuzz_target!(|data: &[u8]| {
	let enclave: Result<EnclaveIdentity, serde_json::Error> = serde_json::from_slice(data);
	assert!(enclave.is_err());
	let tcb_info: Result<TcbInfo, serde_json::Error> = serde_json::from_slice(data);
	assert!(tcb_info.is_err());
});
