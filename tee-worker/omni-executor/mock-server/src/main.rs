use std::env;
use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};
use std::thread;
use std::time::Duration;

fn main() {
	// Check for --version flag
	if env::args().any(|arg| arg == "--version") {
		println!("mock-server version {}", env!("CARGO_PKG_VERSION"));
		return;
	}

	env_logger::init();

	let port = env::args().nth(1).and_then(|p: String| p.parse::<u16>().ok()).unwrap_or(3456);

	println!("Starting mock server on port {}", port);

	// Flag to track if server has started
	let server_started = Arc::new(AtomicBool::new(false));
	let server_started_clone = server_started.clone();

	// Start the server in a separate thread
	let server_handle = thread::spawn(move || {
		match mock_server::run(port) {
			Ok(url) => {
				println!("Mock server started successfully at: {}", url);
				server_started_clone.store(true, Ordering::Relaxed);
				// The server will run until it receives a shutdown signal
				// The run function will block until shutdown is complete
			},
			Err(e) => {
				eprintln!("Failed to start mock server: {:?}", e);
				std::process::exit(1);
			},
		}
	});

	// Wait for server to start
	while !server_started.load(Ordering::Relaxed) {
		thread::sleep(Duration::from_millis(100));
	}

	// Wait for the server thread to complete (when shutdown signal is received)
	match server_handle.join() {
		Ok(()) => {
			println!("Server shutdown complete.");
		},
		Err(_) => {
			eprintln!("Server thread panicked");
			std::process::exit(1);
		},
	}
}
