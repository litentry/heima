use executor_primitives::Hash;
use parentchain_rpc_client::TransactionStatus;
use parity_scale_codec::{Decode, Encode};
use pumpx::methods::add_wallet::AddWalletResponse;
use pumpx::methods::create_transfer_tx::CreateTransferTxResponse;
use pumpx::methods::user_connect::UserConnectResponse;

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
		backend_response: UserConnectResponse,
	},
	RequestIntentResult {
		intent_id: u32,
		success: bool,
	},
	IntentSwapResponse(Vec<u8>),
	PumpxExportWallet(Vec<u8>),
	PumpxAddWallet(AddWalletResponse),
	PumpxSignLimitOrder(Vec<Vec<u8>>),
	PumpxTransferWithdraw(CreateTransferTxResponse),
	PumpxNotifyLimitOrderResult,
	SubmitUserOp(Option<String>), // transaction_hash
	EstimateUserOpGas {
		call_gas_limit: u128,
		verification_gas_limit: u128,
		pre_verification_gas: u128,
		paymaster_verification_gas_limit: u128,
		paymaster_post_op_gas_limit: u128,
	},
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
	PumpxSignerError(PumpxSignerError),
	IntentNonceMismatch,
	UnsupportedChain,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum PumpxApiError {
	GoogleCodeVerificationFailed,
	UserConnectionFailed,
	UnknownError,
	InvalidInput,
	AddWalletFailed,
	CreateTransferUnsignedTxFailed,
	SendTransferTxFailed,
	CreateTransferTxFailed,
	GetAccountUserIdFailed,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum PumpxSignerError {
	RequestSignatureFailed,
	RequestWalletFailed,
}
