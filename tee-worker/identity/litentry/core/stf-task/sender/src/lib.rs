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

#![feature(trait_alias)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use thiserror_sgx as thiserror;
}

use lazy_static::lazy_static;
use log::*;

mod error;
pub use error::*;

use litentry_primitives::RequestType;

#[cfg(feature = "std")]
use std::sync::Mutex;
#[cfg(feature = "sgx")]
use std::sync::SgxMutex as Mutex;
use std::{
	boxed::Box,
	error::Error as StdError,
	sync::{
		mpsc::{channel, Receiver, Sender},
		Arc,
	},
};

pub type StfSender = Sender<RequestType>;

// Global storage of the sender. Should not be accessed directly.
lazy_static! {
	static ref GLOBAL_STF_REQUEST_TASK: Arc<Mutex<Option<StfTaskSender>>> =
		Arc::new(Mutex::new(Default::default()));
}

/// Trait to send an stf request to the stf request thread.
pub trait SendStfRequest {
	fn send_stf_request(&self, request: RequestType) -> Result<()>;
}

/// Struct to access the `send_stf_request` function.
pub struct StfRequestSender {}
impl StfRequestSender {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for StfRequestSender {
	fn default() -> Self {
		Self::new()
	}
}

impl SendStfRequest for StfRequestSender {
	fn send_stf_request(&self, request: RequestType) -> Result<()> {
		debug!("send stf request: {:?}", request);

		// Acquire lock on extrinsic sender
		let mutex_guard = GLOBAL_STF_REQUEST_TASK.lock().map_err(|_| Error::MutexAccess)?;

		let stf_task_sender = mutex_guard.clone().ok_or(Error::ComponentNotInitialized)?;

		// Release mutex lock, so we don't block the lock longer than necessary.
		drop(mutex_guard);

		// Send the request to the receiver loop.
		stf_task_sender.send(request)
	}
}

/// Initialization of the extrinsic sender. Needs to be called before any sender access.
pub fn init_stf_task_sender_storage() -> Result<Receiver<RequestType>> {
	let (sender, receiver) = channel();
	let mut stf_task_storage = GLOBAL_STF_REQUEST_TASK.lock().map_err(|_| Error::MutexAccess)?;
	*stf_task_storage = Some(StfTaskSender::new(sender));
	Ok(receiver)
}

/// Wrapping struct around the actual sender. Should not be accessed directly.
#[derive(Clone, Debug)]
struct StfTaskSender {
	sender: StfSender,
}

impl StfTaskSender {
	pub fn new(sender: StfSender) -> Self {
		Self { sender }
	}

	fn send(&self, request: RequestType) -> Result<()> {
		self.sender.send(request).map_err(|e| Error::Other(e.into()))?;
		Ok(())
	}
}
