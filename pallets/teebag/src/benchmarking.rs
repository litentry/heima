use super::{Pallet as Teebag, *};
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use std::time::SystemTime;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn create_test_enclaves<T: Config>(n: u32, mrenclave: MrEnclave) {
	for i in 0..n {
		let who: T::AccountId = account("who", i, 1);
		let test_enclave = Enclave::new(WorkerType::Identity).with_mrenclave(mrenclave.clone());
		assert_ok!(Teebag::<T>::add_enclave(&who, &test_enclave));
	}
}

fn generate_random_mrenclave() -> MrEnclave {
	let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();

	let mut mrenclave = [0u8; 32];
	for (i, byte) in time.to_ne_bytes().iter().cycle().take(32).enumerate() {
		mrenclave[i] = *byte;
	}

	MrEnclave::from(mrenclave)
}

fn create_test_authorized_enclaves<T: Config>(n: u32, worker_type: WorkerType) {
	for _ in 0..n {
		let mrenclave = generate_random_mrenclave();
		AuthorizedEnclave::<T>::try_mutate(worker_type, |v| v.try_push(mrenclave))
			.expect("Failed to add authorized enclave");
	}
}

#[benchmarks(
    where <T as frame_system::Config>::Hash: From<[u8; 32]>
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
		assert_eq!(EnclaveRegistry::<T>::get(who.clone()).unwrap(), test_enclave.clone());
		assert_last_event::<T>(
			Event::EnclaveAdded {
				who,
				worker_type: test_enclave.worker_type,
				url: test_enclave.url.clone(),
			}
			.into(),
		)
	}

	#[benchmark]
	fn force_remove_enclave() {
		create_test_enclaves::<T>(T::MaxEnclaveIdentifier::get() - 1, test_util::TEST4_MRENCLAVE);
		let who: T::AccountId = account("who", 1, 99999);
		let test_enclave = Enclave::new(WorkerType::Identity);
		assert_ok!(Teebag::<T>::add_enclave(&who, &test_enclave));
		assert_eq!(
			Teebag::<T>::enclave_count(WorkerType::Identity),
			T::MaxEnclaveIdentifier::get()
		);

		#[extrinsic_call]
		_(RawOrigin::Root, who.clone());

		assert_eq!(Teebag::<T>::enclave_count(WorkerType::Identity), 0);
		assert_eq!(EnclaveRegistry::<T>::get(who.clone()), None);
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

	impl_benchmark_test_suite!(Teebag, super::mock::new_bench_ext(), super::mock::Test);
}
