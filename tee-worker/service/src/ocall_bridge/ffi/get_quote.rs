/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
	Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

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

use crate::ocall_bridge::bridge_api::{Bridge, RemoteAttestationBridge};
use log::*;
use sgx_types::{error::*, types::*};
use std::{slice, sync::Arc};

/// p_quote must be a pre-allocated memory region of size `maxlen`
#[no_mangle]
pub unsafe extern "C" fn ocall_get_quote(
	p_sigrl: *const u8,
	sigrl_len: u32,
	p_report: *const Report,
	quote_type: QuoteSignType,
	p_spid: *const Spid,
	p_nonce: *const QuoteNonce,
	p_qe_report: *mut Report,
	p_quote: *mut u8,
	maxlen: u32,
	p_quote_len: *mut u32,
) -> SgxStatus {
	get_quote(
		p_sigrl,
		sigrl_len,
		p_report,
		quote_type,
		p_spid,
		p_nonce,
		p_qe_report,
		p_quote,
		maxlen,
		p_quote_len,
		Bridge::get_ra_api(), // inject the RA API (global state)
	)
}

#[allow(clippy::too_many_arguments)]
fn get_quote(
	p_sigrl: *const u8,
	sigrl_len: u32,
	p_report: *const Report,
	quote_type: QuoteSignType,
	p_spid: *const Spid,
	p_nonce: *const QuoteNonce,
	p_qe_report: *mut Report,
	p_quote: *mut u8,
	maxlen: u32,
	p_quote_len: *mut u32,
	ra_api: Arc<dyn RemoteAttestationBridge>,
) -> SgxStatus {
	debug!("    Entering ocall_get_quote");

	let revocation_list: Vec<u8> =
		unsafe { slice::from_raw_parts(p_sigrl, sigrl_len as usize).to_vec() };

	let report = unsafe { *p_report };
	let spid = unsafe { *p_spid };
	let quote_nonce = unsafe { *p_nonce };

	let get_quote_result =
		match ra_api.get_quote(revocation_list, report, quote_type, spid, quote_nonce) {
			Ok(r) => r,
			Err(e) => {
				error!("[-]  Failed to get quote: {:?}", e);
				return e.into()
			},
		};

	let quote = get_quote_result.1;

	if quote.len() as u32 > maxlen {
		return SgxStatus::Unexpected
	}

	let quote_slice = unsafe { slice::from_raw_parts_mut(p_quote, quote.len()) };
	quote_slice.clone_from_slice(quote.as_slice());

	unsafe {
		*p_qe_report = get_quote_result.0;
		*p_quote_len = quote.len() as u32;
	};

	SgxStatus::Success
}

#[no_mangle]
pub unsafe extern "C" fn ocall_get_dcap_quote(
	p_report: *const Report,
	p_quote: *mut u8,
	quote_size: u32,
) -> SgxStatus {
	get_dcap_quote(
		p_report,
		p_quote,
		quote_size,
		Bridge::get_ra_api(), // inject the RA API (global state)
	)
}

fn get_dcap_quote(
	p_report: *const Report,
	p_quote: *mut u8,
	quote_size: u32,
	ra_api: Arc<dyn RemoteAttestationBridge>,
) -> SgxStatus {
	let report = unsafe { *p_report };

	let quote = match ra_api.get_dcap_quote(report, quote_size) {
		Ok(r) => r,
		Err(e) => {
			error!("Failed to get dcap quote: {:?}", e);
			return e.into()
		},
	};

	if quote.len() as u32 > quote_size {
		return SgxStatus::Unexpected
	}

	let quote_slice = unsafe { slice::from_raw_parts_mut(p_quote, quote.len()) };
	quote_slice.clone_from_slice(quote.as_slice());

	SgxStatus::Success
}
