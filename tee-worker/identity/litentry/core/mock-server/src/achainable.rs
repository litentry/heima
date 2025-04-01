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
#![allow(opaque_hidden_inferred_bound)]

const RES_TOTALTRANSACTIONS: &str = r#"
{
	"result": true,
	"display": [
		{
			"text": "Total transactions under 1 (Transactions: 41)",
			"result": true
		}
	],
	"runningCost": 1
}"#;
const RES_BODY_TRUE: &str = r#"
{
	"result": true
}"#;
const RES_BODY_FALSE: &str = r#"
{
	"result": false
}"#;
const RES_BODY_INVALID_CLASS_OF_YEAR: &str = r#"
{
    "name": "Account created between {dates}",
    "result": false,
    "display": [],
    "analyticsDisplay": [],
    "metadata": [
        null
    ],
    "runningCost": 1
}
"#;
const RES_BODY_OK_CLASS_OF_YEAR: &str = r#"
{
    "name": "Account created between {dates}",
    "result": true,
    "display": [
        {
            "text": "Account created between 01/01/2015 and 01/09/2023",
            "result": true
        }
    ],
    "analyticsDisplay": [],
    "metadata": [
        "2017-10-27T07:38:14.000Z"
    ],
    "runningCost": 1
}
"#;
const RES_BODY_OK_HOLDING_AMOUNT: &str = r#"
{
	"name": "Balance over {amount}",
	"result": true,
	"display": [
		{
			"text": "Balance over 0 (Balance is 800)",
			"result": true
		}
	],
	"analyticsDisplay": [],
	"runningCost": 1
}
"#;
const RES_ERRBODY: &str = r#"Error request."#;

use lc_data_providers::achainable::ReqBody;
use warp::{http::Response, path::FullPath, Filter};

pub(crate) fn query() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::post().and(warp::path::full()).and(warp::body::json()).map(
		|p: FullPath, body: ReqBody| {
			if p.as_str() != "/v1/run/system-labels" {
				return Response::builder().status(400).body(RES_ERRBODY.to_string());
			}
			if body.name == "Account total transactions under {amount}" {
				// Total transaction
				Response::builder().body(RES_TOTALTRANSACTIONS.to_string())
			} else if body.name == "Account created between {dates}" {
				// Class of year invalid address
				if body.address == "0x06e23f8209eCe9a33E24fd81440D46B08517adb5" {
					Response::builder().body(RES_BODY_INVALID_CLASS_OF_YEAR.to_string())
				} else {
					Response::builder().body(RES_BODY_OK_CLASS_OF_YEAR.to_string())
				}
			}
			// HoldingTime
			else if body.name == "Balance hodling {amount} since {date}"
				|| body.name == "ERC20 hodling {amount} of {token} since {date}"
			{
				if body.address == "0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C13"
					|| body.address == "0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C11"
				{
					Response::builder().body(RES_BODY_TRUE.to_string())
				} else if body.address == "0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C12"
					|| body.address == "0x1A64eD145A3CFAB3AA3D08721D520B4FD6Cf2C14"
				{
					Response::builder().body(RES_BODY_FALSE.to_string())
				} else if body.address == "0xa94586fBB5B736a3f6AF31f84EEcc7677D2e7F84"
					|| body.address == "0x4b978322643F9Bf6C15bf26d866B81E99F26b8DA"
				{
					Response::builder().body(RES_BODY_TRUE.to_string())
				} else {
					Response::builder().body(RES_BODY_FALSE.to_string())
				}
			}
			// HoldingAmount
			else if body.name == "Balance over {amount}" {
				Response::builder().body(RES_BODY_OK_HOLDING_AMOUNT.to_string())
			} else {
				Response::builder().body(RES_BODY_TRUE.to_string())
			}
		},
	)
}

pub(crate) fn query_labels(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::post().and(warp::path!("v1" / "run" / "labels" / String)).map(|label| {
		if label == "a719e99c-1f9b-432e-8f1d-cb3de0f14dde" {
			Response::builder().body(RES_BODY_TRUE.to_string())
		} else {
			Response::builder().body(RES_BODY_FALSE.to_string())
		}
	})
}
