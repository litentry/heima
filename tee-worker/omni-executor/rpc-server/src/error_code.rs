// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object

pub const INVALID_RAW_REQUEST_CODE: i32 = -32000;
pub const DECRYPT_REQUEST_FAILED_CODE: i32 = -32001;
pub const DECODE_REQUEST_FAILED_CODE: i32 = -32002;
pub const AUTH_VERIFICATION_FAILED_CODE: i32 = -32003;
pub const REQUIRE_ENCRYPTED_REQUEST_CODE: i32 = -32004;
