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

use crate::*;
use codec::Decode;
use frame_support::{StorageHasher, Twox64Concat};
use http::header::CONNECTION;
use itc_rest_client::{
	error::Error as RestClientError,
	http_client::{HttpClient, SendWithCertificateVerification},
	rest_client::{Headers, RestClient},
	RestPath, RestPost,
};
use itp_stf_primitives::types::AccountId;
use itp_utils::hex_display::AsBytesRef;
use lc_common::abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls};
use lc_credentials::{
	litentry_profile::lit_staking::UpdateLITStakingAmountCredential, IssuerRuntimeVersion,
};
use lc_data_providers::build_client_with_cert;
use litentry_primitives::ParentchainBalance;
use serde::{Deserialize, Serialize};
use std::string::ToString;

const LIT_TOKEN_DECIMALS: u128 = 1_000_000_000_000;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonRPCRequest {
	id: usize,
	jsonrpc: String,
	method: String,
	params: Vec<String>,
}

impl JsonRPCRequest {
	pub fn state_getstorage(params: &str) -> Self {
		Self {
			id: 1,
			jsonrpc: "2.0".to_string(),
			method: "state_getStorage".to_string(),
			params: vec![params.to_string()],
		}
	}
}

impl RestPath<String> for JsonRPCRequest {
	fn get_path(path: String) -> core::result::Result<String, RestClientError> {
		Ok(path)
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonRPCResponse {
	pub id: usize,
	pub jsonrpc: String,
	pub result: Option<String>,
}

pub struct LitentryStakingClient {
	client: RestClient<HttpClient<SendWithCertificateVerification>>,
}

impl Default for LitentryStakingClient {
	fn default() -> Self {
		Self::new()
	}
}

impl LitentryStakingClient {
	pub fn new() -> Self {
		let mut headers = Headers::new();
		headers.insert(CONNECTION.as_str(), "close");
		let client = build_client_with_cert("https://litentry-rpc.dwellir.com:443", headers);
		LitentryStakingClient { client }
	}

	pub fn send_request(&mut self, data: &JsonRPCRequest) -> Result<JsonRPCResponse> {
		self.client
			.post_capture::<String, JsonRPCRequest, JsonRPCResponse>(String::default(), data)
			.map_err(|e| {
				Error::RequestVCFailed(
					Assertion::LITStaking,
					ErrorDetail::DataProviderError(ErrorString::truncate_from(
						format!("{e:?}").as_bytes().to_vec(),
					)),
				)
			})
	}
}

pub trait QueryParachainStaking {
	fn query_delegator_state(&mut self, key: &str) -> Result<Option<String>>;
}

impl QueryParachainStaking for LitentryStakingClient {
	fn query_delegator_state(&mut self, key: &str) -> Result<Option<String>> {
		let data = JsonRPCRequest::state_getstorage(key);
		let res = self.send_request(&data)?;

		Ok(res.result)
	}
}

// primitive types copied from pallet-parachain-staking
//
// it's to break the cargo deps to the whole pallet, especially when we
// expect a `polkadot-v0.9.42` version of pallet within the worker
mod parachain_staking_primitives {
	use super::*;
	use codec::{Decode, Encode};

	#[derive(Clone, Encode, Decode)]
	pub struct OrderedSet<T>(pub Vec<T>);

	#[derive(Clone, Encode, Decode)]
	pub struct Bond {
		pub owner: AccountId,
		pub amount: ParentchainBalance,
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode)]
	pub enum DelegatorStatus {
		/// Active with no scheduled exit
		#[codec(index = 0)]
		Active,
	}

	#[derive(Clone, Encode, Decode)]
	/// Delegator state
	pub struct Delegator {
		/// Delegator account
		pub id: AccountId,
		/// All current delegations
		pub delegations: OrderedSet<Bond>,
		/// Total balance locked for this delegator
		pub total: ParentchainBalance,
		/// Sum of pending revocation amounts + bond less amounts
		pub less_total: ParentchainBalance,
		/// Status for this delegator
		pub status: DelegatorStatus,
	}
}
use parachain_staking_primitives::*;

pub struct DelegatorState;
impl DelegatorState {
	pub fn query_lit_staking(
		&mut self,
		client: &mut LitentryStakingClient,
		identities: Vec<Identity>,
	) -> Result<u128> {
		let mut total_staking_amount = 0_u128;

		loop_with_abort_strategy::<fn(&_) -> bool, Identity, Error>(
			identities,
			|identity| {
				let storage_key = DelegatorState::delegator_state_storage_key(identity)?;
				let storage_in_hex = client.query_delegator_state(&storage_key)?;

				if let Some(storage_in_hex) = storage_in_hex {
					let delegator = DelegatorState::decode_delegator(&storage_in_hex)?;
					let total = delegator.total;

					total_staking_amount += total;
				}
				Ok(LoopControls::Continue)
			},
			AbortStrategy::FailFast::<fn(&_) -> bool>,
		)
		.map_err(|errors| errors[0].clone())?;

		Ok(total_staking_amount / LIT_TOKEN_DECIMALS)
	}

	fn delegator_state_storage_key(identity: &Identity) -> Result<String> {
		// encoded partial key: ParachainStaking DelegatorState
		// 0xa686a3043d0adcf2fa655e57bc595a78131da8bc800de21b19b3ba9ed33cfacc
		let params = "0xa686a3043d0adcf2fa655e57bc595a78131da8bc800de21b19b3ba9ed33cfacc";
		let acc = identity
			.to_native_account()
			.ok_or(Error::RequestVCFailed(Assertion::LITStaking, ErrorDetail::ParseError))?;
		let cocat = Twox64Concat::hash(acc.as_ref());

		Ok(params.to_string() + &hex::encode(&cocat))
	}

	fn decode_delegator(storage_in_hex: &str) -> Result<Delegator> {
		// Remove 0x
		if let Some(storage_in_hex_without_prefix) = storage_in_hex.strip_prefix("0x") {
			if let Ok(decoded) = hex::decode(storage_in_hex_without_prefix) {
				if let Ok(delegator) = Delegator::decode(&mut decoded.as_bytes_ref()) {
					return Ok(delegator)
				}
			}
		}

		Err(Error::RequestVCFailed(
			Assertion::LITStaking,
			ErrorDetail::DataProviderError(ErrorString::truncate_from(
				"Invalid ParachainStaking DelegatorState".as_bytes().to_vec(),
			)),
		))
	}
}

pub fn build(req: &AssertionBuildRequest) -> Result<Credential> {
	debug!("Assertion building LIT staking amount");

	let mut identities = vec![];
	req.identities
		.iter()
		.filter(|(identity, _)| identity.is_substrate())
		.for_each(|identity| {
			identities.push(identity.0.clone());
		});

	let mut client = LitentryStakingClient::new();
	let staking_amount = DelegatorState.query_lit_staking(&mut client, identities)?;
	let runtime_version = IssuerRuntimeVersion {
		parachain: req.parachain_runtime_version,
		sidechain: req.sidechain_runtime_version,
	};
	match Credential::new(&req.who, &req.shard, &runtime_version) {
		Ok(mut credential_unsigned) => {
			credential_unsigned.update_lit_staking_amount(staking_amount);
			Ok(credential_unsigned)
		},
		Err(e) => {
			error!("Generate unsigned credential failed {:?}", e);
			Err(Error::RequestVCFailed(Assertion::LITStaking, e.into_error_detail()))
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::{crypto::Ss58Codec, ed25519};

	#[test]
	fn decode_delegator_works() {
		let delegator_in_hex = "0xa06c6b3f286fc8fe59cfe3d85ea8df043ddc08742ff9cfe2ed3d6ba1da1f4b4a18149ccce9d526a65ba54fccc24f5d1dee62f9b87915d4004eb932959d468a9e6200f008d0236e040000000000000000001acba651394c1b8d65d06702662921077e58b09d0fc15f3126a427144f83c95f00b0b522399003000000000000000000223188d5f28ee27f7e9067e89bc52fca8f1da20c6a7548a21cef18d8934f820f006048774c3a0400000000000000000024f07a3858f8d4dece9806b6c7e4ee165ba8c98e6ff4d066dd0284d46aee967d00e046350eb003000000000000000000e0d0031d0a450dfc4bb16333fe575dfb8452bb0f79c768181478190a5d9a653f00b064d8ab8103000000000000000000fc7a9dd32be14db4695555aa9a2abd240a8c2160f84ccb403a985701dd13fe5000409dde18410300000000000000000000d04f567cab160000000000000000000000000000000000000000000000000000";
		let d = DelegatorState::decode_delegator(delegator_in_hex);
		assert!(d.is_ok());
	}

	#[test]
	fn decode_delegator_should_fail_works() {
		let delegator_in_hex = "0xa0";
		let d = DelegatorState::decode_delegator(delegator_in_hex);
		assert!(d.is_err());
	}

	#[test]
	fn delegator_state_storage_key_works() {
		let address = "4A2nt96tH4ej9B3S9sagrKdJubYvu8No4hE9kjjrXMWvGLYW";

		let pubkey = ed25519::Public::from_ss58check(address).unwrap();
		let acc = AccountId::new(pubkey.0);
		let identity = Identity::from(acc);

		let storage_key = DelegatorState::delegator_state_storage_key(&identity).unwrap();
		let target_key = "0xa686a3043d0adcf2fa655e57bc595a78131da8bc800de21b19b3ba9ed33cfacc01e6a2b4eb558329a06c6b3f286fc8fe59cfe3d85ea8df043ddc08742ff9cfe2ed3d6ba1da1f4b4a".to_string();
		assert_eq!(storage_key, target_key);
	}
}
