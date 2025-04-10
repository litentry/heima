// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object
use native_task_handler::{NativeTaskError, PumpxApiError, PumpxSignerError};

pub const INVALID_RAW_REQUEST_CODE: i32 = -32000;
pub const DECRYPT_REQUEST_FAILED_CODE: i32 = -32001;
pub const DECODE_REQUEST_FAILED_CODE: i32 = -32002;
pub const AUTH_VERIFICATION_FAILED_CODE: i32 = -32003;
pub const REQUIRE_ENCRYPTED_REQUEST_CODE: i32 = -32004;
pub const AES_KEY_CONVERT_FAILED_CODE: i32 = -32005;

// Native task error codes
const UNAUTHORIZED_SENDER_CODE: i32 = -32006;
const AUTH_TOKEN_CREATION_FAILED_CODE: i32 = -32007;
const INVALID_MEMBER_IDENTITY_CODE: i32 = -32008;
const VALIDATION_DATA_VERIFICATION_FAILED_CODE: i32 = -32009;
const UNSUPPORTED_IDENTITY_TYPE_CODE: i32 = -32010;

const PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE: i32 = -32031;
const PUMPX_API_USER_CONNECTION_FAILED_CODE: i32 = -32032;
const PUMPX_API_ERROR_CODE: i32 = -32033;
const PUMPX_API_ADD_WALLET_FAILED_CODE: i32 = -32034;
const PUMPX_API_CREATE_TRANSFER_UNSIGNED_TX_FAILED_CODE: i32 = -32035;
const PUMPX_API_SEND_TRANSFER_TX_FAILED_CODE: i32 = -32036;

const PUMPX_SIGNER_REQUEST_SIGNATURE_FAILED_CODE: i32 = -32050;

const INTENT_NONCE_MISMATCH_ERROR_CODE: i32 = -32060;

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
		},
		NativeTaskError::PumpxSignerError(signer_error) => match signer_error {
			PumpxSignerError::RequestSignatureFailed => PUMPX_SIGNER_REQUEST_SIGNATURE_FAILED_CODE,
		},
		NativeTaskError::InternalError => {
			log::error!("Internal error: {:?}", error);
			// This should not happen, we return the generic interal error code already from the api
			-32099
		},
		NativeTaskError::IntentNonceMismatch => INTENT_NONCE_MISMATCH_ERROR_CODE,
	}
}
