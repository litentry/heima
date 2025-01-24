use executor_core::native_call::NativeCall;
use parentchain_rpc_client::{CustomConfig, SubxtClient};
use parity_scale_codec::Encode;
use primitives::OmniAccountAuthType;
use std::sync::Arc;
use storage::{MemberOmniAccountStorage, Storage, StorageDB};
use tokio::sync::{mpsc, oneshot};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeTaskSender = mpsc::Sender<NativeTask>;

pub struct NativeTask {
	pub call: NativeCall,
	pub auth_type: OmniAccountAuthType,
	pub response_sender: ResponseSender,
}
pub struct TaskHandlerContext {
	pub parentchain_rpc_client: Arc<SubxtClient<CustomConfig>>,
	pub storage_db: Arc<StorageDB>,
	pub jwt_secret: String,
}

pub async fn run_native_task_handler(
	buffer: usize,
	ctx: Arc<TaskHandlerContext>,
) -> NativeTaskSender {
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
