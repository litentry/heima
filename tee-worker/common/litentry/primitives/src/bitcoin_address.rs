/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

use bitcoin::{
	address::Address, key::PublicKey, network::Network, secp256k1::Secp256k1, Script,
	XOnlyPublicKey,
};
use core::str::FromStr;
use std::string::{String, ToString};

// Some dependency conflict of bitcoin crate with enclave building
// when putting these functions into core-premitives/utils.
pub fn p2wpkh_address(pubkey_string: &str) -> String {
	let pubkey = PublicKey::from_str(pubkey_string).expect("pubkey");
	let address = Address::p2wpkh(&pubkey, Network::Bitcoin);
	if let Ok(address) = address {
		return address.to_string()
	}
	"".to_string()
}

pub fn p2sh_address(pubkey_string: &str) -> String {
	let pubkey = PublicKey::from_str(pubkey_string).expect("pubkey");
	let address = Address::p2shwpkh(&pubkey, Network::Bitcoin);
	if let Ok(address) = address {
		return address.to_string()
	}
	"".to_string()
}

pub fn p2tr_address(pubkey_string: &str) -> String {
	let pubkey = PublicKey::from_str(pubkey_string).expect("pubkey");
	let xonly_pubkey = XOnlyPublicKey::from(pubkey.inner);
	// unisat wallet uses is this way
	let secp = Secp256k1::verification_only();
	let address = Address::p2tr(&secp, xonly_pubkey, None, Network::Bitcoin);
	address.to_string()
}

pub fn p2pkh_address(pubkey_string: &str) -> String {
	let pubkey = PublicKey::from_str(pubkey_string).expect("pubkey");
	let address = Address::p2pkh(&pubkey, Network::Bitcoin);
	address.to_string()
}

pub fn p2wsh_address(pubkey_string: &str) -> String {
	let script = Script::from_bytes(pubkey_string.as_bytes());
	let address = Address::p2wsh(script, Network::Bitcoin);
	address.to_string()
}
