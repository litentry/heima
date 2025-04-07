// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object
use native_task_handler::NativeTaskError;

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
const PUMPX_API_ERROR_CODE: i32 = -32011;
const INTENT_NONCE_MISMATCH_ERROR_CODE: i32 = -32012;

pub fn get_native_task_error_code(error: &NativeTaskError) -> i32 {
	match error {
		NativeTaskError::UnauthorizedSender => UNAUTHORIZED_SENDER_CODE,
		NativeTaskError::AuthTokenCreationFailed => AUTH_TOKEN_CREATION_FAILED_CODE,
		NativeTaskError::InternalError => INVALID_MEMBER_IDENTITY_CODE,
		NativeTaskError::InvalidMemberIdentity => VALIDATION_DATA_VERIFICATION_FAILED_CODE,
		NativeTaskError::ValidationDataVerificationFailed => UNSUPPORTED_IDENTITY_TYPE_CODE,
		NativeTaskError::UnsupportedIdentityType => PUMPX_API_ERROR_CODE,
		NativeTaskError::PumpxApiError => PUMPX_API_ERROR_CODE,
		NativeTaskError::IntentNonceMismatch => INTENT_NONCE_MISMATCH_ERROR_CODE,
	}
}
