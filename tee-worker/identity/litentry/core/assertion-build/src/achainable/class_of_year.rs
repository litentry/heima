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

use crate::{achainable::request_achainable_classofyear, *};
use lc_credentials::{Credential, IssuerRuntimeVersion};
use lc_data_providers::DataProviderConfig;
use litentry_primitives::{AchainableClassOfYear, AchainableParams, AssertionBuildRequest};
use log::debug;

const VC_SUBJECT_DESCRIPTION: &str =
	"The class of year that the user account was created on a particular network (must have on-chain records)";
const VC_SUBJECT_TYPE: &str = "Account Class Of Year";

/// NOTE:
///
/// Build class of year
/// name: Account created between {dates}
/// chain: ethereum
///
/// True:
/// assertions:[
/// {
///    {
///        src:$account_created_year,
///        op:==,
///        dst:2015
///    }
/// }
/// ]
///
/// False:
/// assertions":[
/// {
///    src:$account_created_year,
///    op:==,
///    dst:Invalid
/// }
/// ]

pub fn build_class_of_year(
	req: &AssertionBuildRequest,
	param: AchainableClassOfYear,
	data_provider_config: &DataProviderConfig,
) -> Result<Credential> {
	debug!("Assertion Achainable build_class_of_year, who: {:?}", account_id_to_string(&req.who));
	let identities = transpose_identity(&req.identities);
	let addresses = identities
		.into_iter()
		.flat_map(|(_, addresses)| addresses)
		.collect::<Vec<String>>();

	let achainable_param = AchainableParams::ClassOfYear(param);
	let (ret, created_date) =
		request_achainable_classofyear(addresses, achainable_param.clone(), data_provider_config)?;

	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};

	match Credential::new(&req.who, &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.add_subject_info(VC_SUBJECT_DESCRIPTION, VC_SUBJECT_TYPE);
			credential_unsigned.update_class_of_year(ret, created_date);

			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(
				Assertion::Achainable(achainable_param),
				e.into_error_detail(),
			))
		},
	}
}
