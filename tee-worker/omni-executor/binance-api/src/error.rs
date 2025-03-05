#[derive(Debug)]
pub enum Error {
	RequestFailed,
	ParseResponseFailed,
	MethodNotSupported,
	InvalidParams,
}
