mod native_call;

use parentchain_primitives::OmniAccountAuthType;
use parity_scale_codec::Encode;
use tokio::sync::{mpsc, oneshot};

pub use native_call::NativeCall;
pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeTaskSender = mpsc::Sender<NativeTask>;

pub struct NativeTask {
	pub call: NativeCall,
	pub auth_type: OmniAccountAuthType,
	pub response_sender: ResponseSender,
}

pub async fn run_native_task_handler(buffer: usize) -> NativeTaskSender {
	let (sender, mut receiver) = mpsc::channel::<NativeTask>(buffer);

	tokio::spawn(async move {
		while let Some(task) = receiver.recv().await {
			execute_native_call(task.call, task.response_sender).await;
		}
	});

	sender
}

async fn execute_native_call(call: NativeCall, response_sender: ResponseSender) {
	match call {
		NativeCall::noop(_) => {
			let _ = response_sender.send("noop".encode());
		},
	}
}
