use executor_primitives::Hash;
use parentchain_rpc_client::TransactionStatus;
use parity_scale_codec::{Decode, Encode};
use pumpx::methods::add_wallet::AddWalletResponse;
use pumpx::methods::create_transfer_tx::CreateTransferTxResponse;
use pumpx::methods::user_connect::UserConnectResponse;
use serde::{Deserialize, Serialize};

/// Information about estimated token cost for ERC20 paymaster operations
#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq, Eq)]
pub struct TokenCostEstimate {
	/// The ERC20 token address used for gas payment
	pub token_address: String,
	/// Token amount in smallest unit (e.g., wei for 18 decimal tokens)
	pub amount: u128,
	/// Token decimals for frontend formatting
	pub decimals: u8,
	/// Exchange rate used for calculation (tokens per ETH * 10^18)
	pub exchange_rate: u128,
}

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
		estimated_token_cost: Option<TokenCostEstimate>,
	},
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTaskError {
	UnauthorizedSender,
	AuthTokenCreationFailed,
	InternalError(Option<String>),
	InvalidMemberIdentity,
	ValidationDataVerificationFailed,
	UnsupportedIdentityType,
	PumpxApiError(PumpxApiError),
	PumpxSignerError(PumpxSignerError),
	IntentNonceMismatch,
	UnsupportedChain,
	ChainNotSupported(u64),
	InvalidUserOperation(String),
	GasEstimationFailed,
	SignatureServiceUnavailable,
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
