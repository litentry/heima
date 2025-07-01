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
use crate::types::{AccountId, Nonce};
use alloc::{format, string::String};
use codec::{Decode, Encode};
use derive_more::Display;
use litentry_primitives::{Assertion, ErrorDetail, ErrorString, IMPError, VCMPError};

pub type StfResult<T> = Result<T, StfError>;

#[derive(Debug, Display, PartialEq, Eq, Encode, Decode, Clone)]
pub enum StfError {
	// litentry
	#[codec(index = 0)]
	#[display(fmt = "LinkIdentityFailed: {:?}", _0)]
	LinkIdentityFailed(ErrorDetail),
	#[codec(index = 1)]
	#[display(fmt = "DeactivateIdentityFailed: {:?}", _0)]
	DeactivateIdentityFailed(ErrorDetail),
	#[codec(index = 2)]
	#[display(fmt = "ActivateIdentityFailed: {:?}", _0)]
	ActivateIdentityFailed(ErrorDetail),
	#[codec(index = 3)]
	#[display(fmt = "RequestVCFailed: {:?} {:?}", _0, _1)]
	RequestVCFailed(Assertion, ErrorDetail),
	#[codec(index = 4)]
	SetScheduledMrEnclaveFailed,
	#[codec(index = 5)]
	#[display(fmt = "SetIdentityNetworksFailed: {:?}", _0)]
	SetIdentityNetworksFailed(ErrorDetail),
	#[codec(index = 6)]
	InvalidAccount,
	#[codec(index = 7)]
	UnclassifiedError,
	#[codec(index = 8)]
	#[display(fmt = "RemovingIdentityFailed: {:?}", _0)]
	RemoveIdentityFailed(ErrorDetail),
	#[codec(index = 9)]
	EmptyIDGraph,

	// upstream errors
	#[codec(index = 20)]
	#[display(fmt = "Insufficient privileges {:?}, are you sure you are root?", _0)]
	MissingPrivileges(AccountId),
	#[codec(index = 21)]
	#[display(fmt = "Valid enclave signer account is required")]
	RequireEnclaveSignerAccount,
	#[codec(index = 22)]
	#[display(fmt = "Error dispatching runtime call. {:?}", _0)]
	Dispatch(String),
	#[codec(index = 23)]
	#[display(fmt = "Not enough funds to perform operation")]
	MissingFunds,
	#[codec(index = 24)]
	#[display(fmt = "Invalid Nonce {:?} != {:?}", _0, _1)]
	InvalidNonce(Nonce, Nonce),
	#[codec(index = 25)]
	StorageHashMismatch,
	#[codec(index = 26)]
	InvalidStorageDiff,
	#[codec(index = 27)]
	InvalidMetadata,
	#[codec(index = 28)]
	#[display(fmt = "CleaningIDGraphsFailed: {:?}", _0)]
	CleanIDGraphsFailed(ErrorDetail),
}

impl From<IMPError> for StfError {
	fn from(e: IMPError) -> Self {
		match e {
			IMPError::LinkIdentityFailed(d) => StfError::LinkIdentityFailed(d),
			IMPError::DeactivateIdentityFailed(d) => StfError::DeactivateIdentityFailed(d),
			IMPError::ActivateIdentityFailed(d) => StfError::ActivateIdentityFailed(d),
			_ => StfError::UnclassifiedError,
		}
	}
}

impl From<VCMPError> for StfError {
	fn from(e: VCMPError) -> Self {
		match e {
			VCMPError::RequestVCFailed(a, d) => StfError::RequestVCFailed(a, d),
			_ => StfError::UnclassifiedError,
		}
	}
}

impl StfError {
	// Convert StfError to IMPError that would be sent to parentchain
	pub fn to_imp_error(&self) -> IMPError {
		match self {
			StfError::LinkIdentityFailed(d) => IMPError::LinkIdentityFailed(d.clone()),
			StfError::DeactivateIdentityFailed(d) => IMPError::DeactivateIdentityFailed(d.clone()),
			StfError::ActivateIdentityFailed(d) => IMPError::ActivateIdentityFailed(d.clone()),
			_ => IMPError::UnclassifiedError(ErrorDetail::StfError(ErrorString::truncate_from(
				format!("{:?}", self).as_bytes().to_vec(),
			))),
		}
	}
	// Convert StfError to VCMPError that would be sent to parentchain
	pub fn to_vcmp_error(&self) -> VCMPError {
		match self {
			StfError::RequestVCFailed(a, d) => VCMPError::RequestVCFailed(a.clone(), d.clone()),
			_ => VCMPError::UnclassifiedError(ErrorDetail::StfError(ErrorString::truncate_from(
				format!("{:?}", self).as_bytes().to_vec(),
			))),
		}
	}
}
