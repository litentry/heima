#![feature(trait_alias)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use lazy_static::lazy_static;
use litentry_primitives::AesRequest;
use log::*;
#[cfg(feature = "std")]
use std::sync::Mutex;
#[cfg(feature = "sgx")]
use std::sync::SgxMutex as Mutex;
use std::{
	format,
	string::{String, ToString},
	sync::{
		mpsc::{channel, Receiver, Sender as MpscSender},
		Arc,
	},
};

#[derive(Debug)]
pub struct VCRequest {
	pub request: AesRequest,
}

pub type VcSender = MpscSender<VCRequest>;

// Global storage of the sender. Should not be accessed directly.
lazy_static! {
	static ref GLOBAL_VC_TASK_SENDER: Arc<Mutex<Option<VcTaskSender>>> =
		Arc::new(Mutex::new(Default::default()));
}

pub struct VcRequestSender {}
impl VcRequestSender {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for VcRequestSender {
	fn default() -> Self {
		Self::new()
	}
}

impl VcRequestSender {
	pub fn send(&self, request: VCRequest) -> Result<(), String> {
		debug!("send vc request: {:?}", request);

		// Acquire lock on vc task sender
		let mutex_guard = GLOBAL_VC_TASK_SENDER.lock().map_err(|_| "Could not access Mutex")?;
		let vc_task_sender = mutex_guard.clone().ok_or("Daemon sender was not initialized")?;
		// Release mutex lock, so we don't block the lock longer than necessary.
		drop(mutex_guard);

		// Send the request to the receiver loop.
		vc_task_sender.send(request)?;

		Ok(())
	}
}

/// Initialization of the vc task sender. Needs to be called before any sender access.
pub fn init_vc_task_sender() -> Receiver<VCRequest> {
	let (sender, receiver) = channel();
	// It makes no sense to handle the unwrap, as this statement fails only if the lock has been poisoned
	// I believe at that point it is an unrecoverable error
	let mut vc_task_sender = GLOBAL_VC_TASK_SENDER.lock().unwrap();
	*vc_task_sender = Some(VcTaskSender::new(sender, false));
	receiver
}

pub fn pause_vc_task_sender() -> Result<(), String> {
	info!("Pause vc task sender");
	let mut mutex_guard = GLOBAL_VC_TASK_SENDER.lock().map_err(|_| "Could not access Mutex")?;
	let sender = mutex_guard.as_mut().ok_or("Daemon sender was not initialized")?;
	sender.set_paused(true);
	Ok(())
}

/// Wrapping struct around the actual sender. Should not be accessed directly. (unnecessary)
#[derive(Clone, Debug)]
struct VcTaskSender {
	sender: VcSender,
	paused: bool,
}

impl VcTaskSender {
	pub fn new(sender: VcSender, paused: bool) -> Self {
		Self { sender, paused }
	}

	pub fn set_paused(&mut self, paused: bool) {
		self.paused = paused;
	}

	fn send(&self, request: VCRequest) -> Result<(), String> {
		if self.paused {
			return Err("Failed to send vc task: sender is paused".to_string());
		}

		self.sender
			.send(request)
			.map_err(|e| format!("Failed to send vc task: {:?}", e))
	}
}
