use std::collections::HashMap;

pub trait TryIntoParams {
	fn try_into_params(&self) -> Result<HashMap<String, String>, &'static str>;
}
