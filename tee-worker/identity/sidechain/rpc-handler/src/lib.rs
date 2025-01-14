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

#![feature(trait_alias)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use futures_sgx as futures;
	pub use jsonrpc_core_sgx as jsonrpc_core;
	pub use rust_base58_sgx as base58;
}

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::import_block_api::add_import_block_rpc_method;
use itp_import_queue::{ImportQueue, PushToQueue};
use its_primitives::types::SignedBlock;
use jsonrpc_core::IoHandler;
use std::sync::Arc;

pub mod constants;
pub mod direct_top_pool_api;
mod import_block_api;

pub fn add_sidechain_api(
	io_handler: &mut IoHandler,
	sidechain_import_queue: Arc<ImportQueue<SignedBlock>>,
) {
	add_import_block_rpc_method(move |block| sidechain_import_queue.push_single(block), io_handler);
}
