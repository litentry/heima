// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use pumpx::PumpxApi;
use tracing::{debug, error};

/// Verify a Google authentication code using the Pumpx API
pub async fn verify_google_code(
	pumpx_api: &dyn PumpxApi,
	access_token: &str,
	google_code: String,
	language: Option<String>,
) -> bool {
	debug!("Calling pumpx verify_google_code, code: {}", google_code);
	let verify_result = pumpx_api.verify_google_code(access_token, google_code, language).await;
	verify_result.map_or_else(
		|e| {
			error!("Google code verification request failed: {:?}", e);
			false
		},
		|res| {
			res.data.result.map_or_else(
				|| {
					error!("Google code verification response result is none");
					false
				},
				|success| success,
			)
		},
	)
}
