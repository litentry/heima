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
	utils::length_from_raw_data,
};
use crate::{
	Cpusvn, DcapQuote, DcapQuoteHeader, Fmspc, Pcesvn, QuotingEnclave, SgxReport, SgxReportBody,
	SgxStatus, TcbVersionStatus, ATTESTATION_KEY_SIZE, REPORT_SIGNATURE_SIZE,
};
use alloc::string::String;
use core::time::Duration;
use der::asn1::ObjectIdentifier;
use frame_support::ensure;
use p256::ecdsa::{signature::DigestVerifier, Signature, VerifyingKey};
use parity_scale_codec::Decode;
use sha2::{Digest, Sha256};
use sp_std::{
	convert::{TryFrom, TryInto},
	prelude::*,
	vec,
};
use x509_cert::Certificate;

pub mod collateral;
#[cfg(test)]
mod tests;
mod utils;

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

/// Encode two 32-byte values in DER format
/// This is meant for 256 bit ECC signatures or public keys
pub fn encode_as_der(data: &[u8]) -> Result<Vec<u8>, &'static str> {
	if data.len() != 64 {
		return Result::Err("Key must be 64 bytes long");
	}
	let mut sequence = der::asn1::SequenceOf::<der::asn1::UintRef, 2>::new();
	sequence
		.add(der::asn1::UintRef::new(&data[0..32]).map_err(|_| "Invalid public key")?)
		.map_err(|_| "Invalid public key")?;
	sequence
		.add(der::asn1::UintRef::new(&data[32..]).map_err(|_| "Invalid public key")?)
		.map_err(|_| "Invalid public key")?;
	// 72 should be enough in all cases. 2 + 2 x (32 + 3)
	let mut asn1 = vec![0u8; 72];
	let mut writer = der::SliceWriter::new(&mut asn1);
	writer.encode(&sequence).map_err(|_| "Could not encode public key to DER")?;
	Ok(writer.finish().map_err(|_| "Could not convert public key to DER")?.to_vec())
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
	verify_signature(certificate, data, &signature, webpki::ring::ECDSA_P256_SHA256)?;
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
	verify_signature(certificate, data, &signature, webpki::ring::ECDSA_P256_SHA256)?;
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
	parts.filter(|p| !p.is_empty()).filter_map(|p| base64::decode(p).ok()).collect()
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
	ensure!(quote.header.attestation_key_type == 2, "Only support for ECDSA-256");
	ensure!(
		quote.quote_signature_data.qe_certification_data.certification_data_type == 5,
		"Only support for PEM formatted PCK Cert Chain"
	);

	let certs = extract_certs(&quote.quote_signature_data.qe_certification_data.certification_data);

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
	ensure!(quote.header.attestation_key_type == 2, "Only support for ECDSA-256");
	ensure!(
		quote.quote_signature_data.qe_certification_data.certification_data_type == 5,
		"Only support for PEM formatted PCK Cert Chain"
	);
	ensure!(quote.quote_signature_data.qe_report.verify(qe), "Enclave rejected by quoting enclave");
	let mut xt_signer_array = [0u8; 32];
	xt_signer_array.copy_from_slice(&quote.body.report_data.d[..32]);

	let certs = extract_certs(&quote.quote_signature_data.qe_certification_data.certification_data);
	ensure!(certs.len() >= 2, "Certificate chain must have at least two certificates");
	let intermediate_certificate_slices: Vec<webpki::types::CertificateDer> =
		certs[1..].iter().map(|c| c.as_slice().into()).collect();
	let leaf_cert_der = webpki::types::CertificateDer::from(certs[0].as_slice());
	let leaf_cert = webpki::EndEntityCert::try_from(&leaf_cert_der)
		.map_err(|_| "Failed to parse leaf certificate")?;
	verify_certificate_chain(&leaf_cert, &intermediate_certificate_slices, verification_time)?;

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
	let hash = sha2::Sha256::digest(hash_data);
	ensure!(
		hash.as_slice() == &quote.quote_signature_data.qe_report.report_data.d[0..32],
		"Hashes must match"
	);

	let qe_report_offset = attestation_key_offset + ATTESTATION_KEY_SIZE;
	let qe_report_slice = &dcap_quote_raw[qe_report_offset..(qe_report_offset + REPORT_SIZE)];
	let pub_key = {
		let mut buf = [0u8; 65];
		buf[0] = 0x04; // SEC1 uncompressed prefix
		buf[1..].copy_from_slice(&quote.quote_signature_data.ecdsa_attestation_key); // 64 bytes
		buf
	};

	let report = &dcap_quote_raw[0..(DCAP_QUOTE_HEADER_SIZE + REPORT_SIZE)];
	let sig = &quote.quote_signature_data.isv_enclave_report_signature;

	// Verify that the enclave data matches the signature generated by the trusted attestation key.
	// This establishes trust into the data of the enclave we actually want to verify
	verify_report_signature(&pub_key, report, sig)?;

	// Verify that the QE report was signed by Intel. This establishes trust into the QE report.
	let asn1_signature = encode_as_der(&quote.quote_signature_data.qe_report_signature)?;
	verify_signature(
		&leaf_cert,
		qe_report_slice,
		&asn1_signature,
		webpki::ring::ECDSA_P256_SHA256,
	)?;

	ensure!(dcap_quote_clone.is_empty(), "There should be no bytes left over after decoding");
	let report = SgxReport {
		mr_enclave: quote.body.mr_enclave,
		status: SgxStatus::Ok,
		pubkey: xt_signer_array,
		timestamp: verification_time,
		build_mode: quote.body.sgx_build_mode(),
	};
	Ok((fmspc, tcb_info, report))
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
		.map(|e| e.extn_value.clone());

	let extension = extension_iter.next();
	ensure!(
		extension.is_some() && extension_iter.next().is_none(),
		"There should only be one section containing Intel extensions"
	);
	// SAFETY: Ensured above that extension.is_some() == true
	Ok(extension.unwrap().into_bytes())
}

fn get_fmspc(der: &[u8]) -> Result<Fmspc, &'static str> {
	let bytes_oid = OID_FMSPC.as_bytes();
	let mut offset = der
		.windows(bytes_oid.len())
		.position(|window| window == bytes_oid)
		.ok_or("Certificate does not contain 'FMSPC_OID'")?;
	offset += 12; // length oid (10) + asn1 tag (1) + asn1 length10 (1)

	let fmspc_size = core::mem::size_of::<Fmspc>() / core::mem::size_of::<u8>();
	let data = der.get(offset..offset + fmspc_size).ok_or("Index out of bounds")?;
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

fn verify_report_signature(
	pubkey_sec1: &[u8],   // 65 bytes: 0x04 || X || Y
	message: &[u8],       // Message to verify (the report)
	signature_raw: &[u8], // 64 bytes: r || s
) -> Result<(), &'static str> {
	// Load verifying key from uncompressed SEC1-encoded public key
	let verifying_key =
		VerifyingKey::from_sec1_bytes(pubkey_sec1).map_err(|_| "Invalid public key format")?;

	// Parse raw signature (r || s)
	let signature = Signature::from_slice(signature_raw).map_err(|_| "Invalid ECDSA signature")?;

	// Hash the message
	let digest = Sha256::new().chain_update(message);

	// Verify the signature
	verifying_key
		.verify_digest(digest, &signature)
		.map_err(|_| "Failed to verify report signature")
}
