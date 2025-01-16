mod native_call;

use parity_scale_codec::Encode;
use tokio::sync::{mpsc, oneshot};

pub use native_call::NativeCall;
pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeCallSender = mpsc::Sender<(NativeCall, ResponseSender)>;

pub async fn run_native_call_executor(buffer: usize) -> NativeCallSender {
	let (sender, mut receiver) = mpsc::channel::<(NativeCall, ResponseSender)>(buffer);

	tokio::spawn(async move {
		while let Some((call, response_sender)) = receiver.recv().await {
			execute_native_call(call, response_sender).await;
		}
	});

	sender
}

async fn execute_native_call(call: NativeCall, response_sender: ResponseSender) {
	match call {
		NativeCall::noop => {
			let _ = response_sender.send("noop".encode());
		},
	}
}
