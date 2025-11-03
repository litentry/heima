// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object

// Standard JSON-RPC error codes
pub const INVALID_PARAMS_CODE: i32 = -32602;
pub const INTERNAL_ERROR_CODE: i32 = -32603;

// Server-defined error codes (-32000 to -32099)
#[allow(dead_code)]
pub const INVALID_RAW_REQUEST_CODE: i32 = -32000;
pub const DECRYPT_REQUEST_FAILED_CODE: i32 = -32001;
#[allow(dead_code)]
pub const DECODE_REQUEST_FAILED_CODE: i32 = -32002;
pub const AUTH_VERIFICATION_FAILED_CODE: i32 = -32003;
#[allow(dead_code)]
pub const REQUIRE_ENCRYPTED_REQUEST_CODE: i32 = -32004;
pub const AES_KEY_CONVERT_FAILED_CODE: i32 = -32005;

// Native task error codes
#[allow(dead_code)]
pub const UNAUTHORIZED_SENDER_CODE: i32 = -32006;
#[allow(dead_code)]
pub const AUTH_TOKEN_CREATION_FAILED_CODE: i32 = -32007;
#[allow(dead_code)]
pub const INVALID_MEMBER_IDENTITY_CODE: i32 = -32008;
#[allow(dead_code)]
pub const VALIDATION_DATA_VERIFICATION_FAILED_CODE: i32 = -32009;
#[allow(dead_code)]
pub const UNSUPPORTED_IDENTITY_TYPE_CODE: i32 = -32010;
#[allow(dead_code)]
pub const REQUIRE_AUTHENTICATION_CODE: i32 = -32012;

pub const PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE: i32 = -32031;
pub const PUMPX_API_ADD_WALLET_FAILED_CODE: i32 = -32034;
pub const PUMPX_API_CREATE_TRANSFER_TX_FAILED_CODE: i32 = -32038;
pub const PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE: i32 = -32039;
pub const POST_HEIMA_LOGIN_FAILED_CODE: i32 = -32041;

#[allow(dead_code)]
pub const PUMPX_SIGNER_REQUEST_SIGNATURE_FAILED_CODE: i32 = -32050;
pub const PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE: i32 = -32051;
pub const PUMPX_SIGNER_PUBKEY_TO_ADDRESS_FAILED_CODE: i32 = -32052;

#[allow(dead_code)]
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

// External Service Error Codes (-32160 to -32179)
pub const SIGNER_SERVICE_ERROR_CODE: i32 = -32160;
pub const EMAIL_SERVICE_ERROR_CODE: i32 = -32162;
pub const STORAGE_SERVICE_ERROR_CODE: i32 = -32163;
pub const EXTERNAL_API_ERROR_CODE: i32 = -32164;

// Response Processing Error Codes (-32180 to -32199)
pub const UNEXPECTED_RESPONSE_TYPE_CODE: i32 = -32180;

// Native Task Error Codes (-32200 to -32219)
pub const INVALID_USEROP_CODE: i32 = -32201;
pub const GAS_ESTIMATION_FAILED_CODE: i32 = -32202;
pub const SIGNATURE_SERVICE_UNAVAILABLE_CODE: i32 = -32203;
