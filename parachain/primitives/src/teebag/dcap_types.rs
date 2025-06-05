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

use crate::{MrEnclave, MrSigner, QuotingEnclave, SgxBuildMode};
use alloc::vec;
use parity_scale_codec::{Decode, Encode, Input};
use scale_info::TypeInfo;

const SGX_REPORT_DATA_SIZE: usize = 64;
#[derive(Debug, Encode, Decode, PartialEq, Eq, Copy, Clone, TypeInfo)]
#[repr(C)]
pub struct SgxReportData {
	pub d: [u8; SGX_REPORT_DATA_SIZE],
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
	pub header: DcapQuoteHeader,
	pub body: SgxReportBody,
	pub signature_data_len: u32,
	pub quote_signature_data: EcdsaQuoteSignature,
}

/// All the documentation about this can be found in the `PCK_Certificate_CRL_Spec-1.1` page 62.
#[derive(Debug, Encode, Decode, Copy, Clone, TypeInfo)]
#[repr(C)]
pub struct DcapQuoteHeader {
	/// Version of the Quote data structure.
	///
	/// This is version 3 for the DCAP ECDSA attestation.
	pub version: u16,
	/// Type of the Attestation Key used by the Quoting Enclave.
	/// • Supported values:
	/// - 2 (ECDSA-256-with-P-256 curve)
	/// - 3 (ECDSA-384-with-P-384 curve) (Note: currently not supported)
	pub attestation_key_type: u16,
	/// Reserved field, value 0.
	pub reserved: u32,
	/// Security Version of the Quoting Enclave currently loaded on the platform.
	pub qe_svn: u16,
	/// Security Version of the Provisioning Certification Enclave currently loaded on the
	/// platform.
	pub pce_svn: u16,
	/// Unique identifier of the QE Vendor.
	///
	/// This will usually be Intel's Quoting enclave with the ID: 939A7233F79C4CA9940A0DB3957F0607.
	pub qe_vendor_id: [u8; 16],
	/// Custom user-defined data.
	pub user_data: [u8; 20],
}

pub const ATTESTATION_KEY_SIZE: usize = 64;
pub const REPORT_SIGNATURE_SIZE: usize = 64;

#[derive(Debug, Decode, Clone, TypeInfo)]
#[repr(C)]
pub struct EcdsaQuoteSignature {
	pub isv_enclave_report_signature: [u8; REPORT_SIGNATURE_SIZE],
	pub ecdsa_attestation_key: [u8; ATTESTATION_KEY_SIZE],
	pub qe_report: SgxReportBody,
	pub qe_report_signature: [u8; REPORT_SIGNATURE_SIZE],
	pub qe_authentication_data: QeAuthenticationData,
	pub qe_certification_data: QeCertificationData,
}

#[derive(Debug, Clone, TypeInfo)]
#[repr(C)]
pub struct QeAuthenticationData {
	pub size: u16,
	pub certification_data: vec::Vec<u8>,
}

impl Decode for QeAuthenticationData {
	fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
		let mut size_buf: [u8; 2] = [0; 2];
		input.read(&mut size_buf)?;
		let size = u16::from_le_bytes(size_buf);

		let mut certification_data = vec![0; size.into()];
		input.read(&mut certification_data)?;

		Ok(Self { size, certification_data })
	}
}

#[derive(Debug, Clone, TypeInfo)]
#[repr(C)]
pub struct QeCertificationData {
	pub certification_data_type: u16,
	pub size: u32,
	pub certification_data: vec::Vec<u8>,
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

		Ok(Self { certification_data_type, size, certification_data })
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
	pub cpu_svn: [u8; 16], /* (  0) Security Version of the CPU */
	/// State Save Area (SSA) extended feature set. Flags used for specific exception handling
	/// settings. Unless, you know what you are doing these should all be 0.
	///
	/// See: https://cdrdv2-public.intel.com/671544/exception-handling-in-intel-sgx.pdf.
	pub misc_select: [u8; 4], /* ( 16) Which fields defined in SSA.MISC */
	/// Unused reserved bytes.
	pub reserved1: [u8; SGX_REPORT_BODY_RESERVED1_BYTES], /* ( 20) */
	/// Extended Product ID of an enclave.
	pub isv_ext_prod_id: [u8; 16], /* ( 32) ISV assigned Extended Product ID */
	/// Attributes, defines features that should be enabled for an enclave.
	///
	/// Here, we only check if the Debug mode is enabled.
	///
	/// More details in `SGX_Developer_Guide` under `Debug (Opt-in) Enclave Consideration` on page
	/// 24.
	pub attributes: SGXAttributes, /* ( 48) Any special Capabilities the Enclave possess */
	/// Enclave measurement.
	///
	/// A single 256-bit hash that identifies the code and initial data to
	/// be placed inside the enclave, the expected order and position in which they are to be
	/// placed, and the security properties of those pages. More details in `SGX_Developer_Guide`
	/// page 6.
	pub mr_enclave: MrEnclave, /* ( 64) The value of the enclave's ENCLAVE measurement */
	/// Unused reserved bytes.
	pub reserved2: [u8; SGX_REPORT_BODY_RESERVED2_BYTES], /* ( 96) */
	/// The enclave author’s public key.
	///
	/// More details in `SGX_Developer_Guide` page 6.
	pub mr_signer: MrSigner, /* (128) The value of the enclave's SIGNER measurement */
	/// Unused reserved bytes.
	pub reserved3: [u8; SGX_REPORT_BODY_RESERVED3_BYTES], /* (160) */
	/// Config ID of an enclave.
	///
	/// Todo: #142 - Investigate the relevancy of this value.
	pub config_id: [u8; 64], /* (192) CONFIGID */
	/// The Product ID of the enclave.
	///
	/// The Independent Software Vendor (ISV) should configure a unique ISVProdID for each product
	/// that may want to share sealed data between enclaves signed with a specific `MRSIGNER`.
	pub isv_prod_id: u16, /* (256) Product ID of the Enclave */
	/// ISV security version of the enclave.
	///
	/// This is the enclave author's responsibility to increase it whenever a security related
	/// update happened. Here, we will only check it for the `Quoting Enclave` to ensure that the
	/// quoting enclave is recent enough.
	///
	/// More details in `SGX_Developer_Guide` page 6.
	pub isv_svn: u16, /* (258) Security Version of the Enclave */
	/// Config Security version of the enclave.
	pub config_svn: u16, /* (260) CONFIGSVN */
	/// Unused reserved bytes.
	pub reserved4: [u8; SGX_REPORT_BODY_RESERVED4_BYTES], /* (262) */
	/// Family ID assigned by the ISV.
	///
	/// Todo: #142 - Investigate the relevancy of this value.
	pub isv_family_id: [u8; 16], /* (304) ISV assigned Family ID */
	/// Custom data to be defined by the enclave author.
	///
	/// We use this to provide the public key of the enclave that is to be registered on the chain.
	/// Doing this, will prove that the public key is from a legitimate SGX enclave when it is
	/// verified together with the remote attestation.
	pub report_data: SgxReportData, /* (320) Data provided by the user */
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
	pub version: u16,               /* 0 */
	pub sign_type: u16,             /* 2 */
	pub epid_group_id: u32,         /* 4 */
	pub qe_svn: u16,                /* 8 */
	pub pce_svn: u16,               /* 10 */
	pub xeid: u32,                  /* 12 */
	pub basename: [u8; 32],         /* 16 */
	pub report_body: SgxReportBody, /* 48 */
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
