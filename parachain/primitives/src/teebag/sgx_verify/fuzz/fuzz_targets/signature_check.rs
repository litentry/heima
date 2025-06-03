#![no_main]
#![feature(core_panic)]

pub extern crate alloc;
extern crate core;

use libfuzzer_sys::fuzz_target;
use sgx_verify::deserialize_enclave_identity;

fuzz_target!(|data: &[u8]| {
	if data.len() < 64 {
		return
	}

	let cert = include_str!("../../test/dcap/qe_identity_cert.pem");
	let cert = cert.replace('\n', "");
	let decoded_cert = base64::decode(&cert).unwrap();
	let cert_der = webpki::types::CertificateDer::from(decoded_cert.as_slice());
	let cert = webpki::EndEntityCert::try_from(&cert_der).unwrap();
	let quoting_enclave = br#"{"id":"QE","version":2,"issueDate":"2022-10-18T21:55:07Z","nextUpdate":"2022-11-17T21:55:07Z","tcbEvaluationDataNumber":12,"miscselect":"00000000","miscselectMask":"FFFFFFFF","attributes":"11000000000000000000000000000000","attributesMask":"FBFFFFFFFFFFFFFF0000000000000000","mrsigner":"8C4F5775D796503E96137F77C68A829A0056AC8DED70140B081B094490C57BFF","isvprodid":1,"tcbLevels":[{"tcb":{"isvsvn":6},"tcbDate":"2021-11-10T00:00:00Z","tcbStatus":"UpToDate"},{"tcb":{"isvsvn":5},"tcbDate":"2020-11-11T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"isvsvn":4},"tcbDate":"2019-11-13T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"isvsvn":2},"tcbDate":"2019-05-15T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"isvsvn":1},"tcbDate":"2018-08-15T00:00:00Z","tcbStatus":"OutOfDate"}]}"#;
	let signature = &data[0..64];

	let res = deserialize_enclave_identity(&quoting_enclave[..], &signature, &cert);
	assert!(res.is_err(), "Found a valid signature");
});
