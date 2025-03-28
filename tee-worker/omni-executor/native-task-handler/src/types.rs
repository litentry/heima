use executor_primitives::{Hash, Identity};
use parentchain_rpc_client::TransactionStatus;
use parity_scale_codec::{Decode, Encode};
use pumpx::types::UserConnectResponse;
use std::vec::Vec;

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum NativeOperationOk {
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
		user_connect_response: UserConnectResponse,
	},
}

impl From<CallResponse> for Result<NativeOperationOk, NativeOperationError> {
	fn from(response: CallResponse) -> Self {
		Ok(NativeOperationOk::CallResponse(response))
	}
}

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum QueryResponse {
	AccountStore(Vec<Identity>),
}

impl From<QueryResponse> for Result<NativeOperationOk, NativeOperationError> {
	fn from(response: QueryResponse) -> Self {
		Ok(NativeOperationOk::QueryResponse(response))
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeOperationError {
	UnauthorizedSender,
	AuthTokenCreationFailed,
	InternalError,
	InvalidMemberIdentity,
	ValidationDataVerificationFailed,
	UnsupportedIdentityType,
	PumpxApiError,
}
