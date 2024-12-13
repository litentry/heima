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

// `Tcb...` primitive part, copied from Integritee

use crate::{Cpusvn, Pcesvn, Vec};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

/// The list of valid TCBs for an enclave.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct QeTcb {
    pub isvsvn: u16,
}

impl QeTcb {
    pub fn new(isvsvn: u16) -> Self {
        Self { isvsvn }
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct TcbVersionStatus {
    pub cpusvn: Cpusvn,
    pub pcesvn: Pcesvn,
}

impl TcbVersionStatus {
    pub fn new(cpusvn: Cpusvn, pcesvn: Pcesvn) -> Self {
        Self { cpusvn, pcesvn }
    }

    pub fn verify_examinee(&self, examinee: &TcbVersionStatus) -> bool {
        for (v, r) in self.cpusvn.iter().zip(examinee.cpusvn.iter()) {
            if *v > *r {
                return false;
            }
        }
        self.pcesvn <= examinee.pcesvn
    }
}

/// This represents all the collateral data that we need to store on chain in order to verify
/// the quoting enclave validity of another enclave that wants to register itself on chain
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct TcbInfoOnChain {
    // Todo: make timestamp: Moment
    pub issue_date: u64, // unix epoch in milliseconds
    // Todo: make timestamp: Moment
    pub next_update: u64, // unix epoch in milliseconds
    tcb_levels: Vec<TcbVersionStatus>,
}

impl TcbInfoOnChain {
    pub fn new(issue_date: u64, next_update: u64, tcb_levels: Vec<TcbVersionStatus>) -> Self {
        Self {
            issue_date,
            next_update,
            tcb_levels,
        }
    }

    pub fn verify_examinee(&self, examinee: &TcbVersionStatus) -> bool {
        for tb in &self.tcb_levels {
            if tb.verify_examinee(examinee) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn tcb_full_is_valid() {
        // The strings are the hex encodings of the 16-byte CPUSVN numbers
        let reference = TcbVersionStatus::new(hex!("11110204018007000000000000000000"), 7);
        assert!(reference.verify_examinee(&reference));
        assert!(reference.verify_examinee(&TcbVersionStatus::new(
            hex!("11110204018007000000000000000000"),
            7
        )));
        assert!(reference.verify_examinee(&TcbVersionStatus::new(
            hex!("21110204018007000000000000000001"),
            7
        )));
        assert!(!reference.verify_examinee(&TcbVersionStatus::new(
            hex!("10110204018007000000000000000000"),
            6
        )));
        assert!(!reference.verify_examinee(&TcbVersionStatus::new(
            hex!("11110204018007000000000000000000"),
            6
        )));
    }
}
