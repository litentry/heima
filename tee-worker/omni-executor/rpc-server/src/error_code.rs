// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object
use crate::native_task_types::{NativeTaskError, PumpxApiError, PumpxSignerError};
use tracing::error;

// Standard JSON-RPC error codes
pub const PARSE_ERROR_CODE: i32 = -32700;
pub const INVALID_PARAMS_CODE: i32 = -32602;
pub const INTERNAL_ERROR_CODE: i32 = -32603;

// Server-defined error codes (-32000 to -32099)
pub const INVALID_RAW_REQUEST_CODE: i32 = -32000;
pub const DECRYPT_REQUEST_FAILED_CODE: i32 = -32001;
pub const DECODE_REQUEST_FAILED_CODE: i32 = -32002;
pub const AUTH_VERIFICATION_FAILED_CODE: i32 = -32003;
pub const REQUIRE_ENCRYPTED_REQUEST_CODE: i32 = -32004;
pub const AES_KEY_CONVERT_FAILED_CODE: i32 = -32005;

// Native task error codes
pub const UNAUTHORIZED_SENDER_CODE: i32 = -32006;
const AUTH_TOKEN_CREATION_FAILED_CODE: i32 = -32007;
const INVALID_MEMBER_IDENTITY_CODE: i32 = -32008;
const VALIDATION_DATA_VERIFICATION_FAILED_CODE: i32 = -32009;
const UNSUPPORTED_IDENTITY_TYPE_CODE: i32 = -32010;
pub const REQUIRE_AUTHENTICATION_CODE: i32 = -32012;

pub const PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE: i32 = -32031;
pub const PUMPX_API_USER_CONNECTION_FAILED_CODE: i32 = -32032;
const PUMPX_API_ERROR_CODE: i32 = -32033;
const PUMPX_API_ADD_WALLET_FAILED_CODE: i32 = -32034;
const PUMPX_API_CREATE_TRANSFER_UNSIGNED_TX_FAILED_CODE: i32 = -32035;
const PUMPX_API_SEND_TRANSFER_TX_FAILED_CODE: i32 = -32036;
const PUMPX_API_INVALID_INPUT_FAILED_CODE: i32 = -32037;
const PUMPX_API_CREATE_TRANSFER_TX_FAILED_CODE: i32 = -32038;
pub const PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE: i32 = -32039;
pub const POST_HEIMA_LOGIN_FAILED_CODE: i32 = -32041;

pub const PUMPX_SIGNER_REQUEST_SIGNATURE_FAILED_CODE: i32 = -32050;
pub const PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE: i32 = -32051;
pub const PUMPX_SIGNER_PUBKEY_TO_ADDRESS_FAILED_CODE: i32 = -32052;

pub const INTENT_NONCE_MISMATCH_ERROR_CODE: i32 = -32060;

// Input Validation Error Codes (-32100 to -32119)
pub const INVALID_CHAIN_ID_CODE: i32 = -32100;
pub const INVALID_WALLET_INDEX_CODE: i32 = -32101;
pub const INVALID_ADDRESS_FORMAT_CODE: i32 = -32102;
pub const INVALID_AMOUNT_CODE: i32 = -32103;
pub const INVALID_TOKEN_ADDRESS_CODE: i32 = -32104;
pub const MISSING_REQUIRED_FIELD_CODE: i32 = -32105;
pub const INVALID_HEX_FORMAT_CODE: i32 = -32106;
pub const INVALID_EMAIL_FORMAT_CODE: i32 = -32107;

// Account/Identity Error Codes (-32120 to -32139)
pub const ACCOUNT_PARSE_ERROR_CODE: i32 = -32121;
pub const INVALID_ACCOUNT_LENGTH_CODE: i32 = -32124;

// External Service Error Codes (-32160 to -32179)
pub const SIGNER_SERVICE_ERROR_CODE: i32 = -32160;
pub const EMAIL_SERVICE_ERROR_CODE: i32 = -32162;
pub const STORAGE_SERVICE_ERROR_CODE: i32 = -32163;
pub const EXTERNAL_API_ERROR_CODE: i32 = -32164;

// Response Processing Error Codes (-32180 to -32199)
pub const UNEXPECTED_RESPONSE_TYPE_CODE: i32 = -32180;

// Native Task Error Codes (-32200 to -32219)
pub const INVALID_USER_OPERATION_CODE: i32 = -32201;
pub const GAS_ESTIMATION_FAILED_CODE: i32 = -32202;
pub const SIGNATURE_SERVICE_UNAVAILABLE_CODE: i32 = -32203;

pub fn get_native_task_error_code(error: &NativeTaskError) -> i32 {
	match error {
		NativeTaskError::UnauthorizedSender => UNAUTHORIZED_SENDER_CODE,
		NativeTaskError::AuthTokenCreationFailed => AUTH_TOKEN_CREATION_FAILED_CODE,
		NativeTaskError::InvalidMemberIdentity => INVALID_MEMBER_IDENTITY_CODE,
		NativeTaskError::ValidationDataVerificationFailed => {
			VALIDATION_DATA_VERIFICATION_FAILED_CODE
		},
		NativeTaskError::UnsupportedIdentityType => UNSUPPORTED_IDENTITY_TYPE_CODE,
		NativeTaskError::PumpxApiError(api_error) => match api_error {
			PumpxApiError::GoogleCodeVerificationFailed => {
				PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE
			},
			PumpxApiError::UserConnectionFailed => PUMPX_API_USER_CONNECTION_FAILED_CODE,
			PumpxApiError::UnknownError => PUMPX_API_ERROR_CODE,
			PumpxApiError::AddWalletFailed => PUMPX_API_ADD_WALLET_FAILED_CODE,
			PumpxApiError::CreateTransferUnsignedTxFailed => {
				PUMPX_API_CREATE_TRANSFER_UNSIGNED_TX_FAILED_CODE
			},
			PumpxApiError::SendTransferTxFailed => PUMPX_API_SEND_TRANSFER_TX_FAILED_CODE,
			PumpxApiError::InvalidInput => PUMPX_API_INVALID_INPUT_FAILED_CODE,
			PumpxApiError::CreateTransferTxFailed => PUMPX_API_CREATE_TRANSFER_TX_FAILED_CODE,
			PumpxApiError::GetAccountUserIdFailed => PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE,
		},
		NativeTaskError::PumpxSignerError(signer_error) => match signer_error {
			PumpxSignerError::RequestSignatureFailed => PUMPX_SIGNER_REQUEST_SIGNATURE_FAILED_CODE,
			PumpxSignerError::RequestWalletFailed => PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE,
		},
		NativeTaskError::InternalError(_) => {
			error!("Internal error: {:?}", error);
			// This should not happen, we return the generic interal error code already from the api
			INTERNAL_ERROR_CODE
		},
		NativeTaskError::IntentNonceMismatch => INTENT_NONCE_MISMATCH_ERROR_CODE,
		NativeTaskError::UnsupportedChain => INVALID_CHAIN_ID_CODE,

		NativeTaskError::ChainNotSupported(_) => INVALID_CHAIN_ID_CODE,
		NativeTaskError::InvalidUserOperation(_) => INVALID_USER_OPERATION_CODE,
		NativeTaskError::GasEstimationFailed => GAS_ESTIMATION_FAILED_CODE,
		NativeTaskError::SignatureServiceUnavailable => SIGNATURE_SERVICE_UNAVAILABLE_CODE,
	}
}
