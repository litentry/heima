#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use lazy_static::lazy_static;
use log::*;

#[cfg(feature = "std")]
use std::sync::Mutex;
#[cfg(feature = "sgx")]
use std::sync::SgxMutex as Mutex;

use itp_types::OpaqueCall;
use std::{
	format,
	string::String,
	sync::{
		mpsc::{channel, Receiver, Sender},
		Arc,
	},
};

lazy_static! {
	static ref GLOBAL_PARACHAIN_EXTRINSIC_TASK: Arc<Mutex<Option<ParachainExtrinsicTaskSender>>> =
		Arc::new(Mutex::new(Default::default()));
}

#[derive(Debug, Clone)]
struct ParachainExtrinsicTaskSender {
	sender: Sender<OpaqueCall>,
}

impl ParachainExtrinsicTaskSender {
	fn new(sender: Sender<OpaqueCall>) -> Self {
		Self { sender }
	}

	fn send(&self, call: OpaqueCall) -> Result<(), String> {
		self.sender
			.send(call)
			.map_err(|e| format!("Failed to send extrinsic to handler: {:?}", e))
	}
}

pub trait SendParachainExtrinsic {
	fn send(&self, call: OpaqueCall) -> Result<(), String>;
}

pub struct ParachainExtrinsicSender {}

impl ParachainExtrinsicSender {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for ParachainExtrinsicSender {
	fn default() -> Self {
		Self::new()
	}
}

impl SendParachainExtrinsic for ParachainExtrinsicSender {
	fn send(&self, call: OpaqueCall) -> Result<(), String> {
		debug!("Sending parachain extrinsic {:?}", call);

		let mutex_guard =
			GLOBAL_PARACHAIN_EXTRINSIC_TASK.lock().map_err(|_| "Mutex lock failed")?;

		let sender = mutex_guard.clone().ok_or("Parachain extrinsic sender was not initialized")?;

		drop(mutex_guard);

		sender.send(call)
	}
}

pub fn init_parachain_extrinsic_sender_storage() -> Result<Receiver<OpaqueCall>, String> {
	let (sender, receiver) = channel();
	let mut storage = GLOBAL_PARACHAIN_EXTRINSIC_TASK.lock().expect("Mutex lock failed");
	*storage = Some(ParachainExtrinsicTaskSender::new(sender));

	Ok(receiver)
}
