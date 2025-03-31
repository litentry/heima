use executor_primitives::Hash;
use parentchain_rpc_client::TransactionStatus;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum NativeTaskOk {
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

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTaskError {
	UnauthorizedSender,
	AuthTokenCreationFailed,
	InternalError,
	InvalidMemberIdentity,
	ValidationDataVerificationFailed,
}
