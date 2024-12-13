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

// `QuotingEnclave` primitive part, copied from Integritee

use crate::{MrSigner, QeTcb, Vec};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

/// This represents all the collateral data that we need to store on chain in order to verify
/// the quoting enclave validity of another enclave that wants to register itself on chain
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct QuotingEnclave {
    // Todo: make timestamp: Moment
    pub issue_date: u64, // unix epoch in milliseconds
    // Todo: make timestamp: Moment
    pub next_update: u64, // unix epoch in milliseconds
    pub miscselect: [u8; 4],
    pub miscselect_mask: [u8; 4],
    pub attributes: [u8; 16],
    pub attributes_mask: [u8; 16],
    pub mrsigner: MrSigner,
    pub isvprodid: u16,
    /// Contains only the TCB versions that are considered UpToDate
    pub tcb: Vec<QeTcb>,
}

impl QuotingEnclave {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        issue_date: u64,
        next_update: u64,
        miscselect: [u8; 4],
        miscselect_mask: [u8; 4],
        attributes: [u8; 16],
        attributes_mask: [u8; 16],
        mrsigner: MrSigner,
        isvprodid: u16,
        tcb: Vec<QeTcb>,
    ) -> Self {
        Self {
            issue_date,
            next_update,
            miscselect,
            miscselect_mask,
            attributes,
            attributes_mask,
            mrsigner,
            isvprodid,
            tcb,
        }
    }

    pub fn attributes_flags_mask_as_u64(&self) -> u64 {
        let slice_as_array: [u8; 8] = self.attributes_mask[0..8].try_into().unwrap();
        u64::from_le_bytes(slice_as_array)
    }

    pub fn attributes_flags_as_u64(&self) -> u64 {
        let slice_as_array: [u8; 8] = self.attributes[0..8].try_into().unwrap();
        u64::from_le_bytes(slice_as_array)
    }
}
