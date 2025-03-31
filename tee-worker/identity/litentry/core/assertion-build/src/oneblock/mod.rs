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

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

pub mod course;

use crate::*;
use http::header::{AUTHORIZATION, CONNECTION};
use http_req::response::Headers;
use itc_rest_client::{error::Error as RestClientError, RestGet, RestPath};
use lc_data_providers::{build_client_with_cert, DataProviderConfig};
use serde::{Deserialize, Serialize};
use std::string::ToString;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OneBlockResponse {
	#[serde(flatten)]
	data: serde_json::Value,
}

impl RestPath<String> for OneBlockResponse {
	fn get_path(_path: String) -> core::result::Result<String, RestClientError> {
		Ok("v1/blocks/e4068e6a326243468f35dcdc0c43f686/children".to_string())
	}
}

fn fetch_data_from_notion(
	course_type: &OneBlockCourseType,
	data_provider_config: &DataProviderConfig,
) -> Result<OneBlockResponse> {
	let oneblock_notion_key = data_provider_config.oneblock_notion_key.to_string();
	let oneblock_notion_url = data_provider_config.oneblock_notion_url.to_string();

	let mut headers = Headers::new();
	headers.insert(CONNECTION.as_str(), "close");
	headers.insert("Notion-Version", "2022-06-28");
	headers.insert(AUTHORIZATION.as_str(), oneblock_notion_key.as_str());

	let mut client = build_client_with_cert(oneblock_notion_url.as_str(), headers);

	client.get::<String, OneBlockResponse>(String::default()).map_err(|e| {
		Error::RequestVCFailed(
			Assertion::OneBlock(course_type.clone()),
			ErrorDetail::DataProviderError(ErrorString::truncate_from(
				format!("{e:?}").as_bytes().to_vec(),
			)),
		)
	})
}

const ONEBLOCK_TABLE_COL_NUM: usize = 6;
const ONEBLOCK_COURSE_PARTICIPATION_CONTENT: [&str; 6] =
	["第一课", "第二课", "第三课", "第四课", "第五课", "第六课"];
const ONEBLOCK_COURSE_COMPLETION_CONTENT: [&str; 2] = ["YES", "NO"];
const ONEBLOCK_COURSE_OUTSTANDING_CONTENT: [&str; 2] = ["YES", "NO"];

#[derive(Debug)]
pub struct OneBlockData {
	// 学号	 | 姓名	 | substrate地址 | 课程观看进度 | 是否毕业 | 是否优秀毕业
	rows: Vec<serde_json::Value>,
}

impl OneBlockData {
	pub fn new(rows: Vec<serde_json::Value>) -> Self {
		Self { rows }
	}

	// As shown in the table, the FIRST column is the student NO.
	fn check_student_number(&self, columns: &[serde_json::Value]) -> bool {
		self.get_column_text(&columns[0]).parse::<u32>().is_ok()
	}

	pub fn get_student_address(&self, columns: &[serde_json::Value]) -> String {
		self.get_column_text(&columns[2])
	}

	pub fn get_course_participation_text(&self, columns: &[serde_json::Value]) -> String {
		self.get_column_text(&columns[3])
	}

	pub fn get_course_completion_text(&self, columns: &[serde_json::Value]) -> String {
		self.get_column_text(&columns[4]).to_ascii_uppercase()
	}

	pub fn get_course_outstanding_text(&self, columns: &[serde_json::Value]) -> String {
		self.get_column_text(&columns[5]).to_ascii_uppercase()
	}

	pub fn check(&self, course_type: &OneBlockCourseType, addresses: Vec<String>) -> bool {
		for row in self.rows.iter() {
			if let Some(columns) = self.collect_columns(row) {
				if self.check_student_number(&columns)
					&& addresses.contains(&self.get_student_address(&columns))
				{
					return self.qualify(&columns, course_type);
				}
			}
		}

		false
	}
}

pub trait OneBlockTableDataHandler {
	fn collect_columns(&self, row: &serde_json::Value) -> Option<Vec<serde_json::Value>>;
	fn get_column_text(&self, column: &serde_json::Value) -> String;
}

impl OneBlockTableDataHandler for OneBlockData {
	fn collect_columns(&self, row: &serde_json::Value) -> Option<Vec<serde_json::Value>> {
		row.get("table_row")
			.and_then(|data| data.get("cells"))
			.and_then(|data| data.as_array())
			.filter(|data| data.len() == ONEBLOCK_TABLE_COL_NUM)
			.cloned()
	}

	fn get_column_text(&self, column: &serde_json::Value) -> String {
		column
			.as_array()
			.filter(|cells| cells.len() == 1)
			.and_then(|cell_data| {
				cell_data[0].get("text").and_then(|data| {
					data.get("content")
						.and_then(|data| data.as_str())
						.map(|data| data.trim().to_string())
				})
			})
			.unwrap_or_default()
	}
}

pub trait OneBlockAssertionQualify {
	fn qualify(&self, columns: &[serde_json::Value], course_type: &OneBlockCourseType) -> bool;
}

impl OneBlockAssertionQualify for OneBlockData {
	fn qualify(&self, columns: &[serde_json::Value], course_type: &OneBlockCourseType) -> bool {
		match course_type {
			OneBlockCourseType::CourseCompletion => {
				let text = self.get_course_completion_text(columns);
				ONEBLOCK_COURSE_COMPLETION_CONTENT[0] == text
			},
			OneBlockCourseType::CourseOutstanding => {
				let text = self.get_course_outstanding_text(columns);
				ONEBLOCK_COURSE_OUTSTANDING_CONTENT[0] == text
			},
			OneBlockCourseType::CourseParticipation => {
				let text = self.get_course_participation_text(columns);
				ONEBLOCK_COURSE_PARTICIPATION_CONTENT[5] == text
			},
		}
	}
}

pub fn query_oneblock_status(
	course_type: &OneBlockCourseType,
	addresses: Vec<String>,
	data_provider_config: &DataProviderConfig,
) -> Result<bool> {
	let oneblock_response = fetch_data_from_notion(course_type, data_provider_config)?;
	debug!("OneBlock Assertion Response: {oneblock_response:?}");

	Ok(check_oneblock_data(&oneblock_response, course_type, addresses))
}

pub fn check_oneblock_data(
	oneblock_response: &OneBlockResponse,
	course_type: &OneBlockCourseType,
	addresses: Vec<String>,
) -> bool {
	if let Some(rows) = oneblock_response
		.data
		.get("results")
		.and_then(|results| results.as_array())
		.cloned()
	{
		let data = OneBlockData::new(rows);
		return data.check(course_type, addresses);
	}

	false
}

#[cfg(test)]
mod tests {
	use super::*;

	const RESPONSE_ONEBLOCK: &str = r#"
{"object":"list","results":[{"object":"block","id":"8b371d31-1e60-4060-9f63-3527fa8cfe0c","parent":{"type":"block_id","block_id":"e4068e6a-3262-4346-8f35-dcdc0c43f111"},"created_time":"2023-09-05T03:21:00.000Z","last_edited_time":"2023-09-05T04:11:00.000Z","created_by":{"object":"user","id":"03bab8a4-8794-44d8-8843-961ee4c33485"},"last_edited_by":{"object":"user","id":"c208e40c-306a-4b31-a249-49139ff24411"},"has_children":false,"archived":false,"type":"table_row","table_row":{"cells":[[{"type":"text","text":{"content":"学号","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"学号","href":null}],[{"type":"text","text":{"content":"姓名","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"姓名","href":null}],[{"type":"text","text":{"content":"substrate地址","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"substrate地址","href":null}],[{"type":"text","text":{"content":"课程观看进度","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"课程观看进度","href":null}],[{"type":"text","text":{"content":"是否毕业","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"是否毕业","href":null}],[{"type":"text","text":{"content":"是否优秀毕业","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"是否优秀毕业","href":null}]]}},{"object":"block","id":"00d4a220-db1f-4d24-8b4f-4d3815dab3c5","parent":{"type":"block_id","block_id":"e4068e6a-3262-4346-8f35-dcdc0c43f111"},"created_time":"2023-09-05T03:21:00.000Z","last_edited_time":"2023-09-05T04:11:00.000Z","created_by":{"object":"user","id":"03bab8a4-8794-44d8-8843-961ee4c33485"},"last_edited_by":{"object":"user","id":"c208e40c-306a-4b31-a249-49139ff24411"},"has_children":false,"archived":false,"type":"table_row","table_row":{"cells":[[{"type":"text","text":{"content":"Team1","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"Team1","href":null}],[],[],[],[],[]]}},{"object":"block","id":"71130f91-0326-468a-8c42-c64c17e779af","parent":{"type":"block_id","block_id":"e4068e6a-3262-4346-8f35-dcdc0c43f111"},"created_time":"2023-09-05T03:21:00.000Z","last_edited_time":"2023-09-05T06:25:00.000Z","created_by":{"object":"user","id":"03bab8a4-8794-44d8-8843-961ee4c33485"},"last_edited_by":{"object":"user","id":"c208e40c-306a-4b31-a249-49139ff24411"},"has_children":false,"archived":false,"type":"table_row","table_row":{"cells":[[{"type":"text","text":{"content":"1264","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"1264","href":null}],[{"type":"text","text":{"content":"Student1","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"Student1","href":null}],[{"type":"text","text":{"content":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQQ","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQQ","href":null}],[{"type":"text","text":{"content":"第一课","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"第一课","href":null}],[{"type":"text","text":{"content":"yes","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"yes","href":null}],[{"type":"text","text":{"content":"yes","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"yes","href":null}]]}},{"object":"block","id":"72cdaf64-9dc5-4e71-8285-bba8d03abd99","parent":{"type":"block_id","block_id":"e4068e6a-3262-4346-8f35-dcdc0c43f111"},"created_time":"2023-09-05T03:21:00.000Z","last_edited_time":"2023-09-05T04:11:00.000Z","created_by":{"object":"user","id":"03bab8a4-8794-44d8-8843-961ee4c33485"},"last_edited_by":{"object":"user","id":"c208e40c-306a-4b31-a249-49139ff24411"},"has_children":false,"archived":false,"type":"table_row","table_row":{"cells":[[{"type":"text","text":{"content":"1263","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"1263","href":null}],[{"type":"text","text":{"content":"Clement Tam","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"Clement Tam","href":null}],[{"type":"text","text":{"content":"5HYaWcUJvX1xjNnduouJnD3F5q5X7uCpGxHV2yCRZEurymEE","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"5HYaWcUJvX1xjNnduouJnD3F5q5X7uCpGxHV2yCRZEurymEE","href":null}],[],[],[]]}},{"object":"block","id":"42d0ada7-1e3d-493a-bd2f-17821d2e54a2","parent":{"type":"block_id","block_id":"e4068e6a-3262-4346-8f35-dcdc0c43f111"},"created_time":"2023-09-05T03:21:00.000Z","last_edited_time":"2023-09-05T04:11:00.000Z","created_by":{"object":"user","id":"03bab8a4-8794-44d8-8843-961ee4c33485"},"last_edited_by":{"object":"user","id":"c208e40c-306a-4b31-a249-49139ff24411"},"has_children":false,"archived":false,"type":"table_row","table_row":{"cells":[[{"type":"text","text":{"content":"1262","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"1262","href":null}],[{"type":"text","text":{"content":"Student2","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"Student2","href":null}],[{"type":"text","text":{"content":"12zh1QyBrqddzgLbxBHjmoCnna8XsT9pTTMPCUt1f7WtC1f5","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"12zh1QyBrqddzgLbxBHjmoCnna8XsT9pTTMPCUt1f7WtC1f5","href":null}],[],[],[]]}}],"next_cursor":null,"has_more":false,"type":"block","block":{}}
"#;

	#[test]
	fn check_oneblock_data_works() {
		let address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQQ".to_string();
		let oneblock_response: OneBlockResponse = serde_json::from_str(RESPONSE_ONEBLOCK).unwrap();

		let participation = check_oneblock_data(
			&oneblock_response,
			&OneBlockCourseType::CourseParticipation,
			vec![address.clone()],
		);
		assert!(!participation);

		let completion = check_oneblock_data(
			&oneblock_response,
			&OneBlockCourseType::CourseCompletion,
			vec![address.clone()],
		);
		assert!(completion);

		let outstanding = check_oneblock_data(
			&oneblock_response,
			&OneBlockCourseType::CourseOutstanding,
			vec![address],
		);
		assert!(outstanding);
	}
}
