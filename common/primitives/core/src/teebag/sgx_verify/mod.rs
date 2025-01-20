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

//! Contains all the logic for understanding and verifying SGX remote attestation reports.
//!
//! Intel's documentation is scattered across different documents:
//!
//! "Intel® Software Guard Extensions: PCK Certificate and Certificate Revocation List Profile
//! Specification", further denoted as `PCK_Certificate_CRL_Spec-1.1`.
//!
//! * https://download.01.org/intel-sgx/dcap-1.2/linux/docs/Intel_SGX_PCK_Certificate_CRL_Spec-1.1.pdf
//!
//! Intel® SGX Developer Guide, further denoted as `SGX_Developer_Guide`:
//!
//! * https://download.01.org/intel-sgx/linux-1.5/docs/Intel_SGX_Developer_Guide.pdf

use self::{
    collateral::{EnclaveIdentity, TcbInfo},
    netscape_comment::NetscapeComment,
    utils::length_from_raw_data,
};
use crate::{
    Cpusvn, Fmspc, MrEnclave, MrSigner, Pcesvn, QuotingEnclave, SgxBuildMode, TcbVersionStatus,
};
use alloc::string::String;
use chrono::DateTime;
use core::time::Duration;
use der::asn1::ObjectIdentifier;
use frame_support::{ensure, traits::Len};
use parity_scale_codec::{Decode, Encode, Input};
use ring::signature::{self};
use scale_info::TypeInfo;
use serde_json::Value;
use sp_std::{
    convert::{TryFrom, TryInto},
    prelude::*,
    vec,
};
use x509_cert::Certificate;

pub mod collateral;
mod ephemeral_key;
mod netscape_comment;
#[cfg(test)]
mod tests;
mod utils;

const SGX_REPORT_DATA_SIZE: usize = 64;
#[derive(Debug, Encode, Decode, PartialEq, Eq, Copy, Clone, TypeInfo)]
#[repr(C)]
pub struct SgxReportData {
    d: [u8; SGX_REPORT_DATA_SIZE],
}

#[derive(Debug, Encode, Decode, PartialEq, Eq, Copy, Clone, TypeInfo)]
#[repr(C)]
pub struct SGXAttributes {
    flags: u64,
    xfrm: u64,
}

/// This is produced by an SGX platform, when it wants to be attested.
#[derive(Debug, Decode, Clone, TypeInfo)]
#[repr(C)]
pub struct DcapQuote {
    header: DcapQuoteHeader,
    body: SgxReportBody,
    signature_data_len: u32,
    quote_signature_data: EcdsaQuoteSignature,
}

/// All the documentation about this can be found in the `PCK_Certificate_CRL_Spec-1.1` page 62.
#[derive(Debug, Encode, Decode, Copy, Clone, TypeInfo)]
#[repr(C)]
pub struct DcapQuoteHeader {
    /// Version of the Quote data structure.
    ///
    /// This is version 3 for the DCAP ECDSA attestation.
    version: u16,
    /// Type of the Attestation Key used by the Quoting Enclave.
    /// • Supported values:
    /// - 2 (ECDSA-256-with-P-256 curve)
    /// - 3 (ECDSA-384-with-P-384 curve) (Note: currently not supported)
    attestation_key_type: u16,
    /// Reserved field, value 0.
    reserved: u32,
    /// Security Version of the Quoting Enclave currently loaded on the platform.
    qe_svn: u16,
    /// Security Version of the Provisioning Certification Enclave currently loaded on the
    /// platform.
    pce_svn: u16,
    /// Unique identifier of the QE Vendor.
    ///
    /// This will usually be Intel's Quoting enclave with the ID: 939A7233F79C4CA9940A0DB3957F0607.
    qe_vendor_id: [u8; 16],
    /// Custom user-defined data.
    user_data: [u8; 20],
}

const ATTESTATION_KEY_SIZE: usize = 64;
const REPORT_SIGNATURE_SIZE: usize = 64;

#[derive(Debug, Decode, Clone, TypeInfo)]
#[repr(C)]
pub struct EcdsaQuoteSignature {
    isv_enclave_report_signature: [u8; REPORT_SIGNATURE_SIZE],
    ecdsa_attestation_key: [u8; ATTESTATION_KEY_SIZE],
    qe_report: SgxReportBody,
    qe_report_signature: [u8; REPORT_SIGNATURE_SIZE],
    qe_authentication_data: QeAuthenticationData,
    qe_certification_data: QeCertificationData,
}

#[derive(Debug, Clone, TypeInfo)]
#[repr(C)]
pub struct QeAuthenticationData {
    size: u16,
    certification_data: Vec<u8>,
}

impl Decode for QeAuthenticationData {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let mut size_buf: [u8; 2] = [0; 2];
        input.read(&mut size_buf)?;
        let size = u16::from_le_bytes(size_buf);

        let mut certification_data = vec![0; size.into()];
        input.read(&mut certification_data)?;

        Ok(Self {
            size,
            certification_data,
        })
    }
}

#[derive(Debug, Clone, TypeInfo)]
#[repr(C)]
pub struct QeCertificationData {
    certification_data_type: u16,
    size: u32,
    certification_data: Vec<u8>,
}

impl Decode for QeCertificationData {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let mut certification_data_type_buf: [u8; 2] = [0; 2];
        input.read(&mut certification_data_type_buf)?;
        let certification_data_type = u16::from_le_bytes(certification_data_type_buf);

        let mut size_buf: [u8; 4] = [0; 4];
        input.read(&mut size_buf)?;
        let size = u32::from_le_bytes(size_buf);
        // This is an arbitrary limit to prevent out of memory issues. Intel does not specify a max
        // value
        if size > 65_000 {
            return Result::Err(parity_scale_codec::Error::from(
                "Certification data too long. Max 65000 bytes are allowed",
            ));
        }

        // Safety: The try_into() can only fail due to overflow on a 16-bit system, but we anyway
        // ensure the value is small enough above.
        let mut certification_data = vec![0; size.try_into().unwrap()];
        input.read(&mut certification_data)?;

        Ok(Self {
            certification_data_type,
            size,
            certification_data,
        })
    }
}

// see Intel SGX SDK https://github.com/intel/linux-sgx/blob/master/common/inc/sgx_report.h
const SGX_REPORT_BODY_RESERVED1_BYTES: usize = 12;
const SGX_REPORT_BODY_RESERVED2_BYTES: usize = 32;
const SGX_REPORT_BODY_RESERVED3_BYTES: usize = 32;
const SGX_REPORT_BODY_RESERVED4_BYTES: usize = 42;
const SGX_FLAGS_DEBUG: u64 = 0x0000000000000002;

/// SGX report about an enclave.
///
/// We don't verify all of the fields, as some contain business logic specific data that is
/// not related to the overall validity of an enclave. We only check security related fields. The
/// only exception to this is the quoting enclave, where we validate specific fields against known
/// values.
#[derive(Debug, Encode, Decode, Copy, Clone, TypeInfo)]
#[repr(C)]
pub struct SgxReportBody {
    /// Security version of the CPU.
    ///
    /// Reflects the processors microcode update version.
    cpu_svn: [u8; 16], /* (  0) Security Version of the CPU */
    /// State Save Area (SSA) extended feature set. Flags used for specific exception handling
    /// settings. Unless, you know what you are doing these should all be 0.
    ///
    /// See: https://cdrdv2-public.intel.com/671544/exception-handling-in-intel-sgx.pdf.
    misc_select: [u8; 4], /* ( 16) Which fields defined in SSA.MISC */
    /// Unused reserved bytes.
    reserved1: [u8; SGX_REPORT_BODY_RESERVED1_BYTES], /* ( 20) */
    /// Extended Product ID of an enclave.
    isv_ext_prod_id: [u8; 16], /* ( 32) ISV assigned Extended Product ID */
    /// Attributes, defines features that should be enabled for an enclave.
    ///
    /// Here, we only check if the Debug mode is enabled.
    ///
    /// More details in `SGX_Developer_Guide` under `Debug (Opt-in) Enclave Consideration` on page
    /// 24.
    attributes: SGXAttributes, /* ( 48) Any special Capabilities the Enclave possess */
    /// Enclave measurement.
    ///
    /// A single 256-bit hash that identifies the code and initial data to
    /// be placed inside the enclave, the expected order and position in which they are to be
    /// placed, and the security properties of those pages. More details in `SGX_Developer_Guide`
    /// page 6.
    mr_enclave: MrEnclave, /* ( 64) The value of the enclave's ENCLAVE measurement */
    /// Unused reserved bytes.
    reserved2: [u8; SGX_REPORT_BODY_RESERVED2_BYTES], /* ( 96) */
    /// The enclave author’s public key.
    ///
    /// More details in `SGX_Developer_Guide` page 6.
    mr_signer: MrSigner, /* (128) The value of the enclave's SIGNER measurement */
    /// Unused reserved bytes.
    reserved3: [u8; SGX_REPORT_BODY_RESERVED3_BYTES], /* (160) */
    /// Config ID of an enclave.
    ///
    /// Todo: #142 - Investigate the relevancy of this value.
    config_id: [u8; 64], /* (192) CONFIGID */
    /// The Product ID of the enclave.
    ///
    /// The Independent Software Vendor (ISV) should configure a unique ISVProdID for each product
    /// that may want to share sealed data between enclaves signed with a specific `MRSIGNER`.
    isv_prod_id: u16, /* (256) Product ID of the Enclave */
    /// ISV security version of the enclave.
    ///
    /// This is the enclave author's responsibility to increase it whenever a security related
    /// update happened. Here, we will only check it for the `Quoting Enclave` to ensure that the
    /// quoting enclave is recent enough.
    ///
    /// More details in `SGX_Developer_Guide` page 6.
    isv_svn: u16, /* (258) Security Version of the Enclave */
    /// Config Security version of the enclave.
    config_svn: u16, /* (260) CONFIGSVN */
    /// Unused reserved bytes.
    reserved4: [u8; SGX_REPORT_BODY_RESERVED4_BYTES], /* (262) */
    /// Family ID assigned by the ISV.
    ///
    /// Todo: #142 - Investigate the relevancy of this value.
    isv_family_id: [u8; 16], /* (304) ISV assigned Family ID */
    /// Custom data to be defined by the enclave author.
    ///
    /// We use this to provide the public key of the enclave that is to be registered on the chain.
    /// Doing this, will prove that the public key is from a legitimate SGX enclave when it is
    /// verified together with the remote attestation.
    report_data: SgxReportData, /* (320) Data provided by the user */
}

impl SgxReportBody {
    pub fn sgx_build_mode(&self) -> SgxBuildMode {
        if self.attributes.flags & SGX_FLAGS_DEBUG == SGX_FLAGS_DEBUG {
            SgxBuildMode::Debug
        } else {
            SgxBuildMode::Production
        }
    }

    fn verify_misc_select_field(&self, o: &QuotingEnclave) -> bool {
        for i in 0..self.misc_select.len() {
            if (self.misc_select[i] & o.miscselect_mask[i])
                != (o.miscselect[i] & o.miscselect_mask[i])
            {
                return false;
            }
        }
        true
    }

    fn verify_attributes_field(&self, o: &QuotingEnclave) -> bool {
        let attributes_flags = self.attributes.flags;

        let quoting_enclave_attributes_mask = o.attributes_flags_mask_as_u64();
        let quoting_enclave_attributes_flags = o.attributes_flags_as_u64();

        (attributes_flags & quoting_enclave_attributes_mask) == quoting_enclave_attributes_flags
    }

    pub fn verify(&self, o: &QuotingEnclave) -> bool {
        if self.isv_prod_id != o.isvprodid || self.mr_signer != o.mrsigner {
            return false;
        }
        if !self.verify_misc_select_field(o) {
            return false;
        }
        if !self.verify_attributes_field(o) {
            return false;
        }
        for tcb in &o.tcb {
            // If the enclave isvsvn is bigger than one of the
            if self.isv_svn >= tcb.isvsvn {
                return true;
            }
        }
        false
    }
}
// see Intel SGX SDK https://github.com/intel/linux-sgx/blob/master/common/inc/sgx_quote.h
#[derive(Encode, Decode, Copy, Clone, TypeInfo)]
#[repr(C)]
pub struct SgxQuote {
    version: u16,               /* 0 */
    sign_type: u16,             /* 2 */
    epid_group_id: u32,         /* 4 */
    qe_svn: u16,                /* 8 */
    pce_svn: u16,               /* 10 */
    xeid: u32,                  /* 12 */
    basename: [u8; 32],         /* 16 */
    report_body: SgxReportBody, /* 48 */
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo, Default)]
pub enum SgxStatus {
    #[default]
    #[codec(index = 0)]
    Invalid,
    #[codec(index = 1)]
    Ok,
    #[codec(index = 2)]
    GroupOutOfDate,
    #[codec(index = 3)]
    GroupRevoked,
    #[codec(index = 4)]
    ConfigurationNeeded,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
pub struct SgxReport {
    pub mr_enclave: MrEnclave,
    pub pubkey: [u8; 32],
    pub status: SgxStatus,
    pub timestamp: u64, // unix timestamp in milliseconds
    pub build_mode: SgxBuildMode,
}

type SignatureAlgorithms = &'static [&'static dyn webpki::types::SignatureVerificationAlgorithm];
static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
    webpki::ring::RSA_PKCS1_2048_8192_SHA256,
    webpki::ring::RSA_PKCS1_2048_8192_SHA384,
    webpki::ring::RSA_PKCS1_2048_8192_SHA512,
    webpki::ring::RSA_PKCS1_3072_8192_SHA384,
];

//pub const IAS_REPORT_CA: &[u8] = include_bytes!("../AttestationReportSigningCACert.pem");

pub static IAS_SERVER_ROOTS: &[webpki::types::TrustAnchor<'static>; 1] = &[
	/*
	 * -----BEGIN CERTIFICATE-----
	 * MIIFSzCCA7OgAwIBAgIJANEHdl0yo7CUMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNV
	 * BAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNV
	 * BAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0
	 * YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwIBcNMTYxMTE0MTUzNzMxWhgPMjA0OTEy
	 * MzEyMzU5NTlaMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwL
	 * U2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQD
	 * DCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwggGiMA0G
	 * CSqGSIb3DQEBAQUAA4IBjwAwggGKAoIBgQCfPGR+tXc8u1EtJzLA10Feu1Wg+p7e
	 * LmSRmeaCHbkQ1TF3Nwl3RmpqXkeGzNLd69QUnWovYyVSndEMyYc3sHecGgfinEeh
	 * rgBJSEdsSJ9FpaFdesjsxqzGRa20PYdnnfWcCTvFoulpbFR4VBuXnnVLVzkUvlXT
	 * L/TAnd8nIZk0zZkFJ7P5LtePvykkar7LcSQO85wtcQe0R1Raf/sQ6wYKaKmFgCGe
	 * NpEJUmg4ktal4qgIAxk+QHUxQE42sxViN5mqglB0QJdUot/o9a/V/mMeH8KvOAiQ
	 * byinkNndn+Bgk5sSV5DFgF0DffVqmVMblt5p3jPtImzBIH0QQrXJq39AT8cRwP5H
	 * afuVeLHcDsRp6hol4P+ZFIhu8mmbI1u0hH3W/0C2BuYXB5PC+5izFFh/nP0lc2Lf
	 * 6rELO9LZdnOhpL1ExFOq9H/B8tPQ84T3Sgb4nAifDabNt/zu6MmCGo5U8lwEFtGM
	 * RoOaX4AS+909x00lYnmtwsDVWv9vBiJCXRsCAwEAAaOByTCBxjBgBgNVHR8EWTBX
	 * MFWgU6BRhk9odHRwOi8vdHJ1c3RlZHNlcnZpY2VzLmludGVsLmNvbS9jb250ZW50
	 * L0NSTC9TR1gvQXR0ZXN0YXRpb25SZXBvcnRTaWduaW5nQ0EuY3JsMB0GA1UdDgQW
	 * BBR4Q3t2pn680K9+QjfrNXw7hwFRPDAfBgNVHSMEGDAWgBR4Q3t2pn680K9+Qjfr
	 * NXw7hwFRPDAOBgNVHQ8BAf8EBAMCAQYwEgYDVR0TAQH/BAgwBgEB/wIBADANBgkq
	 * hkiG9w0BAQsFAAOCAYEAeF8tYMXICvQqeXYQITkV2oLJsp6J4JAqJabHWxYJHGir
	 * IEqucRiJSSx+HjIJEUVaj8E0QjEud6Y5lNmXlcjqRXaCPOqK0eGRz6hi+ripMtPZ
	 * sFNaBwLQVV905SDjAzDzNIDnrcnXyB4gcDFCvwDFKKgLRjOB/WAqgscDUoGq5ZVi
	 * zLUzTqiQPmULAQaB9c6Oti6snEFJiCQ67JLyW/E83/frzCmO5Ru6WjU4tmsmy8Ra
	 * Ud4APK0wZTGtfPXU7w+IBdG5Ez0kE1qzxGQaL4gINJ1zMyleDnbuS8UicjJijvqA
	 * 152Sq049ESDz+1rRGc2NVEqh1KaGXmtXvqxXcTB+Ljy5Bw2ke0v8iGngFBPqCTVB
	 * 3op5KBG3RjbF6RRSzwzuWfL7QErNC8WEy5yDVARzTA5+xmBc388v9Dm21HGfcC8O
	 * DD+gT9sSpssq0ascmvH49MOgjt1yoysLtdCtJW/9FZpoOypaHx0R+mJTLwPXVMrv
	 * DaVzWh5aiEx+idkSGMnX
	 * -----END CERTIFICATE-----
	 */
	webpki::types::TrustAnchor {
		subject: webpki::types::Der::from_slice(b"1\x0b0\t\x06\x03U\x04\x06\x13\x02US1\x0b0\t\x06\x03U\x04\x08\x0c\x02CA1\x140\x12\x06\x03U\x04\x07\x0c\x0bSanta Clara1\x1a0\x18\x06\x03U\x04\n\x0c\x11Intel Corporation100.\x06\x03U\x04\x03\x0c\'Intel SGX Attestation Report Signing CA"),
		subject_public_key_info: webpki::types::Der::from_slice(b"0\r\x06\t*\x86H\x86\xf7\r\x01\x01\x01\x05\x00\x03\x82\x01\x8f\x000\x82\x01\x8a\x02\x82\x01\x81\x00\x9f<d~\xb5w<\xbbQ-\'2\xc0\xd7A^\xbbU\xa0\xfa\x9e\xde.d\x91\x99\xe6\x82\x1d\xb9\x10\xd51w7\twFjj^G\x86\xcc\xd2\xdd\xeb\xd4\x14\x9dj/c%R\x9d\xd1\x0c\xc9\x877\xb0w\x9c\x1a\x07\xe2\x9cG\xa1\xae\x00IHGlH\x9fE\xa5\xa1]z\xc8\xec\xc6\xac\xc6E\xad\xb4=\x87g\x9d\xf5\x9c\t;\xc5\xa2\xe9ilTxT\x1b\x97\x9euKW9\x14\xbeU\xd3/\xf4\xc0\x9d\xdf\'!\x994\xcd\x99\x05\'\xb3\xf9.\xd7\x8f\xbf)$j\xbe\xcbq$\x0e\xf3\x9c-q\x07\xb4GTZ\x7f\xfb\x10\xeb\x06\nh\xa9\x85\x80!\x9e6\x91\tRh8\x92\xd6\xa5\xe2\xa8\x08\x03\x19>@u1@N6\xb3\x15b7\x99\xaa\x82Pt@\x97T\xa2\xdf\xe8\xf5\xaf\xd5\xfec\x1e\x1f\xc2\xaf8\x08\x90o(\xa7\x90\xd9\xdd\x9f\xe0`\x93\x9b\x12W\x90\xc5\x80]\x03}\xf5j\x99S\x1b\x96\xdei\xde3\xed\"l\xc1 }\x10B\xb5\xc9\xab\x7f@O\xc7\x11\xc0\xfeGi\xfb\x95x\xb1\xdc\x0e\xc4i\xea\x1a%\xe0\xff\x99\x14\x88n\xf2i\x9b#[\xb4\x84}\xd6\xff@\xb6\x06\xe6\x17\x07\x93\xc2\xfb\x98\xb3\x14X\x7f\x9c\xfd%sb\xdf\xea\xb1\x0b;\xd2\xd9vs\xa1\xa4\xbdD\xc4S\xaa\xf4\x7f\xc1\xf2\xd3\xd0\xf3\x84\xf7J\x06\xf8\x9c\x08\x9f\r\xa6\xcd\xb7\xfc\xee\xe8\xc9\x82\x1a\x8eT\xf2\\\x04\x16\xd1\x8cF\x83\x9a_\x80\x12\xfb\xdd=\xc7M%by\xad\xc2\xc0\xd5Z\xffo\x06\"B]\x1b\x02\x03\x01\x00\x01"),
		name_constraints: None
	},
];

/// The needed code for a trust anchor can be extracted using `webpki` with something like this:
/// println!("{:?}", webpki::TrustAnchor::try_from_cert_der(&root_cert));
#[allow(clippy::zero_prefixed_literal)]
pub static DCAP_SERVER_ROOTS: &[webpki::types::TrustAnchor<'static>; 1] =
    &[webpki::types::TrustAnchor {
        subject: webpki::types::Der::from_slice(&[
            49, 26, 48, 24, 06, 03, 85, 04, 03, 12, 17, 73, 110, 116, 101, 108, 32, 83, 71, 88, 32,
            82, 111, 111, 116, 32, 67, 65, 49, 26, 48, 24, 06, 03, 85, 04, 10, 12, 17, 73, 110,
            116, 101, 108, 32, 67, 111, 114, 112, 111, 114, 97, 116, 105, 111, 110, 49, 20, 48, 18,
            06, 03, 85, 04, 07, 12, 11, 83, 97, 110, 116, 97, 32, 67, 108, 97, 114, 97, 49, 11, 48,
            09, 06, 03, 85, 04, 08, 12, 02, 67, 65, 49, 11, 48, 09, 06, 03, 85, 04, 06, 19, 02, 85,
            83,
        ]),
        subject_public_key_info: webpki::types::Der::from_slice(&[
            48, 19, 06, 07, 42, 134, 72, 206, 61, 02, 01, 06, 08, 42, 134, 72, 206, 61, 03, 01, 07,
            03, 66, 00, 04, 11, 169, 196, 192, 192, 200, 97, 147, 163, 254, 35, 214, 176, 44, 218,
            16, 168, 187, 212, 232, 142, 72, 180, 69, 133, 97, 163, 110, 112, 85, 37, 245, 103,
            145, 142, 46, 220, 136, 228, 13, 134, 11, 208, 204, 78, 226, 106, 172, 201, 136, 229,
            05, 169, 83, 85, 140, 69, 63, 107, 09, 04, 174, 115, 148,
        ]),
        name_constraints: None,
    }];

/// Contains an unvalidated ias remote attestation certificate.
///
/// Wrapper to implemented parsing and verification traits on it.
pub struct CertDer<'a>(&'a [u8]);

/// Encode two 32-byte values in DER format
/// This is meant for 256 bit ECC signatures or public keys
pub fn encode_as_der(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.len() != 64 {
        return Result::Err("Key must be 64 bytes long");
    }
    let mut sequence = der::asn1::SequenceOf::<der::asn1::UIntRef, 2>::new();
    sequence
        .add(der::asn1::UIntRef::new(&data[0..32]).map_err(|_| "Invalid public key")?)
        .map_err(|_| "Invalid public key")?;
    sequence
        .add(der::asn1::UIntRef::new(&data[32..]).map_err(|_| "Invalid public key")?)
        .map_err(|_| "Invalid public key")?;
    // 72 should be enough in all cases. 2 + 2 x (32 + 3)
    let mut asn1 = vec![0u8; 72];
    let mut writer = der::SliceWriter::new(&mut asn1);
    writer
        .encode(&sequence)
        .map_err(|_| "Could not encode public key to DER")?;
    Ok(writer
        .finish()
        .map_err(|_| "Could not convert public key to DER")?
        .to_vec())
}

/// Extracts the specified data into a `EnclaveIdentity` instance.
/// Also verifies that the data matches the given signature, was produced by the given certificate
/// and matches the data
pub fn deserialize_enclave_identity(
    data: &[u8],
    signature: &[u8],
    certificate: &webpki::EndEntityCert,
) -> Result<EnclaveIdentity, &'static str> {
    let signature = encode_as_der(signature)?;
    verify_signature(
        certificate,
        data,
        &signature,
        webpki::ring::ECDSA_P256_SHA256,
    )?;
    serde_json::from_slice(data).map_err(|_| "Deserialization failed")
}

/// Extracts the specified data into a `TcbInfo` instance.
/// Also verifies that the data matches the given signature, was produced by the given certificate
/// and matches the data
pub fn deserialize_tcb_info(
    data: &[u8],
    signature: &[u8],
    certificate: &webpki::EndEntityCert,
) -> Result<TcbInfo, &'static str> {
    let signature = encode_as_der(signature)?;
    verify_signature(
        certificate,
        data,
        &signature,
        webpki::ring::ECDSA_P256_SHA256,
    )?;
    serde_json::from_slice(data).map_err(|_| "Deserialization failed")
}

/// Extract a list of certificates from a byte vec. The certificates must be separated by
/// `-----BEGIN CERTIFICATE-----` and `-----END CERTIFICATE-----` markers
pub fn extract_certs(cert_chain: &[u8]) -> Vec<Vec<u8>> {
    // The certificates should be valid UTF-8 but if not we continue. The certificate verification
    // will fail at a later point.
    let certs_concat = String::from_utf8_lossy(cert_chain);
    let certs_concat = certs_concat.replace('\n', "");
    let certs_concat = certs_concat.replace("-----BEGIN CERTIFICATE-----", "");
    // Use the end marker to split the string into certificates
    let parts = certs_concat.split("-----END CERTIFICATE-----");
    parts
        .filter(|p| !p.is_empty())
        .filter_map(|p| base64::decode(p).ok())
        .collect()
}

/// Verifies that the `leaf_cert` in combination with the `intermediate_certs` establishes
/// a valid certificate chain that is rooted in one of the trust anchors that was compiled into to
/// the pallet
pub fn verify_certificate_chain<'a>(
    leaf_cert: &webpki::EndEntityCert<'a>,
    intermediate_certs: &[webpki::types::CertificateDer<'a>],
    verification_time: u64,
) -> Result<(), &'static str> {
    let time =
        webpki::types::UnixTime::since_unix_epoch(Duration::from_secs(verification_time / 1000));
    let sig_algs = &[webpki::ring::ECDSA_P256_SHA256];
    leaf_cert
        .verify_for_usage(
            sig_algs,
            DCAP_SERVER_ROOTS,
            intermediate_certs,
            time,
            webpki::KeyUsage::client_auth(),
            None,
        )
        .map_err(|_| "Invalid certificate chain")?;
    Ok(())
}
#[allow(unused)]
pub fn extract_tcb_info_from_raw_dcap_quote(
    dcap_quote_raw: &[u8],
) -> Result<(Fmspc, TcbVersionStatus), &'static str> {
    let mut dcap_quote_clone = dcap_quote_raw;
    let quote: DcapQuote =
        Decode::decode(&mut dcap_quote_clone).map_err(|_| "Failed to decode attestation report")?;

    ensure!(quote.header.version == 3, "Only support for version 3");
    ensure!(
        quote.header.attestation_key_type == 2,
        "Only support for ECDSA-256"
    );
    ensure!(
        quote
            .quote_signature_data
            .qe_certification_data
            .certification_data_type
            == 5,
        "Only support for PEM formatted PCK Cert Chain"
    );

    let certs = extract_certs(
        &quote
            .quote_signature_data
            .qe_certification_data
            .certification_data,
    );

    let (fmspc, tcb_info) = extract_tcb_info(&certs[0])?;

    Ok((fmspc, tcb_info))
}

pub fn verify_dcap_quote(
    dcap_quote_raw: &[u8],
    verification_time: u64,
    qe: &QuotingEnclave,
) -> Result<(Fmspc, TcbVersionStatus, SgxReport), &'static str> {
    let mut dcap_quote_clone = dcap_quote_raw;
    let quote: DcapQuote =
        Decode::decode(&mut dcap_quote_clone).map_err(|_| "Failed to decode attestation report")?;

    ensure!(quote.header.version == 3, "Only support for version 3");
    ensure!(
        quote.header.attestation_key_type == 2,
        "Only support for ECDSA-256"
    );
    ensure!(
        quote
            .quote_signature_data
            .qe_certification_data
            .certification_data_type
            == 5,
        "Only support for PEM formatted PCK Cert Chain"
    );
    ensure!(
        quote.quote_signature_data.qe_report.verify(qe),
        "Enclave rejected by quoting enclave"
    );
    let mut xt_signer_array = [0u8; 32];
    xt_signer_array.copy_from_slice(&quote.body.report_data.d[..32]);

    let certs = extract_certs(
        &quote
            .quote_signature_data
            .qe_certification_data
            .certification_data,
    );
    ensure!(
        certs.len() >= 2,
        "Certificate chain must have at least two certificates"
    );
    let intermediate_certificate_slices: Vec<webpki::types::CertificateDer> =
        certs[1..].iter().map(|c| c.as_slice().into()).collect();
    let leaf_cert_der = webpki::types::CertificateDer::from(certs[0].as_slice());
    let leaf_cert = webpki::EndEntityCert::try_from(&leaf_cert_der)
        .map_err(|_| "Failed to parse leaf certificate")?;
    verify_certificate_chain(
        &leaf_cert,
        &intermediate_certificate_slices,
        verification_time,
    )?;

    let (fmspc, tcb_info) = extract_tcb_info(&certs[0])?;

    // For this part some understanding of the document (Especially chapter A.4: Quote Format)
    // Intel® Software Guard Extensions (Intel® SGX) Data Center Attestation Primitives: ECDSA Quote
    // Library API https://download.01.org/intel-sgx/latest/dcap-latest/linux/docs/Intel_SGX_ECDSA_QuoteLibReference_DCAP_API.pdf

    const AUTHENTICATION_DATA_SIZE: usize = 32; // This is actually variable but assume 32 for now. This is also hard-coded to 32 in the Intel
                                                // DCAP repo
    const DCAP_QUOTE_HEADER_SIZE: usize = core::mem::size_of::<DcapQuoteHeader>();
    const REPORT_SIZE: usize = core::mem::size_of::<SgxReportBody>();
    const QUOTE_SIGNATURE_DATA_LEN_SIZE: usize = core::mem::size_of::<u32>();

    let attestation_key_offset = DCAP_QUOTE_HEADER_SIZE
        + REPORT_SIZE
        + QUOTE_SIGNATURE_DATA_LEN_SIZE
        + REPORT_SIGNATURE_SIZE;
    let authentication_data_offset = attestation_key_offset
        + ATTESTATION_KEY_SIZE
        + REPORT_SIZE
        + REPORT_SIGNATURE_SIZE
        + core::mem::size_of::<u16>(); //Size of the QE authentication data. We ignore this for now and assume 32. See
                                       // AUTHENTICATION_DATA_SIZE
    let mut hash_data = [0u8; ATTESTATION_KEY_SIZE + AUTHENTICATION_DATA_SIZE];
    hash_data[0..ATTESTATION_KEY_SIZE].copy_from_slice(
        &dcap_quote_raw[attestation_key_offset..(attestation_key_offset + ATTESTATION_KEY_SIZE)],
    );
    hash_data[ATTESTATION_KEY_SIZE..].copy_from_slice(
        &dcap_quote_raw
            [authentication_data_offset..(authentication_data_offset + AUTHENTICATION_DATA_SIZE)],
    );
    // Ensure that the hash matches the intel signed hash in the QE report. This establishes trust
    // into the attestation key.
    let hash = ring::digest::digest(&ring::digest::SHA256, &hash_data);
    ensure!(
        hash.as_ref() == &quote.quote_signature_data.qe_report.report_data.d[0..32],
        "Hashes must match"
    );

    let qe_report_offset = attestation_key_offset + ATTESTATION_KEY_SIZE;
    let qe_report_slice = &dcap_quote_raw[qe_report_offset..(qe_report_offset + REPORT_SIZE)];
    let mut pub_key = [0x04u8; 65]; //Prepend 0x04 to specify uncompressed format
    pub_key[1..].copy_from_slice(&quote.quote_signature_data.ecdsa_attestation_key);

    let peer_public_key =
        signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, pub_key);
    let isv_report_slice = &dcap_quote_raw[0..(DCAP_QUOTE_HEADER_SIZE + REPORT_SIZE)];
    // Verify that the enclave data matches the signature generated by the trusted attestation key.
    // This establishes trust into the data of the enclave we actually want to verify
    peer_public_key
        .verify(
            isv_report_slice,
            &quote.quote_signature_data.isv_enclave_report_signature,
        )
        .map_err(|_| "Failed to verify report signature")?;

    // Verify that the QE report was signed by Intel. This establishes trust into the QE report.
    let asn1_signature = encode_as_der(&quote.quote_signature_data.qe_report_signature)?;
    verify_signature(
        &leaf_cert,
        qe_report_slice,
        &asn1_signature,
        webpki::ring::ECDSA_P256_SHA256,
    )?;

    ensure!(
        dcap_quote_clone.is_empty(),
        "There should be no bytes left over after decoding"
    );
    let report = SgxReport {
        mr_enclave: quote.body.mr_enclave,
        status: SgxStatus::Ok,
        pubkey: xt_signer_array,
        timestamp: verification_time,
        build_mode: quote.body.sgx_build_mode(),
    };
    Ok((fmspc, tcb_info, report))
}

// make sure this function doesn't panic!
pub fn verify_ias_report(cert_der: &[u8]) -> Result<SgxReport, &'static str> {
    // Before we reach here, the runtime already verified the extrinsic is properly signed by the
    // extrinsic sender Hence, we skip: EphemeralKey::try_from(cert)?;
    let cert = CertDer(cert_der);
    let netscape = NetscapeComment::try_from(cert)?;
    let sig_cert_der = webpki::types::CertificateDer::from(netscape.sig_cert.as_slice());
    let sig_cert = webpki::EndEntityCert::try_from(&sig_cert_der).map_err(|_| "Bad der")?;

    verify_signature(
        &sig_cert,
        netscape.attestation_raw,
        &netscape.sig,
        webpki::ring::RSA_PKCS1_2048_8192_SHA256,
    )?;

    // FIXME: now hardcoded. but certificate renewal would have to be done manually anyway...
    // chain wasm update or by some sudo call
    let valid_until = webpki::types::UnixTime::since_unix_epoch(Duration::from_secs(1573419050));
    verify_server_cert(&sig_cert, valid_until)?;

    parse_report(&netscape)
}

fn parse_report(netscape: &NetscapeComment) -> Result<SgxReport, &'static str> {
    let report_raw: &[u8] = netscape.attestation_raw;
    // parse attestation report
    let attn_report: Value = match serde_json::from_slice(report_raw) {
        Ok(report) => report,
        Err(_) => return Err("RA report parsing error"),
    };

    let _ra_timestamp = match &attn_report["timestamp"] {
        Value::String(time) => {
            let time_fixed = time.clone() + "+0000";
            match DateTime::parse_from_str(&time_fixed, "%Y-%m-%dT%H:%M:%S%.f%z") {
                Ok(d) => d.timestamp(),
                Err(_) => return Err("RA report timestamp parsing error"),
            }
        }
        _ => return Err("Failed to fetch timestamp from attestation report"),
    };

    // in milliseconds
    let ra_timestamp: u64 = (_ra_timestamp * 1000)
        .try_into()
        .map_err(|_| "Error converting report.timestamp to u64")?;

    // get quote status (mandatory field)
    let ra_status = match &attn_report["isvEnclaveQuoteStatus"] {
        Value::String(quote_status) => match quote_status.as_ref() {
            "OK" => SgxStatus::Ok,
            "GROUP_OUT_OF_DATE" => SgxStatus::GroupOutOfDate,
            "GROUP_REVOKED" => SgxStatus::GroupRevoked,
            "CONFIGURATION_NEEDED" => SgxStatus::ConfigurationNeeded,
            _ => SgxStatus::Invalid,
        },
        _ => return Err("Failed to fetch isvEnclaveQuoteStatus from attestation report"),
    };

    // parse quote body
    if let Value::String(quote_raw) = &attn_report["isvEnclaveQuoteBody"] {
        let quote = match base64::decode(quote_raw) {
            Ok(q) => q,
            Err(_) => return Err("Quote Decoding Error"),
        };
        // TODO: lack security check here
        let sgx_quote: SgxQuote = match Decode::decode(&mut &quote[..]) {
            Ok(q) => q,
            Err(_) => return Err("could not decode quote"),
        };

        let mut xt_signer_array = [0u8; 32];
        xt_signer_array.copy_from_slice(&sgx_quote.report_body.report_data.d[..32]);
        Ok(SgxReport {
            mr_enclave: sgx_quote.report_body.mr_enclave,
            status: ra_status,
            pubkey: xt_signer_array,
            timestamp: ra_timestamp,
            build_mode: sgx_quote.report_body.sgx_build_mode(),
        })
    } else {
        Err("Failed to parse isvEnclaveQuoteBody from attestation report")
    }
}

/// * `signature` - Must be encoded in DER format.
pub fn verify_signature(
    entity_cert: &webpki::EndEntityCert,
    data: &[u8],
    signature: &[u8],
    signature_algorithm: &dyn webpki::types::SignatureVerificationAlgorithm,
) -> Result<(), &'static str> {
    match entity_cert.verify_signature(signature_algorithm, data, signature) {
        Ok(()) => Ok(()),
        Err(_e) => Err("bad signature"),
    }
}

pub fn verify_server_cert(
    sig_cert: &webpki::EndEntityCert,
    timestamp_valid_until: webpki::types::UnixTime,
) -> Result<(), &'static str> {
    let chain: Vec<webpki::types::CertificateDer> = Vec::new();
    match sig_cert.verify_for_usage(
        SUPPORTED_SIG_ALGS,
        IAS_SERVER_ROOTS,
        &chain,
        timestamp_valid_until,
        webpki::KeyUsage::server_auth(),
        None,
    ) {
        Ok(()) => Ok(()),
        Err(_e) => Err("CA verification failed"),
    }
}

/// See document "Intel® Software Guard Extensions: PCK Certificate and Certificate Revocation List
/// Profile Specification" https://download.01.org/intel-sgx/dcap-1.2/linux/docs/Intel_SGX_PCK_Certificate_CRL_Spec-1.1.pdf
const INTEL_SGX_EXTENSION_OID: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113741.1.13.1");
const OID_FMSPC: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113741.1.13.1.4");
const OID_PCESVN: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113741.1.13.1.2.17");
const OID_CPUSVN: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113741.1.13.1.2.18");

pub fn extract_tcb_info(cert: &[u8]) -> Result<(Fmspc, TcbVersionStatus), &'static str> {
    let extension_section = get_intel_extension(cert)?;

    let fmspc = get_fmspc(&extension_section)?;
    let cpusvn = get_cpusvn(&extension_section)?;
    let pcesvn = get_pcesvn(&extension_section)?;

    Ok((fmspc, TcbVersionStatus::new(cpusvn, pcesvn)))
}

fn get_intel_extension(der_encoded: &[u8]) -> Result<Vec<u8>, &'static str> {
    let cert: Certificate =
        der::Decode::from_der(der_encoded).map_err(|_| "Error parsing certificate")?;
    let mut extension_iter = cert
        .tbs_certificate
        .extensions
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|e| e.extn_id == INTEL_SGX_EXTENSION_OID)
        .map(|e| e.extn_value);

    let extension = extension_iter.next();
    ensure!(
        extension.is_some() && extension_iter.next().is_none(),
        "There should only be one section containing Intel extensions"
    );
    // SAFETY: Ensured above that extension.is_some() == true
    Ok(extension.unwrap().to_vec())
}

fn get_fmspc(der: &[u8]) -> Result<Fmspc, &'static str> {
    let bytes_oid = OID_FMSPC.as_bytes();
    let mut offset = der
        .windows(bytes_oid.len())
        .position(|window| window == bytes_oid)
        .ok_or("Certificate does not contain 'FMSPC_OID'")?;
    offset += 12; // length oid (10) + asn1 tag (1) + asn1 length10 (1)

    let fmspc_size = core::mem::size_of::<Fmspc>() / core::mem::size_of::<u8>();
    let data = der
        .get(offset..offset + fmspc_size)
        .ok_or("Index out of bounds")?;
    data.try_into().map_err(|_| "FMSPC must be 6 bytes long")
}

fn get_cpusvn(der: &[u8]) -> Result<Cpusvn, &'static str> {
    let bytes_oid = OID_CPUSVN.as_bytes();
    let mut offset = der
        .windows(bytes_oid.len())
        .position(|window| window == bytes_oid)
        .ok_or("Certificate does not contain 'CPUSVN_OID'")?;
    offset += 13; // length oid (11) + asn1 tag (1) + asn1 length10 (1)

    // CPUSVN is specified to have length 16
    let len = 16;
    let data = der.get(offset..offset + len).ok_or("Index out of bounds")?;
    data.try_into().map_err(|_| "CPUSVN must be 16 bytes long")
}

fn get_pcesvn(der: &[u8]) -> Result<Pcesvn, &'static str> {
    let bytes_oid = OID_PCESVN.as_bytes();
    let mut offset = der
        .windows(bytes_oid.len())
        .position(|window| window == bytes_oid)
        .ok_or("Certificate does not contain 'PCESVN_OID'")?;
    // length oid + asn1 tag (1 byte)
    offset += bytes_oid.len() + 1;
    // PCESVN can be 1 or 2 bytes
    let len = length_from_raw_data(der, &mut offset)?;
    offset += 1; // length_from_raw_data does not move the offset when the length is encoded in a single byte
    ensure!(len == 1 || len == 2, "PCESVN must be 1 or 2 bytes");
    let data = der.get(offset..offset + len).ok_or("Index out of bounds")?;
    if data.len() == 1 {
        Ok(u16::from(data[0]))
    } else {
        // Unwrap is fine here as we check the length above
        // DER integers are encoded in big endian
        Ok(u16::from_be_bytes(data.try_into().unwrap()))
    }
}
