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

use codec::{Decode, Encode};
use ita_sgx_runtime::System;
use itp_stf_interface::ExecuteGetter;
use itp_stf_primitives::{traits::GetterAuthorization, types::KeyPair};
use itp_utils::stringify::account_id_to_string;
use litentry_macros::if_development_or;
use litentry_primitives::{Identity, LitentryMultiSignature};
use log::*;
use sp_std::vec;
use std::prelude::v1::*;

use itp_stf_primitives::traits::PoolTransactionValidation;
use sp_runtime::transaction_validity::{
	TransactionValidityError, UnknownTransaction, ValidTransaction,
};

#[cfg(feature = "development")]
use crate::helpers::ALICE_ACCOUNTID32;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Getter {
	#[codec(index = 0)]
	public(PublicGetter),
	#[codec(index = 1)]
	trusted(TrustedGetterSigned),
}

impl Default for Getter {
	fn default() -> Self {
		Getter::public(PublicGetter::some_value)
	}
}
impl From<PublicGetter> for Getter {
	fn from(item: PublicGetter) -> Self {
		Getter::public(item)
	}
}

impl From<TrustedGetterSigned> for Getter {
	fn from(item: TrustedGetterSigned) -> Self {
		Getter::trusted(item)
	}
}

impl GetterAuthorization for Getter {
	fn is_authorized(&self) -> bool {
		match self {
			Self::trusted(ref getter) => getter.verify_signature(),
			Self::public(_) => true,
		}
	}
}

impl PoolTransactionValidation for Getter {
	fn validate(&self) -> Result<ValidTransaction, TransactionValidityError> {
		match self {
			Self::public(_) =>
				Err(TransactionValidityError::Unknown(UnknownTransaction::CannotLookup)),
			Self::trusted(trusted_getter_signed) => Ok(ValidTransaction {
				priority: 1 << 20,
				requires: vec![],
				provides: vec![trusted_getter_signed.signature.encode()],
				longevity: 64,
				propagate: true,
			}),
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum PublicGetter {
	#[codec(index = 0)]
	some_value,
	#[codec(index = 1)]
	nonce(Identity),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TrustedGetter {
	#[codec(index = 0)]
	free_balance(Identity),
	#[codec(index = 1)]
	reserved_balance(Identity),
}

impl TrustedGetter {
	pub fn sender_identity(&self) -> &Identity {
		match self {
			TrustedGetter::free_balance(sender_identity) => sender_identity,
			TrustedGetter::reserved_balance(sender_identity) => sender_identity,
		}
	}

	pub fn sign(&self, pair: &KeyPair) -> TrustedGetterSigned {
		let signature = pair.sign(self.encode().as_slice());
		TrustedGetterSigned { getter: self.clone(), signature }
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TrustedGetterSigned {
	pub getter: TrustedGetter,
	pub signature: LitentryMultiSignature,
}

impl TrustedGetterSigned {
	pub fn new(getter: TrustedGetter, signature: LitentryMultiSignature) -> Self {
		TrustedGetterSigned { getter, signature }
	}

	pub fn verify_signature(&self) -> bool {
		// in non-prod, we accept signature from Alice too
		if_development_or!(
			{
				self.signature
					.verify(self.getter.encode().as_slice(), self.getter.sender_identity())
					|| self
						.signature
						.verify(self.getter.encode().as_slice(), &ALICE_ACCOUNTID32.into())
			},
			{
				self.signature
					.verify(self.getter.encode().as_slice(), self.getter.sender_identity())
			}
		)
	}
}

impl ExecuteGetter for Getter {
	fn execute(self) -> Option<Vec<u8>> {
		match self {
			Getter::trusted(g) => g.execute(),
			Getter::public(g) => g.execute(),
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		match self {
			Getter::trusted(g) => g.get_storage_hashes_to_update(),
			Getter::public(g) => g.get_storage_hashes_to_update(),
		}
	}
}

impl ExecuteGetter for TrustedGetterSigned {
	fn execute(self) -> Option<Vec<u8>> {
		match self.getter {
			TrustedGetter::free_balance(who) =>
				if let Some(account_id) = who.to_account_id() {
					let info = System::account(&account_id);
					debug!("TrustedGetter free_balance");
					debug!("AccountInfo for {} is {:?}", account_id_to_string(&who), info);
					std::println!("⣿STF⣿ 🔍 TrustedGetter query: free balance for ⣿⣿⣿ is ⣿⣿⣿",);
					Some(info.data.free.encode())
				} else {
					None
				},
			TrustedGetter::reserved_balance(who) =>
				if let Some(account_id) = who.to_account_id() {
					let info = System::account(&account_id);
					debug!("TrustedGetter reserved_balance");
					debug!("AccountInfo for {} is {:?}", account_id_to_string(&who), info);
					debug!("Account reserved balance is {}", info.data.reserved);
					Some(info.data.reserved.encode())
				} else {
					None
				},
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		Vec::new()
	}
}

impl ExecuteGetter for PublicGetter {
	fn execute(self) -> Option<Vec<u8>> {
		match self {
			PublicGetter::some_value => Some(42u32.encode()),
			PublicGetter::nonce(identity) =>
				if let Some(account_id) = identity.to_account_id() {
					let nonce = System::account_nonce(&account_id);
					debug!("PublicGetter nonce");
					debug!("Account nonce is {}", nonce);
					Some(nonce.encode())
				} else {
					None
				},
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		Vec::new()
	}
}
