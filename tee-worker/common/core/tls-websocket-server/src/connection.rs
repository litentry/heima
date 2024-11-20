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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::{
	error::WebSocketError,
	stream_state::{MaybeServerTlsStream, StreamState},
	WebSocketConnection, WebSocketMessageHandler, WebSocketResult,
};
use log::*;
use mio::{event::Event, net::TcpStream, Poll, Ready, Token};
use rustls::{ServerSession, Session};
use std::{
	format, io,
	string::{String, ToString},
	sync::Arc,
	time::Instant,
};
use tungstenite::Message;

/// A web-socket connection object.
pub struct TungsteniteWsConnection<Handler> {
	stream_state: StreamState<TcpStream>,
	connection_token: Token,
	connection_handler: Arc<Handler>,
	is_closed: bool,
}

impl<Handler> TungsteniteWsConnection<Handler>
where
	Handler: WebSocketMessageHandler,
{
	pub fn new(
		tcp_stream: TcpStream,
		maybe_server_session: Option<ServerSession>,
		connection_token: Token,
		handler: Arc<Handler>,
	) -> WebSocketResult<Self> {
		let stream_state = match maybe_server_session {
			Some(sess) => StreamState::new_rustls_stream(sess, tcp_stream),
			None => StreamState::new_plain_stream(tcp_stream),
		};
		Ok(TungsteniteWsConnection {
			stream_state,
			connection_token,
			connection_handler: handler,
			is_closed: false,
		})
	}

	fn maybe_do_tls_read(&mut self) -> ConnectionState {
		match self.stream_state.internal_stream_mut() {
			None => ConnectionState::Closing,
			Some(MaybeServerTlsStream::Plain(_)) => ConnectionState::Alive, // noop for non-TLS ws server
			Some(MaybeServerTlsStream::Rustls(s)) => {
				let tls_session = &mut s.sess;
				match tls_session.read_tls(&mut s.sock) {
					Ok(_) => {},
					Err(err) => {
						if let std::io::ErrorKind::WouldBlock = err.kind() {
							debug!(
								"TLS session is blocked (connection {})",
								self.connection_token.0
							);
							return ConnectionState::Blocked
						}
						warn!(
							"I/O error after reading TLS data (connection {}): {:?}",
							self.connection_token.0, err
						);
						return ConnectionState::Closing
					},
				}

				match tls_session.process_new_packets() {
					Ok(_) => {
						if tls_session.is_handshaking() {
							trace!("TLS session is in handshake");
							return ConnectionState::TlsHandshake
						}
						ConnectionState::Alive
					},
					Err(e) => {
						error!("cannot process TLS packet(s), closing connection: {:?}", e);
						ConnectionState::Closing
					},
				}
			},
		}
	}

	fn maybe_do_tls_write(&mut self) -> ConnectionState {
		match self.stream_state.internal_stream_mut() {
			None => ConnectionState::Closing,
			Some(MaybeServerTlsStream::Plain(_)) => ConnectionState::Alive, // noop for non-TLS ws server
			Some(MaybeServerTlsStream::Rustls(s)) => match s.sess.write_tls(&mut s.sock) {
				Ok(_) => {
					trace!("TLS write successful, connection {} is alive", self.connection_token.0);
					if s.sess.is_handshaking() {
						return ConnectionState::TlsHandshake
					}
					ConnectionState::Alive
				},
				Err(e) => {
					error!("TLS write error (connection {}): {:?}", self.connection_token.0, e);
					ConnectionState::Closing
				},
			},
		}
	}

	/// Read from a web-socket, or initiate handshake if websocket is not initialized yet.
	///
	/// Returns a boolean 'connection should be closed'.
	fn read_or_initialize_websocket(&mut self) -> WebSocketResult<bool> {
		if let StreamState::Established(web_socket) = &mut self.stream_state {
			trace!(
				"Read is possible for connection {}: {}",
				self.connection_token.0,
				web_socket.can_read()
			);
			match web_socket.read_message() {
				Ok(m) =>
					if let Err(e) = self.handle_message(m) {
						error!(
							"Failed to handle web-socket message (connection {}): {:?}",
							self.connection_token.0, e
						);
					},
				Err(e) =>
					return match e {
						tungstenite::Error::Io(e) if e.kind() == io::ErrorKind::WouldBlock =>
							Ok(false),
						_ => {
							trace!(
								"Error while reading web-socket message (connection {}): {:?}",
								self.connection_token.0,
								e
							);
							Ok(true)
						},
					},
			}
			trace!("Read successful for connection {}", self.connection_token.0);
		} else {
			trace!("Initialize connection {}", self.connection_token.0);
			self.stream_state = std::mem::take(&mut self.stream_state).attempt_handshake();
			if self.stream_state.is_invalid() {
				warn!("Web-socket connection ({:?}) failed, closing", self.connection_token);
				return Ok(true)
			}
			debug!("Initialized connection {} successfully", self.connection_token.0);
		}

		Ok(false)
	}

	fn handle_message(&mut self, message: Message) -> WebSocketResult<()> {
		match message {
			Message::Text(string_message) => {
				trace!(
					"Got Message::Text on web-socket (connection {}), calling handler..",
					self.connection_token.0
				);
				let message_handled_timer = Instant::now();
				if let Some(reply) = self
					.connection_handler
					.handle_message(self.connection_token.into(), string_message)?
				{
					trace!(
						"Handling message yielded a reply, sending it now to connection {}..",
						self.connection_token.0
					);
					self.write_message(reply)?;
					trace!("Reply sent successfully to connection {}", self.connection_token.0);
				}
				debug!(
					"Handled web-socket message in {} ms",
					message_handled_timer.elapsed().as_millis()
				);
			},
			Message::Binary(_) => {
				warn!("received binary message, don't have a handler for this format");
			},
			Message::Close(_) => {
				debug!(
					"Received close frame, driving web-socket connection {} to close",
					self.connection_token.0
				);
				if let StreamState::Established(web_socket) = &mut self.stream_state {
					// Send a close frame back and then flush the send queue.
					if let Err(e) = web_socket.close(None) {
						match e {
							tungstenite::Error::ConnectionClosed
							| tungstenite::Error::AlreadyClosed => {},
							_ => warn!(
								"Failed to send close frame (connection {}): {:?}",
								self.connection_token.0, e
							),
						}
					}
					match web_socket.write_pending() {
						Ok(_) => {},
						Err(e) => match e {
							tungstenite::Error::ConnectionClosed
							| tungstenite::Error::AlreadyClosed => {},
							_ => warn!("Failed to write pending frames after closing (connection {}): {:?}", self.connection_token.0, e),
						},
					}
				}
				debug!("Successfully closed connection {}", self.connection_token.0);
			},
			_ => {},
		}
		Ok(())
	}

	pub(crate) fn write_message(&mut self, message: String) -> WebSocketResult<()> {
		match &mut self.stream_state {
			StreamState::Established(web_socket) => {
				if !web_socket.can_write() {
					return Err(WebSocketError::ConnectionClosed)
				}
				debug!("Write message to connection {}: {}", self.connection_token.0, message);
				web_socket
					.write_message(Message::Text(message))
					.map_err(|e| WebSocketError::SocketWriteError(format!("{:?}", e)))
			},
			_ =>
				Err(WebSocketError::SocketWriteError("No active web-socket available".to_string())),
		}
	}
}

impl<Handler> WebSocketConnection for TungsteniteWsConnection<Handler>
where
	Handler: WebSocketMessageHandler,
{
	type Socket = TcpStream;

	fn socket(&self) -> Option<&Self::Socket> {
		self.stream_state.internal_stream().map(|s| s.inner())
	}

	fn get_session_readiness(&self) -> Ready {
		match self.stream_state.internal_stream() {
			None => mio::Ready::empty(),
			Some(s) => {
				let wants_read = s.wants_read();
				let wants_write = s.wants_write();

				if wants_read && wants_write {
					mio::Ready::readable() | mio::Ready::writable()
				} else if wants_write {
					mio::Ready::writable()
				} else {
					mio::Ready::readable()
				}
			},
		}
	}

	fn on_ready(&mut self, poll: &mut Poll, event: &Event) -> WebSocketResult<()> {
		let mut is_closing = false;

		if event.readiness().is_readable() {
			trace!("Connection ({:?}) is readable", self.token());

			let connection_state = self.maybe_do_tls_read();

			if connection_state.is_alive() {
				is_closing = self.read_or_initialize_websocket()?;
			} else {
				is_closing = connection_state.is_closing();
			}
		}

		if event.readiness().is_writable() {
			trace!("Connection ({:?}) is writable", self.token());

			let connection_state = self.maybe_do_tls_write();

			if connection_state.is_alive() {
				if let StreamState::Established(web_socket) = &mut self.stream_state {
					trace!("Web-socket, write pending messages");
					if let Err(e) = web_socket.write_pending() {
						match e {
							tungstenite::Error::ConnectionClosed
							| tungstenite::Error::AlreadyClosed => is_closing = true,
							_ => error!("Failed to write pending web-socket messages: {:?}", e),
						}
					}
				}
			} else {
				is_closing = connection_state.is_closing();
			}
		}

		if is_closing {
			debug!("Connection ({:?}) is closed", self.token());
			self.is_closed = true;
		} else {
			// Re-register with the poll.
			self.reregister(poll)?;
		}
		Ok(())
	}

	fn is_closed(&self) -> bool {
		self.is_closed
	}

	fn token(&self) -> Token {
		self.connection_token
	}
}

/// Internal connection state.
#[derive(Debug, Clone)]
enum ConnectionState {
	Closing,
	Blocked,
	Alive,
	TlsHandshake,
}

impl ConnectionState {
	pub(crate) fn is_alive(&self) -> bool {
		matches!(self, ConnectionState::Alive)
	}

	pub(crate) fn is_closing(&self) -> bool {
		matches!(self, ConnectionState::Closing)
	}
}
