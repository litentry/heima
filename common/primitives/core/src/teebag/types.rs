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

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{ed25519::Public as Ed25519Public, RuntimeDebug, H256};
use sp_std::prelude::*;

pub type MrSigner = [u8; 32];
pub type MrEnclave = [u8; 32];
pub type Cpusvn = [u8; 16];
pub type Pcesvn = u16;
pub type Fmspc = [u8; 6];
pub type ShardIdentifier = H256;
pub type SidechainBlockNumber = u64;
pub type EnclaveFingerprint = H256;

/// Different modes that control enclave registration and running:
/// - `Production`: default value. It perfroms all checks for enclave registration and runtime
/// - `Development`: the most lenient, most check are skipped during registration or runtime
/// - `Maintenance`: a placeholder value for now - maybe to stall sidechain block production
///
/// please note:
/// `Attestation::Ignore` is only possible under `OperationalMode::Development`, but not vice versa.
/// So if you define `Attestation::Ias`, the attestation will be verified even in `Development` mode
#[derive(
    PartialEq, Eq, Clone, Copy, Default, Encode, Decode, Debug, TypeInfo, Serialize, Deserialize,
)]
pub enum OperationalMode {
    #[default]
    Production,
    Development,
    Maintenance,
}

#[derive(Encode, Decode, Default, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum DcapProvider {
    #[default]
    Intel,
    MAA,
    Local,
    Integritee,
}

#[derive(Encode, Decode, Clone, Copy, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum AttestationType {
    #[default]
    Ignore,
    Ias,
    Dcap(DcapProvider),
}

#[derive(Encode, Decode, Clone, Copy, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum WorkerType {
    #[default]
    Identity,
    OmniExecutor,
}

#[derive(Encode, Decode, Clone, Copy, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum WorkerMode {
    #[default]
    OffChainWorker,
    Sidechain,
}

#[derive(Encode, Decode, Copy, Clone, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SgxBuildMode {
    #[default]
    Production,
    Debug,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Copy, Default, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SidechainBlockConfirmation {
    pub block_number: SidechainBlockNumber,
    pub block_header_hash: H256,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Enclave {
    pub worker_type: WorkerType,
    pub worker_mode: WorkerMode,
    pub mrenclave: MrEnclave,
    pub last_seen_timestamp: u64, // unix epoch in milliseconds when it's last seen
    pub url: Vec<u8>,             // utf8 encoded url
    pub shielding_pubkey: Option<Vec<u8>>, // JSON serialised enclave shielding pub key
    pub vc_pubkey: Option<Ed25519Public>,
    pub sgx_build_mode: SgxBuildMode,
    pub attestation_type: AttestationType,
}

impl Enclave {
    pub fn new(worker_type: WorkerType) -> Self {
        Enclave {
            worker_type,
            ..Default::default()
        }
    }

    pub fn with_worker_mode(mut self, worker_mode: WorkerMode) -> Self {
        self.worker_mode = worker_mode;
        self
    }

    pub fn with_mrenclave(mut self, mrenclave: MrEnclave) -> Self {
        self.mrenclave = mrenclave;
        self
    }

    pub fn with_url(mut self, url: Vec<u8>) -> Self {
        self.url = url;
        self
    }

    pub fn with_shielding_pubkey(mut self, shielding_pubkey: Option<Vec<u8>>) -> Self {
        self.shielding_pubkey = shielding_pubkey;
        self
    }

    pub fn with_vc_pubkey(mut self, vc_pubkey: Option<Ed25519Public>) -> Self {
        self.vc_pubkey = vc_pubkey;
        self
    }

    pub fn with_last_seen_timestamp(mut self, t: u64) -> Self {
        self.last_seen_timestamp = t;
        self
    }

    pub fn with_attestation_type(mut self, attestation_type: AttestationType) -> Self {
        self.attestation_type = attestation_type;
        self
    }

    pub fn with_sgx_build_mode(mut self, sgx_build_mode: SgxBuildMode) -> Self {
        self.sgx_build_mode = sgx_build_mode;
        self
    }
}

// use the name `RsaRequest` to differentiate from `AesRequest` (see aes_request.rs in
// tee-worker) `Rsa` implies that the payload is RSA-encrypted (using enclave's shielding key)
#[macro_export]
macro_rules! decl_rsa_request {
    ($($t:meta),*) => {
        #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, $($t),*)]
        pub struct RsaRequest {
            pub shard: ShardIdentifier,
            pub payload: Vec<u8>,
        }
        impl RsaRequest {
            pub fn new(shard: ShardIdentifier, payload: Vec<u8>) -> Self {
                Self { shard, payload }
            }
        }
    };
}

decl_rsa_request!(TypeInfo, RuntimeDebug);
