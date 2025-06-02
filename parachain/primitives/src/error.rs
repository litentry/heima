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
	alloc::{fmt, string::String},
	assertion::Assertion,
};

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{ConstU32, Printable},
	BoundedVec, DispatchError, DispatchErrorWithPostInfo,
};

pub type ErrorString = BoundedVec<u8, ConstU32<100>>;

// enum to reflect the error detail from TEE-worker processing
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum ErrorDetail {
	// error when importing the parentchain blocks and executing indirect calls
	#[codec(index = 0)]
	ImportError,
	// the direct or indirect request comes from an unauthorized signer
	#[codec(index = 1)]
	UnauthorizedSigner,
	// generic error when executing STF, the `ErrorString` should indicate the actual reason
	#[codec(index = 2)]
	StfError(ErrorString),
	// error when sending stf request to the receiver fails
	#[codec(index = 3)]
	SendStfRequestFailed,
	// generic parse error, can be caused by UTF8/JSON serde..
	#[codec(index = 4)]
	ParseError,
	// errors when communicating with data provider, e.g. HTTP error
	#[codec(index = 5)]
	DataProviderError(ErrorString),
	// error when tee-worker detects that verification data is associated with web2 identity but
	// web3 identity linking is requested and opposite
	#[codec(index = 6)]
	InvalidIdentity,
	// error when tee-worker detects that identity verification data is related to other web2
	// account than expected, for example wrong tweet id was provided
	#[codec(index = 7)]
	WrongWeb2Handle,
	// error when during web3 identity verification process tee-worker detects that signed message
	// is different from expected
	#[codec(index = 8)]
	UnexpectedMessage,
	// error when during web3 identity verification process tee-worker fails to verify signature
	// of verification data
	#[codec(index = 10)]
	VerifyWeb3SignatureFailed,
	// error when trying to build vc but no eligible identity is found
	#[codec(index = 11)]
	NoEligibleIdentity,
}

impl fmt::Debug for ErrorDetail {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ErrorDetail::ImportError => write!(f, "ImportError"),
			ErrorDetail::UnauthorizedSigner => write!(f, "UnauthorizedSigner"),
			ErrorDetail::StfError(error_string) => {
				let text = String::from_utf8(error_string.to_vec()).map_err(|_| fmt::Error)?;
				write!(f, "StfError({})", text)
			},
			ErrorDetail::SendStfRequestFailed => write!(f, "SendStfRequestFailed"),
			ErrorDetail::ParseError => write!(f, "ParseError"),
			ErrorDetail::DataProviderError(error_string) => {
				let text = String::from_utf8(error_string.to_vec()).map_err(|_| fmt::Error)?;
				write!(f, "DataProviderError({})", text)
			},
			ErrorDetail::InvalidIdentity => write!(f, "InvalidIdentity"),
			ErrorDetail::WrongWeb2Handle => write!(f, "WrongWeb2Handle"),
			ErrorDetail::UnexpectedMessage => write!(f, "UnexpectedMessage"),
			ErrorDetail::VerifyWeb3SignatureFailed => write!(f, "VerifyWeb3SignatureFailed"),
			ErrorDetail::NoEligibleIdentity => write!(f, "NoEligibleIdentity"),
		}
	}
}

// We could have used Into<ErrorDetail>, but we want it to be more explicit, similar to `into_iter`
pub trait IntoErrorDetail {
	fn into_error_detail(self) -> ErrorDetail;
}

// `From` is implemented for `DispatchError` and `DispatchErrorWithPostInfo` on the top level,
// because we know it can only happen during stf execution in enclave
impl From<DispatchError> for ErrorDetail {
	fn from(e: DispatchError) -> Self {
		ErrorDetail::StfError(ErrorString::truncate_from(
			<DispatchError as Into<&'static str>>::into(e).into(),
		))
	}
}

impl<T> From<DispatchErrorWithPostInfo<T>> for ErrorDetail
where
	T: Eq + PartialEq + Clone + Copy + Encode + Decode + Printable,
{
	fn from(e: DispatchErrorWithPostInfo<T>) -> Self {
		ErrorDetail::StfError(ErrorString::truncate_from(
			<DispatchErrorWithPostInfo<T> as Into<&'static str>>::into(e).into(),
		))
	}
}

// Identity Management Pallet Error
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum IMPError {
	// errors when executing individual error
	#[codec(index = 0)]
	LinkIdentityFailed(ErrorDetail),
	#[codec(index = 1)]
	DeactivateIdentityFailed(ErrorDetail),
	#[codec(index = 2)]
	ActivateIdentityFailed(ErrorDetail),
	// should be unreached, but just to be on the safe side
	// we should classify the error if we ever get this
	#[codec(index = 3)]
	UnclassifiedError(ErrorDetail),
}

impl frame_support::traits::PalletError for IMPError {
	// max_encoded_len
	const MAX_ENCODED_SIZE: usize = 1;
}

// Verified Credential(VC) Management Pallet Error
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum VCMPError {
	#[codec(index = 0)]
	RequestVCFailed(Assertion, ErrorDetail),
	// should be unreached, but just to be on the safe side
	// we should classify the error if we ever get this
	#[codec(index = 1)]
	UnclassifiedError(ErrorDetail),
}
