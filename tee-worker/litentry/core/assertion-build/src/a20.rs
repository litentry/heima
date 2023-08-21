// Copyright 2020-2023 Litentry Technologies GmbH.
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

use http::header::CONNECTION;
use http_req::response::Headers;
use itc_rest_client::{error::Error as RestClientError, RestGet, RestPath};
use lc_data_providers::{build_client, vec_to_string};
use serde::{Deserialize, Serialize};

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::*;
use std::string::ToString;

const VC_A20_SUBJECT_DESCRIPTION: &str =
	"The user is an early bird user of the IdentityHub EVM version and has generated at least 1 credential during 2023 Aug 14th ~ Aug 21st.";
const VC_A20_SUBJECT_TYPE: &str = "IDHub EVM Version Early Bird";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EarlyBirdResponse {
	has_joined: bool,
}
impl RestPath<String> for EarlyBirdResponse {
	fn get_path(path: String) -> core::result::Result<String, RestClientError> {
		Ok(path)
	}
}

pub fn build(req: &AssertionBuildRequest, account: ParameterString) -> Result<Credential> {
	debug!("Assertion A20 build, who: {:?}", account_id_to_string(&req.who));

	let address = vec_to_string(account.to_vec()).map_err(|_| {
		Error::RequestVCFailed(Assertion::A20(account.clone()), ErrorDetail::ParseError)
	})?;

	let is_owned = check_account_owner(req, &address);
	if !is_owned {
		return Err(Error::RequestVCFailed(
			Assertion::A20(account),
			ErrorDetail::StfError(ErrorString::truncate_from(
				"You're not the account owner".into(),
			)),
		))
	}

	let mut headers = Headers::new();
	headers.insert(CONNECTION.as_str(), "close");
	let mut client = build_client(
		"https://archive-test.litentry.io/events/does-user-joined-evm-campaign",
		headers,
	);
	let query = vec![("account", address.as_str())];
	let value = client
		.get_with::<String, EarlyBirdResponse>("".to_string(), query.as_slice())
		.map(|data| data.has_joined)
		.map_err(|e| {
			Error::RequestVCFailed(
				Assertion::A20(account.clone()),
				ErrorDetail::DataProviderError(ErrorString::truncate_from(
					format!("{e:?}").as_bytes().to_vec(),
				)),
			)
		})?;
	match Credential::new(&req.who, &req.shard) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.add_subject_info(VC_A20_SUBJECT_DESCRIPTION, VC_A20_SUBJECT_TYPE);
			credential_unsigned.add_assertion_a20(value);
			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(Assertion::A20(account), e.into_error_detail()))
		},
	}
}

fn check_account_owner(req: &AssertionBuildRequest, address: &String) -> bool {
	let main_account = account_id_to_string(&req.who);
	if main_account == *address {
		return true
	}

	let identities = transpose_identity(&req.identities);
	for (_, addresses) in identities {
		for target in addresses {
			if target == *address {
				return true
			}
		}
	}

	false
}
