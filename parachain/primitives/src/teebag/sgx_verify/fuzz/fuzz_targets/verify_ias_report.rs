#![no_main]

use libfuzzer_sys::fuzz_target;
use sgx_verify::verify_ias_report;

fuzz_target!(|data: &[u8]| {
	// Check test that there is now panic and that the provided data is not a valid IAS report
	assert!(verify_ias_report(data).is_err());
});
