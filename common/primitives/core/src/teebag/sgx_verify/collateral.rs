/*
    Copyright 2022 Integritee AG and Supercomputing Systems AG

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

pub extern crate alloc;

use crate::{Fmspc, MrSigner, Pcesvn, QeTcb, QuotingEnclave, TcbInfoOnChain, TcbVersionStatus};
use alloc::string::String;
use chrono::prelude::{DateTime, Utc};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use sp_std::prelude::*;

/// The data structures in here are designed such that they can be used to serialize/deserialize
/// the "TCB info" and "enclave identity" collateral data in JSON format provided by intel
/// See https://api.portal.trustedservices.intel.com/documentation for further information and examples

#[derive(Serialize, Deserialize)]
pub struct Tcb {
    isvsvn: u16,
}

impl Tcb {
    pub fn is_valid(&self) -> bool {
        // At the time of writing this code everything older than 6 is outdated
        // Intel does the same check in their DCAP implementation
        self.isvsvn >= 6
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbLevel {
    tcb: Tcb,
    /// Intel does not verify the tcb_date in their code and their API documentation also does
    /// not mention it needs verification.
    tcb_date: DateTime<Utc>,
    tcb_status: String,
    #[serde(rename = "advisoryIDs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    advisory_ids: Option<Vec<String>>,
}

impl TcbLevel {
    pub fn is_valid(&self) -> bool {
        // UpToDate is the only valid status (the other being OutOfDate and Revoked)
        // A possible extension would be to also verify that the advisory_ids list is empty,
        // but I think this could also lead to all TcbLevels being invalid
        self.tcb.is_valid() && self.tcb_status == "UpToDate"
    }
}

#[derive(Serialize, Deserialize)]
struct TcbComponent {
    svn: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(rename = "type")] //type is a keyword so we rename the field
    #[serde(skip_serializing_if = "Option::is_none")]
    tcb_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TcbFull {
    sgxtcbcomponents: [TcbComponent; 16],
    pcesvn: Pcesvn,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbLevelFull {
    tcb: TcbFull,
    /// Intel does not verify the tcb_date in their code and their API documentation also does
    /// not mention it needs verification.
    tcb_date: DateTime<Utc>,
    tcb_status: String,
    #[serde(rename = "advisoryIDs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    advisory_ids: Option<Vec<String>>,
}

impl TcbLevelFull {
    pub fn is_valid(&self) -> bool {
        // A possible extension would be to also verify that the advisory_ids list is empty,
        // but I think this could also lead to all TcbLevels being invalid
        //
        // Litentry: be more lenient with it
        self.tcb_status == "UpToDate"
            || self.tcb_status == "SWHardeningNeeded"
            || self.tcb_status == "ConfigurationNeeded"
            || self.tcb_status == "ConfigurationAndSWHardeningNeeded"
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnclaveIdentity {
    id: String,
    version: u16,
    issue_date: DateTime<Utc>,
    next_update: DateTime<Utc>,
    tcb_evaluation_data_number: u16,
    #[serde(deserialize_with = "deserialize_from_hex::<_, 4>")]
    #[serde(serialize_with = "serialize_to_hex::<_, 4>")]
    miscselect: [u8; 4],
    #[serde(deserialize_with = "deserialize_from_hex::<_, 4>")]
    #[serde(serialize_with = "serialize_to_hex::<_, 4>")]
    miscselect_mask: [u8; 4],
    #[serde(deserialize_with = "deserialize_from_hex::<_, 16>")]
    #[serde(serialize_with = "serialize_to_hex::<_, 16>")]
    attributes: [u8; 16],
    #[serde(deserialize_with = "deserialize_from_hex::<_, 16>")]
    #[serde(serialize_with = "serialize_to_hex::<_, 16>")]
    attributes_mask: [u8; 16],
    #[serde(deserialize_with = "deserialize_from_hex::<_, 32>")]
    #[serde(serialize_with = "serialize_to_hex::<_, 32>")]
    mrsigner: MrSigner,
    pub isvprodid: u16,
    pub tcb_levels: Vec<TcbLevel>,
}

fn serialize_to_hex<S, const N: usize>(x: &[u8; N], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hex::encode(x).to_uppercase())
}

fn deserialize_from_hex<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let hex = hex::decode(s).map_err(|_| D::Error::custom("Failed to deserialize hex string"))?;
    hex.try_into()
        .map_err(|_| D::Error::custom("Invalid hex length"))
}

impl EnclaveIdentity {
    /// This extracts the necessary information into the struct that we actually store in the chain
    pub fn to_quoting_enclave(&self) -> QuotingEnclave {
        let mut valid_tcbs: Vec<QeTcb> = Vec::new();
        for tcb in &self.tcb_levels {
            if tcb.is_valid() {
                valid_tcbs.push(QeTcb::new(tcb.tcb.isvsvn));
            }
        }
        QuotingEnclave::new(
            self.issue_date
                .timestamp_millis()
                .try_into()
                .expect("no support for negative unix timestamps"),
            self.next_update
                .timestamp_millis()
                .try_into()
                .expect("no support for negative unix timestamps"),
            self.miscselect,
            self.miscselect_mask,
            self.attributes,
            self.attributes_mask,
            self.mrsigner,
            self.isvprodid,
            valid_tcbs,
        )
    }

    pub fn is_valid(&self, timestamp_millis: i64) -> bool {
        self.id == "QE"
            && self.version == 2
            && self.issue_date.timestamp_millis() < timestamp_millis
            && timestamp_millis < self.next_update.timestamp_millis()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbInfo {
    id: String,
    version: u8,
    issue_date: DateTime<Utc>,
    next_update: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_hex::<_, 6>")]
    #[serde(serialize_with = "serialize_to_hex::<_, 6>")]
    pub fmspc: Fmspc,
    pce_id: String,
    tcb_type: u16,
    tcb_evaluation_data_number: u16,
    tcb_levels: Vec<TcbLevelFull>,
}

impl TcbInfo {
    /// This extracts the necessary information into a tuple (`(Key, Value)`) that we actually store
    /// in the chain
    pub fn to_chain_tcb_info(&self) -> (Fmspc, TcbInfoOnChain) {
        let valid_tcbs: Vec<TcbVersionStatus> = self
            .tcb_levels
            .iter()
            // Only store TCB levels on chain that are currently valid
            .filter(|tcb| tcb.is_valid())
            .map(|tcb| {
                let mut components = [0u8; 16];
                for (i, t) in tcb.tcb.sgxtcbcomponents.iter().enumerate() {
                    components[i] = t.svn;
                }
                TcbVersionStatus::new(components, tcb.tcb.pcesvn)
            })
            .collect();
        (
            self.fmspc,
            TcbInfoOnChain::new(
                self.issue_date
                    .timestamp_millis()
                    .try_into()
                    .expect("no support for negative unix timestamps"),
                self.next_update
                    .timestamp_millis()
                    .try_into()
                    .expect("no support for negative unix timestamps"),
                valid_tcbs,
            ),
        )
    }

    pub fn is_valid(&self, timestamp_millis: i64) -> bool {
        self.id == "SGX"
            && self.version == 3
            && self.issue_date.timestamp_millis() < timestamp_millis
            && timestamp_millis < self.next_update.timestamp_millis()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbInfoSigned {
    pub tcb_info: TcbInfo,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnclaveIdentitySigned {
    pub enclave_identity: EnclaveIdentity,
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcb_level_is_valid() {
        let t: TcbLevel = serde_json::from_str(
            r#"{"tcb":{"isvsvn":6}, "tcbDate":"2021-11-10T00:00:00Z", "tcbStatus":"UpToDate" }"#,
        )
        .unwrap();
        assert!(t.is_valid());

        let t: TcbLevel = serde_json::from_str(
            r#"{"tcb":{"isvsvn":6}, "tcbDate":"2021-11-10T00:00:00Z", "tcbStatus":"OutOfDate" }"#,
        )
        .unwrap();
        assert!(!t.is_valid());

        let t: TcbLevel = serde_json::from_str(
            r#"{"tcb":{"isvsvn":5}, "tcbDate":"2021-11-10T00:00:00Z", "tcbStatus":"UpToDate" }"#,
        )
        .unwrap();
        assert!(!t.is_valid());
    }
}
