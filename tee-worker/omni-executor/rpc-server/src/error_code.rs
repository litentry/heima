// we should use -32000 to -32099 for implementation defined error codes,
// see https://www.jsonrpc.org/specification#error_object

pub const INVALID_AES_REQUEST_CODE: i32 = -32000;
pub const INVALID_SHARD_CODE: i32 = -32001;
pub const REQUEST_DECRYPTION_FAILED_CODE: i32 = -32002;
pub const INVALID_AUTHENTICATED_CALL_CODE: i32 = -32003;
pub const AUTHENTICATION_FAILED_CODE: i32 = -32004;
