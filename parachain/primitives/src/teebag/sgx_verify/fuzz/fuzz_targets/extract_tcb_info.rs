#![no_main]

use libfuzzer_sys::fuzz_target;
use sgx_verify::extract_tcb_info;

fuzz_target!(|data: &[u8]| {
	assert!(extract_tcb_info(data).is_err());
});
