// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object

// Server-defined error codes (-32000 to -32099)
pub const INVALID_BACKEND_RESPONSE_CODE: i32 = -32000;
pub const INVALID_USEROP_CODE: i32 = -32001;
pub const INVALID_RPC_EXTENSION: i32 = -32002;
pub const AUTH_VERIFICATION_FAILED_CODE: i32 = -32003;
pub const GAS_ESTIMATION_FAILED_CODE: i32 = -32005;

// Error when calling external services (-32160 to -32179)
pub const SIGNER_SERVICE_ERROR_CODE: i32 = -32160;
pub const PUMPX_SERVICE_ERROR_CODE: i32 = -32161;
pub const EMAIL_SERVICE_ERROR_CODE: i32 = -32162;
pub const STORAGE_SERVICE_ERROR_CODE: i32 = -32163;
pub const EXTERNAL_API_ERROR_CODE: i32 = -32164;
pub const WILDMETA_SERVICE_ERROR_CODE: i32 = -32165;
