use serde::Serialize;

// /v3/trade/cross_fail
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossFailBody {
	pub request_id: u32,
	pub fail_reason: String,
}
