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

use lc_data_providers::geniidata::{GeniidataResponse, ResponseData, ResponseItem};
use std::{collections::HashMap, vec::Vec};
use warp::{http::Response, Filter};

const EMPTY_RESPONSE: &str = r#"
{
	"code": 0,
	"message": "success",
	"data": {
		"count": 1,
		"limit": "20",
		"offset": "0",
		"list": []
	}
}
"#;

pub(crate) fn query() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::get()
		.and(warp::path!("api" / "1" / "brc20" / "balance"))
		.and(warp::query::<HashMap<String, String>>())
		.map(|params: HashMap<String, String>| {
			let default = String::default();
			let offset = params.get("offset").unwrap_or(&default).as_str();
			let tick = params.get("tick").unwrap_or(&default).as_str();
			let address = params.get("address").unwrap_or(&default).as_str();

			if !offset.is_empty() && offset != "0" {
				return Response::builder().body(EMPTY_RESPONSE.to_string());
			}

			let _expected_address =
				"bc1pgr5fw4p9gl9me0vzjklnlnap669caxc0gsk4j62gff2qktlw6naqm4m3d0";

			match (tick, address) {
				("ordi", _expected_address) => {
					let balance = "100.000000000000000000";
					let res = GeniidataResponse {
						code: 0,
						message: "success".into(),
						data: ResponseData {
							count: 1,
							limit: "20".into(),
							offset: "0".into(),
							list: vec![ResponseItem {
								tick: "ordi".into(),
								address: _expected_address.into(),
								overall_balance: balance.into(),
								transferable_balance: balance.into(),
								available_balance: balance.into(),
							}],
						},
					};
					Response::builder().body(serde_json::to_string(&res).unwrap())
				},
				("rats", _expected_address) => {
					let res = GeniidataResponse {
						code: 0,
						message: "success".into(),
						data: ResponseData {
							count: 0,
							limit: "20".into(),
							offset: "0".into(),
							list: vec![],
						},
					};
					Response::builder().body(serde_json::to_string(&res).unwrap())
				},
				_ => {
					let items = vec![
						("ordi", "100.000000000000000000"),
						("rats", "18000000.000000000000000000"),
						("MMSS", "1000.000000000000000000"),
					];
					let list: Vec<ResponseItem> = items
						.into_iter()
						.map(|(tick, balance)| ResponseItem {
							tick: tick.to_string(),
							address:
								"bc1pmkk62aua2pghenz4nps5jgllfaer29ulgpmjm4p5wlc4ewjx3p3ql260rj"
									.to_string(),
							overall_balance: balance.to_string(),
							transferable_balance: balance.to_string(),
							available_balance: balance.to_string(),
						})
						.collect();
					let res = GeniidataResponse {
						code: 0,
						message: "success".to_string(),
						data: ResponseData {
							count: 3,
							limit: "20".to_string(),
							offset: "0".to_string(),
							list,
						},
					};
					Response::builder().body(serde_json::to_string(&res).unwrap())
				},
			}
		})
}
