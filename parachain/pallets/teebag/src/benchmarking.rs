use super::{Pallet as Teebag, *};
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use hex_literal::hex;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn create_test_enclaves<T: Config>(n: u32, mrenclave: MrEnclave) {
	for i in 0..n {
		let who: T::AccountId = account("who", i, 1);
		let test_enclave = Enclave::new(WorkerType::Identity).with_mrenclave(mrenclave);
		let _ = Teebag::<T>::add_enclave_identifier_internal(test_enclave.worker_type, &who);
		assert_ok!(Teebag::<T>::add_enclave(&who, &test_enclave));
	}
}

fn generate_test_mrenclave(index: u64) -> MrEnclave {
	let seed: u64 = 1671606747000 + index;
	let mut mrenclave = [0u8; 32];
	for (i, byte) in seed.to_ne_bytes().iter().cycle().take(32).enumerate() {
		mrenclave[i] = *byte;
	}

	MrEnclave::from(mrenclave)
}

fn create_test_authorized_enclaves<T: Config>(n: u32, worker_type: WorkerType) {
	for i in 0..n {
		let mrenclave = generate_test_mrenclave(i as u64);
		AuthorizedEnclave::<T>::try_mutate(worker_type, |v| v.try_push(mrenclave))
			.expect("Failed to add authorized enclave");
	}
}

// The following constants are copied from the test data used in the unit tests.
const PUBKEY: [u8; 32] = [
	65, 89, 193, 118, 86, 172, 17, 149, 206, 160, 174, 75, 219, 151, 51, 235, 110, 135, 20, 55,
	147, 162, 106, 110, 143, 207, 57, 64, 67, 63, 203, 95,
];
const QUOTING_ENCLAVE: &[u8; 1038] = br#"{"id":"QE","version":2,"issueDate":"2022-12-04T22:45:33Z","nextUpdate":"2023-01-03T22:45:33Z","tcbEvaluationDataNumber":13,"miscselect":"00000000","miscselectMask":"FFFFFFFF","attributes":"11000000000000000000000000000000","attributesMask":"FBFFFFFFFFFFFFFF0000000000000000","mrsigner":"8C4F5775D796503E96137F77C68A829A0056AC8DED70140B081B094490C57BFF","isvprodid":1,"tcbLevels":[{"tcb":{"isvsvn":6},"tcbDate":"2022-11-09T00:00:00Z","tcbStatus":"UpToDate"},{"tcb":{"isvsvn":5},"tcbDate":"2020-11-11T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00477"]},{"tcb":{"isvsvn":4},"tcbDate":"2019-11-13T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00334","INTEL-SA-00477"]},{"tcb":{"isvsvn":2},"tcbDate":"2019-05-15T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00219","INTEL-SA-00293","INTEL-SA-00334","INTEL-SA-00477"]},{"tcb":{"isvsvn":1},"tcbDate":"2018-08-15T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00202","INTEL-SA-00219","INTEL-SA-00293","INTEL-SA-00334","INTEL-SA-00477"]}]}"#;
const QUOTING_ENCLAVE_SIGNATURE: [u8; 64] = hex!("47accba321e57c20722a0d3d1db11c9b52661239857dc578ca1bde13976ee288cf39f72111ffe445c7389ef56447c79e30e6b83a8863ed9880de5bde4a8d5c91");
const QUOTING_ENCLAVE_CERTIFICATE_CHAIN: &[u8; 1891] =
	include_bytes!("../../../../common/primitives/core/src/teebag/sgx_verify/test/dcap/qe_identity_issuer_chain.pem");
const TCB_INFO: &[u8; 8359] = br#"{"id":"SGX","version":3,"issueDate":"2022-11-17T12:45:32Z","nextUpdate":"2023-04-16T12:45:32Z","fmspc":"00906EA10000","pceId":"0000","tcbType":0,"tcbEvaluationDataNumber":12,"tcbLevels":[{"tcb":{"sgxtcbcomponents":[{"svn":17},{"svn":17},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":7},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":11},"tcbDate":"2021-11-10T00:00:00Z","tcbStatus":"SWHardeningNeeded","advisoryIDs":["INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":17},{"svn":17},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":7},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":10},"tcbDate":"2020-11-11T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":17},{"svn":17},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":11},"tcbDate":"2021-11-10T00:00:00Z","tcbStatus":"ConfigurationAndSWHardeningNeeded","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":17},{"svn":17},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":10},"tcbDate":"2020-11-11T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded","advisoryIDs":["INTEL-SA-00477","INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":15},{"svn":15},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":7},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":10},"tcbDate":"2020-06-10T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":15},{"svn":15},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":10},"tcbDate":"2020-06-10T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":14},{"svn":14},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":7},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":10},"tcbDate":"2019-12-11T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":14},{"svn":14},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":10},"tcbDate":"2019-12-11T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":13},{"svn":13},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":3},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":9},"tcbDate":"2019-11-13T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":13},{"svn":13},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":9},"tcbDate":"2019-11-13T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":6},{"svn":6},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":1},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":7},"tcbDate":"2019-05-15T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00161","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":6},{"svn":6},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":7},"tcbDate":"2019-05-15T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":5},{"svn":5},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":1},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":7},"tcbDate":"2019-01-09T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00233","INTEL-SA-00161","INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":5},{"svn":5},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":1},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":6},"tcbDate":"2018-08-15T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00233","INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":5},{"svn":5},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":7},"tcbDate":"2019-01-09T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded","advisoryIDs":["INTEL-SA-00161","INTEL-SA-00233","INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":5},{"svn":5},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":6},"tcbDate":"2018-08-15T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded","advisoryIDs":["INTEL-SA-00203","INTEL-SA-00161","INTEL-SA-00233","INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":4},{"svn":4},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":5},"tcbDate":"2018-01-04T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00106","INTEL-SA-00115","INTEL-SA-00135","INTEL-SA-00203","INTEL-SA-00161","INTEL-SA-00233","INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]},{"tcb":{"sgxtcbcomponents":[{"svn":2},{"svn":2},{"svn":2},{"svn":4},{"svn":1},{"svn":128},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":4},"tcbDate":"2017-07-26T00:00:00Z","tcbStatus":"OutOfDate","advisoryIDs":["INTEL-SA-00088","INTEL-SA-00106","INTEL-SA-00115","INTEL-SA-00135","INTEL-SA-00203","INTEL-SA-00161","INTEL-SA-00233","INTEL-SA-00220","INTEL-SA-00270","INTEL-SA-00293","INTEL-SA-00219","INTEL-SA-00289","INTEL-SA-00320","INTEL-SA-00329","INTEL-SA-00381","INTEL-SA-00389","INTEL-SA-00477","INTEL-SA-00334"]}]}"#;
const TCB_SIGNATURE: [u8; 64] = hex!("71746f2148ecba04e35cf1ac77a7e6267ce99f6781c1031f724bb5bd94b8c1b6e4c07c01dc151692aa75be80dfba7350bb80c58314a6975189597e28e9bbc75c");
const TCB_CERTIFICATE_CHAIN: &[u8; 1891] = include_bytes!(
	"../../../../common/primitives/core/src/teebag/sgx_verify/test/dcap/tcb_info_issuer_chain.pem"
);

fn register_quoting_enclave_for_testing<T: Config>()
where
	<T as frame_system::Config>::Hash: From<[u8; 32]>,
	<T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
	let signer: T::AccountId = test_util::get_signer::<T::AccountId>(&PUBKEY);
	assert_ok!(Teebag::<T>::register_quoting_enclave(
		RawOrigin::Signed(signer).into(),
		QUOTING_ENCLAVE.to_vec(),
		QUOTING_ENCLAVE_SIGNATURE.to_vec(),
		QUOTING_ENCLAVE_CERTIFICATE_CHAIN.to_vec(),
	));
}

fn register_tcb_info_for_testing<T: Config>()
where
	<T as frame_system::Config>::Hash: From<[u8; 32]>,
	<T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
	let signer: T::AccountId = test_util::get_signer::<T::AccountId>(&PUBKEY);
	assert_ok!(Teebag::<T>::register_tcb_info(
		RawOrigin::Signed(signer).into(),
		TCB_INFO.to_vec(),
		TCB_SIGNATURE.to_vec(),
		TCB_CERTIFICATE_CHAIN.to_vec(),
	));
}

#[benchmarks(
    where <T as frame_system::Config>::Hash: From<[u8; 32]>,
          <T as frame_system::Config>::AccountId: From<[u8; 32]>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn force_add_enclave() {
		let who: T::AccountId = account("who", 1, 1);
		let test_enclave = Enclave::new(WorkerType::Identity);

		#[extrinsic_call]
		_(RawOrigin::Root, who.clone(), test_enclave.clone());

		assert_eq!(Teebag::<T>::enclave_count(WorkerType::Identity), 1);
		assert_eq!(EnclaveRegistry::<T>::get(who.clone()).unwrap(), test_enclave);
		assert_last_event::<T>(
			Event::EnclaveAdded {
				who,
				worker_type: test_enclave.worker_type,
				url: test_enclave.url,
			}
			.into(),
		)
	}

	#[benchmark]
	fn force_remove_enclave() {
		create_test_enclaves::<T>(T::MaxEnclaveIdentifier::get() - 1, test_util::TEST4_MRENCLAVE);
		let who: T::AccountId = account("who", 1, 99999);
		let test_enclave = Enclave::new(WorkerType::Identity);
		let _ = Teebag::<T>::add_enclave_identifier_internal(test_enclave.worker_type, &who);
		assert_ok!(Teebag::<T>::add_enclave(&who, &test_enclave));
		assert_eq!(
			Teebag::<T>::enclave_count(WorkerType::Identity),
			T::MaxEnclaveIdentifier::get()
		);

		#[extrinsic_call]
		_(RawOrigin::Root, who.clone());

		assert_last_event::<T>(Event::EnclaveRemoved { who }.into())
	}

	#[benchmark]
	fn force_remove_enclave_by_mrenclave() {
		create_test_enclaves::<T>(T::MaxEnclaveIdentifier::get(), test_util::TEST4_MRENCLAVE);
		assert_eq!(
			Teebag::<T>::enclave_count(WorkerType::Identity),
			T::MaxEnclaveIdentifier::get()
		);

		#[extrinsic_call]
		_(RawOrigin::Root, test_util::TEST4_MRENCLAVE);

		assert_eq!(Teebag::<T>::enclave_count(WorkerType::Identity), 0);
	}

	#[benchmark]
	fn force_remove_enclave_by_worker_type() {
		create_test_enclaves::<T>(T::MaxEnclaveIdentifier::get(), test_util::TEST4_MRENCLAVE);
		assert_eq!(
			Teebag::<T>::enclave_count(WorkerType::Identity),
			T::MaxEnclaveIdentifier::get()
		);

		#[extrinsic_call]
		_(RawOrigin::Root, WorkerType::Identity);

		assert_eq!(Teebag::<T>::enclave_count(WorkerType::Identity), 0);
	}

	#[benchmark]
	fn force_add_authorized_enclave() {
		let n_enclaves = T::MaxAuthorizedEnclave::get() - 1;
		create_test_authorized_enclaves::<T>(n_enclaves, WorkerType::Identity);
		assert_eq!(
			AuthorizedEnclave::<T>::get(WorkerType::Identity).iter().count() as u32,
			n_enclaves
		);

		#[extrinsic_call]
		_(RawOrigin::Root, WorkerType::Identity, test_util::TEST4_MRENCLAVE);

		assert_eq!(
			AuthorizedEnclave::<T>::get(WorkerType::Identity).iter().count() as u32,
			n_enclaves + 1
		);
		assert_last_event::<T>(
			Event::EnclaveAuthorized {
				worker_type: WorkerType::Identity,
				mrenclave: test_util::TEST4_MRENCLAVE,
			}
			.into(),
		)
	}

	#[benchmark]
	fn force_remove_authorized_enclave() {
		let n_enclaves = T::MaxAuthorizedEnclave::get() - 1;
		create_test_authorized_enclaves::<T>(n_enclaves, WorkerType::Identity);
		assert_eq!(
			AuthorizedEnclave::<T>::get(WorkerType::Identity).iter().count() as u32,
			n_enclaves
		);
		AuthorizedEnclave::<T>::try_mutate(WorkerType::Identity, |v| {
			v.try_push(test_util::TEST4_MRENCLAVE)
		})
		.expect("Failed to add authorized enclave");
		assert_eq!(
			AuthorizedEnclave::<T>::get(WorkerType::Identity).iter().count() as u32,
			n_enclaves + 1
		);

		#[extrinsic_call]
		_(RawOrigin::Root, WorkerType::Identity, test_util::TEST4_MRENCLAVE);

		assert_eq!(
			AuthorizedEnclave::<T>::get(WorkerType::Identity).iter().count() as u32,
			n_enclaves
		);
		assert_last_event::<T>(
			Event::EnclaveUnauthorized {
				worker_type: WorkerType::Identity,
				mrenclave: test_util::TEST4_MRENCLAVE,
			}
			.into(),
		)
	}

	#[benchmark]
	fn register_enclave_with_ias_attestation() {
		AuthorizedEnclave::<T>::try_mutate(WorkerType::Identity, |v| {
			v.try_push(test_util::TEST4_MRENCLAVE)
		})
		.expect("Failed to add authorized enclave");

		assert_ok!(pallet_timestamp::Pallet::<T>::set(
			RawOrigin::None.into(),
			T::Moment::saturated_from(test_util::TEST4_TIMESTAMP)
		));

		let signer: T::AccountId =
			test_util::get_signer::<T::AccountId>(test_util::TEST4_SIGNER_PUB);
		let _ = Teebag::<T>::add_enclave_identifier_internal(WorkerType::Identity, &signer);
		#[extrinsic_call]
		Teebag::<T>::register_enclave(
			RawOrigin::Signed(signer.clone()),
			WorkerType::Identity,
			WorkerMode::OffChainWorker,
			test_util::TEST4_CERT.to_vec(),
			test_util::URL.to_vec(),
			None,
			None,
			AttestationType::Ias,
		);

		let registered_enclave = Enclave::new(WorkerType::Identity)
			.with_mrenclave(test_util::TEST4_MRENCLAVE)
			.with_last_seen_timestamp(test_util::TEST4_TIMESTAMP)
			.with_sgx_build_mode(SgxBuildMode::Debug)
			.with_url(test_util::URL.to_vec())
			.with_attestation_type(AttestationType::Ias);

		assert_eq!(EnclaveRegistry::<T>::get(signer.clone()).unwrap(), registered_enclave);
		assert_last_event::<T>(
			Event::EnclaveAdded {
				who: signer,
				worker_type: WorkerType::Identity,
				url: test_util::URL.to_vec(),
			}
			.into(),
		)
	}

	#[benchmark]
	fn register_enclave_with_dcap_attestation() {
		let valid_timestamp: u64 = 1671606747000;
		assert_ok!(pallet_timestamp::Pallet::<T>::set(
			RawOrigin::None.into(),
			T::Moment::saturated_from(valid_timestamp)
		));
		register_quoting_enclave_for_testing::<T>();
		register_tcb_info_for_testing::<T>();

		let mrenclave: MrEnclave = [
			111, 144, 18, 11, 92, 31, 3, 97, 145, 18, 234, 200, 85, 226, 157, 110, 11, 228, 243,
			91, 41, 2, 170, 22, 154, 255, 255, 119, 25, 39, 126, 188,
		];

		AuthorizedEnclave::<T>::try_mutate(WorkerType::Identity, |v| v.try_push(mrenclave))
			.expect("Failed to add authorized enclave");

		let signer: T::AccountId = test_util::get_signer::<T::AccountId>(&PUBKEY);
		let _ = Teebag::<T>::add_enclave_identifier_internal(WorkerType::Identity, &signer);
		#[extrinsic_call]
		Teebag::<T>::register_enclave(
			RawOrigin::Signed(signer.clone()),
			WorkerType::Identity,
			WorkerMode::OffChainWorker,
			test_util::TEST1_DCAP_QUOTE.to_vec(),
			test_util::URL.to_vec(),
			None,
			None,
			AttestationType::Dcap(DcapProvider::Intel),
		);

		assert_eq!(Teebag::<T>::enclave_count(WorkerType::Identity), 1);
		assert_eq!(
			Teebag::<T>::enclave_registry(&signer).unwrap().last_seen_timestamp,
			valid_timestamp
		);
		assert_last_event::<T>(
			Event::EnclaveAdded {
				who: signer,
				worker_type: WorkerType::Identity,
				url: test_util::URL.to_vec(),
			}
			.into(),
		)
	}

	#[benchmark]
	fn unregister_enclave() {
		AuthorizedEnclave::<T>::try_mutate(WorkerType::Identity, |v| {
			v.try_push(test_util::TEST4_MRENCLAVE)
		})
		.expect("Failed to add authorized enclave");

		assert_ok!(pallet_timestamp::Pallet::<T>::set(
			RawOrigin::None.into(),
			T::Moment::saturated_from(test_util::TEST4_TIMESTAMP),
		));

		let signer: T::AccountId =
			test_util::get_signer::<T::AccountId>(test_util::TEST4_SIGNER_PUB);
		let _ = Teebag::<T>::add_enclave_identifier_internal(WorkerType::Identity, &signer);
		assert_ok!(Teebag::<T>::register_enclave(
			RawOrigin::Signed(signer.clone()).into(),
			WorkerType::Identity,
			WorkerMode::OffChainWorker,
			test_util::TEST4_CERT.to_vec(),
			test_util::URL.to_vec(),
			None,
			None,
			AttestationType::Ias,
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(signer.clone()));

		assert_eq!(Teebag::<T>::enclave_count(WorkerType::Identity), 0);
		assert_last_event::<T>(Event::EnclaveRemoved { who: signer }.into())
	}

	#[benchmark]
	fn register_quoting_enclave() {
		let signer: T::AccountId = test_util::get_signer::<T::AccountId>(&PUBKEY);
		let valid_timestamp: u64 = 1671606747000;

		assert_ok!(pallet_timestamp::Pallet::<T>::set(
			RawOrigin::None.into(),
			T::Moment::saturated_from(valid_timestamp)
		));

		assert_eq!(QuotingEnclaveRegistry::<T>::get(), QuotingEnclave::default());

		#[extrinsic_call]
		_(
			RawOrigin::Signed(signer),
			QUOTING_ENCLAVE.to_vec(),
			QUOTING_ENCLAVE_SIGNATURE.to_vec(),
			QUOTING_ENCLAVE_CERTIFICATE_CHAIN.to_vec(),
		);

		assert_eq!(QuotingEnclaveRegistry::<T>::get().isvprodid, 1);
	}

	#[benchmark]
	fn register_tcb_info() {
		let valid_timestamp: u64 = 1671606747000;
		assert_ok!(pallet_timestamp::Pallet::<T>::set(
			RawOrigin::None.into(),
			T::Moment::saturated_from(valid_timestamp)
		));
		let signer: T::AccountId = test_util::get_signer::<T::AccountId>(&PUBKEY);

		#[extrinsic_call]
		_(
			RawOrigin::Signed(signer),
			TCB_INFO.to_vec(),
			TCB_SIGNATURE.to_vec(),
			TCB_CERTIFICATE_CHAIN.to_vec(),
		);

		let fmspc = hex!("00906EA10000");
		let tcb_info = Teebag::<T>::tcb_info(fmspc);

		assert_eq!(tcb_info.next_update, 1681649132000);
	}

	#[benchmark]
	fn post_opaque_task() {
		let who: T::AccountId = account("who", 1, 1);
		let request = RsaRequest::default();

		#[extrinsic_call]
		_(RawOrigin::Signed(who), request.clone());

		assert_last_event::<T>(Event::OpaqueTaskPosted { request }.into())
	}

	#[benchmark]
	fn parentchain_block_processed() {
		let who: T::AccountId = account("who", 1, 1);
		let test_enclave = Enclave::new(WorkerType::Identity);
		let _ = Teebag::<T>::add_enclave_identifier_internal(test_enclave.worker_type, &who);
		assert_ok!(Teebag::<T>::add_enclave(&who, &test_enclave));
		let block_number: BlockNumberFor<T> = 10u32.into();

		#[extrinsic_call]
		_(RawOrigin::Signed(who.clone()), H256::default(), block_number, H256::default());

		assert_last_event::<T>(
			Event::ParentchainBlockProcessed {
				who,
				block_number,
				block_hash: H256::default(),
				task_merkle_root: H256::default(),
			}
			.into(),
		)
	}

	#[benchmark]
	fn sidechain_block_imported() {
		let who = account("who", 1, 1);
		let shard = H256::default();
		let block_number = 10u64;
		let next_finalization_candidate_block_number: u64 = 11;
		let block_header_hash = H256::default();
		let test_enclave =
			Enclave::new(WorkerType::Identity).with_worker_mode(WorkerMode::Sidechain);
		let _ = Teebag::<T>::add_enclave_identifier_internal(test_enclave.worker_type, &who);
		assert_ok!(Teebag::<T>::add_enclave(&who, &test_enclave));
		SidechainBlockFinalizationCandidate::<T>::insert(shard, 10);

		#[extrinsic_call]
		_(
			RawOrigin::Signed(who.clone()),
			shard,
			block_number,
			next_finalization_candidate_block_number,
			block_header_hash,
		);

		assert_eq!(
			SidechainBlockFinalizationCandidate::<T>::get(shard),
			next_finalization_candidate_block_number
		);
		assert_last_event::<T>(
			Event::SidechainBlockFinalized { who, sidechain_block_number: block_number }.into(),
		)
	}

	#[benchmark]
	fn add_enclave_identifier() {
		let who: T::AccountId = account("who", 1, 1);
		let worker_type = WorkerType::Identity;

		#[extrinsic_call]
		_(RawOrigin::Root, worker_type, who.clone());

		assert!(EnclaveIdentifier::<T>::get(worker_type).contains(&who));
	}

	impl_benchmark_test_suite!(Teebag, super::mock::new_test_ext(false), super::mock::Test);
}
