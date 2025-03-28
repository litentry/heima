use executor_primitives::{Hash, Identity};
use parentchain_rpc_client::TransactionStatus;
use parity_scale_codec::{Decode, Encode};
use std::vec::Vec;

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum NativeTaskOk {
	CallResponse(CallResponse),
	QueryResponse(QueryResponse),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum CallResponse {
	ExtrinsicReport {
		extrinsic_hash: Hash,
		block_hash: Option<Hash>,
		status: TransactionStatus<Hash>,
	},
	AuthToken(String),
	PumpxJwt {
		session_token: String,
		trade_token: String,
	},
}

impl From<CallResponse> for Result<NativeTaskOk, NativeTaskError> {
	fn from(response: CallResponse) -> Self {
		Ok(NativeTaskOk::CallResponse(response))
	}
}

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum QueryResponse {
	AccountStore(Vec<Identity>),
}

impl From<QueryResponse> for Result<NativeTaskOk, NativeTaskError> {
	fn from(response: QueryResponse) -> Self {
		Ok(NativeTaskOk::QueryResponse(response))
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTaskError {
	UnauthorizedSender,
	AuthTokenCreationFailed,
	InternalError,
	InvalidMemberIdentity,
	ValidationDataVerificationFailed,
}
