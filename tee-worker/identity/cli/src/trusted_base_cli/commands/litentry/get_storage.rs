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

use crate::{
	command_utils::{get_worker_api_direct, mrenclave_to_base58},
	trusted_cli::TrustedCli,
	Cli, CliResult, CliResultOk,
};
use codec::Decode;
use frame_metadata::{RuntimeMetadata, StorageEntryType, StorageHasher};
use ita_sgx_runtime::Runtime;
use itc_rpc_client::direct_client::{DirectApi, DirectClient};
use itp_rpc::{Id, RpcRequest, RpcResponse, RpcReturnValue};
use itp_types::DirectRequestStatus;
use itp_utils::FromHexPrefixed;
use log::{error, warn};
use scale_value::{scale::TypeId, Value};
use sp_application_crypto::scale_info::TypeDef;
use std::format;

/// Usage:
///    Plain Storage: ./litentry-cli trusted --mrenclave $mrenclave get-storage Parentchain Number
///        Output: 123
///    Map Storage: ./litentry-cli trusted --mrenclave $mrenclave get-storage System Account 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
///        Output: { "nonce": 0, "consumers": 0, "providers": 1, "sufficients": 0, "data": { "free": 1000000000000000, "reserved": 1000000000000000, "misc_frozen": 0, "fee_frozen": 0 } }
#[derive(Parser)]
pub struct GetStorageCommand {
	/// Pallet Name
	pallet_name: String,

	/// Storage Name
	storage_name: String,

	/// Stroage Key
	keys: Vec<String>,
}
impl GetStorageCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_args: &TrustedCli) -> CliResult {
		let direct_api = get_worker_api_direct(cli);
		let mrenclave = if let Some(mrenclave) = trusted_args.mrenclave.clone() {
			mrenclave
		} else {
			let mrenclave = direct_api
				.get_state_mrenclave()
				.expect("Unable to retrieve MRENCLAVE from endpoint");
			mrenclave_to_base58(&mrenclave)
		};
		if let Some(v) = get_storage_value(
			direct_api,
			mrenclave,
			self.pallet_name.as_str(),
			self.storage_name.as_str(),
			&self.keys,
		) {
			println!("{}", v);
		} else {
			println!("None");
		}

		Ok(CliResultOk::None)
	}
}

fn get_storage_value(
	direct_api: DirectClient,
	mrenclave: String,
	pallet_name: &str,
	storage_name: &str,
	storage_entry_keys: &[String],
) -> Option<Value<TypeId>> {
	let metadata = Runtime::metadata();

	let metadata = match metadata.1 {
		RuntimeMetadata::V14(meta) => meta,
		_ => panic!("Invalid metadata"),
	};

	let pallet = metadata
		.pallets
		.iter()
		.find(|pallet| pallet.name == pallet_name)
		.unwrap()
		.clone();
	let storage = pallet.storage.unwrap();

	let storage_entry = storage
		.entries
		.iter()
		.find(|storage| storage.name == storage_name)
		.unwrap()
		.clone();

	let storage_entry_keys: Vec<Vec<u8>> = storage_entry_keys
		.iter()
		.map(|v| hex::decode(v.strip_prefix("0x").unwrap_or(v)).unwrap())
		.collect();

	let mut entry_bytes: Vec<u8> = vec![];

	write_storage_address_root_bytes(pallet_name, storage_name, &mut entry_bytes);

	let return_ty_id = match storage_entry.ty {
		StorageEntryType::Plain(ty) =>
			if !storage_entry_keys.is_empty() {
				panic!("Wrong Number Of Keys, expected: 0");
			} else {
				ty.id
			},
		StorageEntryType::Map { hashers, key, value } => {
			if hashers.len() != storage_entry_keys.len() {
				panic!("Wrong Number Of Keys, expected: {}", hashers.len());
			}

			let ty = metadata.types.resolve(key.id).unwrap();
			// If the key is a tuple, we encode each value to the corresponding tuple type.
			// If the key is not a tuple, encode a single value to the key type.
			let type_ids = match &ty.type_def {
				TypeDef::Tuple(tuple) => tuple.fields.iter().map(|f| f.id).collect(),
				_other => {
					vec![key.id]
				},
			};
			for ((key, _type_id), hasher) in storage_entry_keys.iter().zip(type_ids).zip(hashers) {
				hash_bytes(key.as_slice(), &hasher, &mut entry_bytes);
			}
			value.id
		},
	};

	if let Some(value) = send_get_storage_request(direct_api, mrenclave, &entry_bytes) {
		let mut bytes = if value.is_empty() { &storage_entry.default[..] } else { &value[..] };
		scale_value::scale::decode_as_type(&mut bytes, return_ty_id, &metadata.types).map_or_else(
			|err| {
				error!("decode error:{:?}", err);
				None
			},
			Some,
		)
	} else {
		None
	}
}

fn send_get_storage_request(
	direct_api: DirectClient,
	mrenclave: String,
	storage_entry_key: &Vec<u8>,
) -> Option<Vec<u8>> {
	let jsonrpc_call: String = RpcRequest::compose_jsonrpc_call(
		Id::Text("1".to_string()),
		"state_getStorage".to_string(),
		vec![mrenclave, format!("0x{}", hex::encode(storage_entry_key))],
	)
	.unwrap();

	let value = match direct_api.get(jsonrpc_call.as_str()) {
		Ok(response) => {
			let response: RpcResponse = serde_json::from_str(&response).unwrap();
			if let Ok(return_value) = RpcReturnValue::from_hex(&response.result) {
				match return_value.status {
					DirectRequestStatus::Ok => Some(return_value.value),
					DirectRequestStatus::Error => {
						warn!("request status is error");
						if let Ok(value) = String::decode(&mut return_value.value.as_slice()) {
							warn!("[Error] {}", value);
						}
						None
					},
					DirectRequestStatus::TrustedOperationStatus(status, top_hash) => {
						warn!("request status is: {:?}, top_hash: {:?}", status, top_hash);
						None
					},
					DirectRequestStatus::Processing(hash) => {
						warn!("request status is processing, hash: {:?}", hash);
						None
					},
				}
			} else {
				None
			}
		},
		Err(e) => {
			error!("failed to send request: {:?}", e);
			None
		},
	};
	direct_api.close().unwrap();
	value
}

fn write_storage_address_root_bytes(pallet_name: &str, storage_name: &str, out: &mut Vec<u8>) {
	out.extend(sp_core::hashing::twox_128(pallet_name.as_bytes()));
	out.extend(sp_core::hashing::twox_128(storage_name.as_bytes()));
}

/// Take some SCALE encoded bytes and a [`StorageHasher`] and hash the bytes accordingly.
fn hash_bytes(input: &[u8], hasher: &StorageHasher, bytes: &mut Vec<u8>) {
	match hasher {
		StorageHasher::Identity => bytes.extend(input),
		StorageHasher::Blake2_128 => bytes.extend(sp_core::hashing::blake2_128(input)),
		StorageHasher::Blake2_128Concat => {
			bytes.extend(sp_core::hashing::blake2_128(input));
			bytes.extend(input);
		},
		StorageHasher::Blake2_256 => bytes.extend(sp_core::hashing::blake2_256(input)),
		StorageHasher::Twox128 => bytes.extend(sp_core::hashing::twox_128(input)),
		StorageHasher::Twox256 => bytes.extend(sp_core::hashing::twox_256(input)),
		StorageHasher::Twox64Concat => {
			bytes.extend(sp_core::hashing::twox_64(input));
			bytes.extend(input);
		},
	}
}
