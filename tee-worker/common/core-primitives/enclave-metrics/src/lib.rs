/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

extern crate core;
#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use codec::{Decode, Encode};
use core::time::Duration;
use litentry_primitives::{Assertion, RequestType};
use std::{boxed::Box, string::String};

#[derive(Encode, Decode, Debug)]
pub enum EnclaveMetric {
	// identity
	SetSidechainBlockHeight(u64),
	TopPoolSizeSet(u64),
	TopPoolSizeIncrement,
	TopPoolSizeDecrement,
	StfTaskExecutionTime(Box<RequestType>, f64),
	SuccessfulTrustedOperationIncrement(String),
	FailedTrustedOperationIncrement(String),
	ParentchainBlockImportTime(Duration),
	SidechainBlockImportTime(Duration),
	SidechainSlotPrepareTime(Duration),
	SidechainSlotStfExecutionTime(Duration),
	SidechainSlotBlockCompositionTime(Duration),
	SidechainBlockBroadcastingTime(Duration),
	VCBuildTime(Assertion, Duration),
	SuccessfullVCIssuance,
	FailedVCIssuance,
	ParentchainEventProcessed(String),
	DynamicAssertionSaveTime(Duration),
	DynamicAssertionGetTime(Duration),
}
