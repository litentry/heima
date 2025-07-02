// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use std::thread;
use tokio::{
	runtime::Builder,
	select,
	signal::unix::{signal, SignalKind},
	sync::oneshot::channel,
	task::{spawn_local, LocalSet},
};
use warp::Filter;

mod binance;
mod pumpx;
mod sendgrid;
mod solana;

// It should only works on UNIX.
async fn shutdown_signal() {
	let mut hangup_stream =
		signal(SignalKind::hangup()).expect("Cannot install SIGHUP signal handler");
	let mut sigint_stream =
		signal(SignalKind::interrupt()).expect("Cannot install SIGINT signal handler");
	let mut sigterm_stream =
		signal(SignalKind::terminate()).expect("Cannot install SIGTERM signal handler");

	select! {
		_val = hangup_stream.recv() => log::warn!("Received SIGHUP"),
		_val = sigint_stream.recv() => log::warn!("Received SIGINT"),
		_val = sigterm_stream.recv() => log::warn!("Received SIGTERM"),
	}
	log::info!("Shutdown signal received, stopping server...");
}

pub fn run(port: u16) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
	run_with_shutdown_control(port, true)
}

pub fn run_with_shutdown_control(
	port: u16,
	wait_for_shutdown: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
	let (result_in, result_out) = channel();
	let (shutdown_in, shutdown_out) = channel();

	thread::spawn(move || {
		let runtime = Builder::new_current_thread().enable_all().build().unwrap();
		LocalSet::new().block_on(&runtime, async {
			let (addr, srv) = warp::serve(
				binance::handle()
					.or(pumpx::handle())
					.or(sendgrid::handle())
					.or(solana::handle())
					.boxed(),
			)
			.bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
				shutdown_signal().await;
				let _ = shutdown_in.send(());
			});

			log::info!("mock-server listen on addr:{:?}", addr);
			let _ = result_in.send(format!("http://{:?}", addr));

			let join = spawn_local(srv);
			let _ = join.await;
			log::info!("Server has been shut down gracefully");
		});
	});

	let url = result_out.blocking_recv()?;

	if wait_for_shutdown {
		let _ = shutdown_out.blocking_recv();
	}

	Ok(url)
}

/// Helper function to start a mock server for testing purposes.
/// Returns the server URL as a String.
/// The server will be started on a random available port.
pub async fn async_run_test_only() -> String {
	tokio::task::spawn_blocking(move || run_with_shutdown_control(0, false))
		.await
		.expect("Fail to start mock server")
		.expect("Failed to get server URL")
}
