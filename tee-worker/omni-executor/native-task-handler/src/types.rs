use executor_primitives::Hash;
use parentchain_rpc_client::TransactionStatus;
use parity_scale_codec::{Decode, Encode};
use pumpx::types::{AddWalletResponse, UserConnectResponse};

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum NativeTaskOk {
	ExtrinsicReport {
		extrinsic_hash: Hash,
		block_hash: Option<Hash>,
		status: TransactionStatus<Hash>,
	},
	AuthToken(String),
	PumpxRequestJwt {
		/// Used for less sensitive operations
		access_token: String,
		/// Used for user's identity verification before making sensitive operations
		id_token: String,

		user_connect_response: UserConnectResponse,
	},
	PumpxExportWallet(Vec<u8>),
	PumpxAddWallet(AddWalletResponse),
	PumpxSignLimitOrder(Vec<Vec<u8>>),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTaskError {
	UnauthorizedSender,
	AuthTokenCreationFailed,
	InternalError,
	InvalidMemberIdentity,
	ValidationDataVerificationFailed,
	UnsupportedIdentityType,
	PumpxApiError(PumpxApiError),
	IntentNonceMismatch,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum PumpxApiError {
	GoogleCodeVerificationFailed,
	UserConnectionFailed,
	UnknownError,
	AddWalletFailed,
}
