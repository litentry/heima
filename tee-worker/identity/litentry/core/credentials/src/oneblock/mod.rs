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

use crate::{
	assertion_logic::{AssertionLogic, Op},
	Credential,
};
use litentry_primitives::OneBlockCourseType;

/// Course Completion / Course Outstanding Student / Course Participation
// (type, description)
const VC_ONEBLOCK_COURSE_INFOS: [(&str, &str); 3] = [
	("Completion - OneBlock+ Substrate Blockchain Development Course", "You have completed the course co-created by OneBlock+ and Parity: [Introduction to Substrate Blockchain Development, Phase 12]."),
	("Outstanding Student - OneBlock+ Substrate Blockchain Development Course", "You were awarded the title [Outstanding Student] in the course [Introduction to Substrate Blockchain Development, Phase 12] co-created by OneBlock+ and Parity."),
	("Participation - OneBlock+ Substrate Blockchain Development Course", "You were a participant to the course co-created by OneBlock+ and Parity: [Introduction to Substrate Blockchain Development, Phase 12]."),
];

pub trait OneBlockAssertionUpdate {
	fn update_notion_assertion(&mut self, course_type: &OneBlockCourseType, value: bool);
}

impl OneBlockAssertionUpdate for Credential {
	fn update_notion_assertion(&mut self, course_type: &OneBlockCourseType, value: bool) {
		let (assertion_content, info) = match course_type {
			OneBlockCourseType::CourseCompletion => {
				("$course_completed", VC_ONEBLOCK_COURSE_INFOS[0])
			},
			OneBlockCourseType::CourseOutstanding => {
				("$outstanding_student", VC_ONEBLOCK_COURSE_INFOS[1])
			},
			OneBlockCourseType::CourseParticipation => {
				("$course_participated", VC_ONEBLOCK_COURSE_INFOS[2])
			},
		};

		let assertion = AssertionLogic::new_item(assertion_content, Op::Equal, "true");
		self.credential_subject.assertions.push(assertion);
		self.credential_subject.values.push(value);

		self.add_subject_info(info.1, info.0);
	}
}
