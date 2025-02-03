use parentchain_rpc_client::TransactionStatus;
use parity_scale_codec::{Codec, Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum NativeCallOk<Hash: Codec> {
	ExtrinsicReport {
		extrinsic_hash: Hash,
		block_hash: Option<Hash>,
		status: TransactionStatus<Hash>,
	},
	AuthToken(String),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeCallError {
	UnexpectedCall(String),
	UnauthorizedSender,
	AuthTokenCreationFailed,
	InternalError,
}
