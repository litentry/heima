// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use crate::*;
use codec::Decode;
use frame_support::storage::storage_prefix;
use itp_ocall_api::EnclaveOnChainOCallApi;
use itp_types::parentchain::Header;
use lc_credentials::IssuerRuntimeVersion;
use litentry_primitives::Address32;

const VC_A13_SUBJECT_DESCRIPTION: &str =
	"The user has a Polkadot Decoded 2023 Litentry Booth Special Badge";
const VC_A13_SUBJECT_TYPE: &str = "Decoded 2023 Basic Special Badge";

pub fn build<O: EnclaveOnChainOCallApi>(
	req: &AssertionBuildRequest,
	ocall_api: Arc<O>,
	who: &AccountId,
) -> Result<Credential> {
	debug!("Assertion A13 build, who: {:?}", account_id_to_string(&who));

	let key_prefix = storage_prefix(b"VCManagement", b"Delegatee");
	let response = ocall_api.get_storage_keys::<Header>(key_prefix.into(), None).map_err(|_| {
		Error::RequestVCFailed(Assertion::A13(who.clone()), ErrorDetail::ParseError)
	})?;
	let keys: Vec<String> = response
		.into_iter()
		.map(|r| String::decode(&mut r.as_slice()).unwrap_or_default())
		.collect();

	// if the signer can't be found in the delegatee list OR not the enclave account
	if !(keys.iter().any(|k| k.ends_with(hex::encode(&req.signer).as_str()))) {
		return Err(Error::RequestVCFailed(
			Assertion::A13(who.clone()),
			ErrorDetail::UnauthorizedSigner,
		))
	}

	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};

	match Credential::new(&Address32::from(who.clone()).into(), &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			// add subject info
			credential_unsigned.add_subject_info(VC_A13_SUBJECT_DESCRIPTION, VC_A13_SUBJECT_TYPE);

			// add assertion
			credential_unsigned.add_assertion_a13();
			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(Assertion::A13(who.clone()), e.into_error_detail()))
		},
	}
}
