// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object

// Standard JSON-RPC error codes
pub const INTERNAL_ERROR_CODE: i32 = -32603;

// Server-defined error codes (-32000 to -32099)
pub const INVALID_BACKEND_RESPONSE_CODE: i32 = -32000;
pub const DECRYPT_REQUEST_FAILED_CODE: i32 = -32001;
pub const INVALID_RPC_EXTENSION: i32 = -32002;
pub const AUTH_VERIFICATION_FAILED_CODE: i32 = -32003;
pub const AES_KEY_CONVERT_FAILED_CODE: i32 = -32005;

// Input Validation Error Codes (-32100 to -32119)
pub const INVALID_CHAIN_ID_CODE: i32 = -32100;
pub const INVALID_WALLET_INDEX_CODE: i32 = -32101;
pub const INVALID_ADDRESS_FORMAT_CODE: i32 = -32102;
pub const INVALID_AMOUNT_CODE: i32 = -32103;
pub const INVALID_TOKEN_ADDRESS_CODE: i32 = -32104;
pub const MISSING_REQUIRED_FIELD_CODE: i32 = -32105;
pub const INVALID_EMAIL_FORMAT_CODE: i32 = -32107;

// Error when calling external services (-32160 to -32179)
pub const SIGNER_SERVICE_ERROR_CODE: i32 = -32160;
pub const PUMPX_SERVICE_ERROR_CODE: i32 = -32161;
pub const EMAIL_SERVICE_ERROR_CODE: i32 = -32162;
pub const STORAGE_SERVICE_ERROR_CODE: i32 = -32163;
pub const EXTERNAL_API_ERROR_CODE: i32 = -32164;
pub const WILDMETA_SERVICE_ERROR_CODE: i32 = 32165;

pub const INVALID_USEROP_CODE: i32 = -32201;
pub const GAS_ESTIMATION_FAILED_CODE: i32 = -32202;
