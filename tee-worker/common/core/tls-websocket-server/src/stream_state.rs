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

use log::*;
use rustls::{ServerSession, Session, StreamOwned};
use std::{
	boxed::Box,
	io::{Read, Result as IoResult, Write},
};
use tungstenite::{
	accept,
	handshake::{server::NoCallback, MidHandshake},
	HandshakeError, ServerHandshake, WebSocket,
};

// similar to `tungstenite::stream::MaybeTlsStream`, but with a server side implementation
#[allow(clippy::large_enum_variant)]
pub(crate) enum MaybeServerTlsStream<S: Read + Write> {
	Plain(S),
	Rustls(StreamOwned<ServerSession, S>),
}

impl<S: Read + Write> MaybeServerTlsStream<S> {
	pub fn inner(&self) -> &S {
		match self {
			MaybeServerTlsStream::Plain(s) => s,
			MaybeServerTlsStream::Rustls(s) => &s.sock,
		}
	}

	pub fn wants_read(&self) -> bool {
		match self {
			MaybeServerTlsStream::Plain(_) => true,
			MaybeServerTlsStream::Rustls(s) => s.sess.wants_read(),
		}
	}

	pub fn wants_write(&self) -> bool {
		match self {
			MaybeServerTlsStream::Plain(_) => false, // do not monitor writable events for non-tls server
			MaybeServerTlsStream::Rustls(s) => s.sess.wants_write(),
		}
	}
}

impl<S: Read + Write> Read for MaybeServerTlsStream<S> {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
		match *self {
			MaybeServerTlsStream::Plain(ref mut s) => s.read(buf),
			MaybeServerTlsStream::Rustls(ref mut s) => s.read(buf),
		}
	}
}

impl<S: Read + Write> Write for MaybeServerTlsStream<S> {
	fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
		match *self {
			MaybeServerTlsStream::Plain(ref mut s) => s.write(buf),
			MaybeServerTlsStream::Rustls(ref mut s) => s.write(buf),
		}
	}

	fn flush(&mut self) -> IoResult<()> {
		match *self {
			MaybeServerTlsStream::Plain(ref mut s) => s.flush(),
			MaybeServerTlsStream::Rustls(ref mut s) => s.flush(),
		}
	}
}

/// Internal stream state representing different websocket statuses
pub(crate) enum StreamState<S: Read + Write> {
	Invalid,
	Initialized(Box<MaybeServerTlsStream<S>>),
	InHandshake(MidHandshake<ServerHandshake<MaybeServerTlsStream<S>, NoCallback>>),
	Established(Box<WebSocket<MaybeServerTlsStream<S>>>),
}

impl<S: Read + Write> Default for StreamState<S> {
	fn default() -> Self {
		Self::Invalid
	}
}

impl<S: Read + Write> StreamState<S> {
	pub(crate) fn new_plain_stream(stream: S) -> Self {
		StreamState::Initialized(Box::new(MaybeServerTlsStream::Plain(stream)))
	}

	pub(crate) fn new_rustls_stream(session: ServerSession, stream: S) -> Self {
		let s = StreamOwned::new(session, stream);
		StreamState::Initialized(Box::new(MaybeServerTlsStream::Rustls(s)))
	}

	pub(crate) fn is_invalid(&self) -> bool {
		matches!(self, StreamState::Invalid)
	}

	pub(crate) fn internal_stream(&self) -> Option<&MaybeServerTlsStream<S>> {
		match self {
			StreamState::Initialized(s) => Some(s),
			StreamState::InHandshake(h) => Some(h.get_ref().get_ref()),
			StreamState::Established(ws) => Some(ws.get_ref()),
			StreamState::Invalid => None,
		}
	}

	pub(crate) fn internal_stream_mut(&mut self) -> Option<&mut MaybeServerTlsStream<S>> {
		match self {
			StreamState::Initialized(s) => Some(s),
			StreamState::InHandshake(h) => Some(h.get_mut().get_mut()),
			StreamState::Established(ws) => Some(ws.get_mut()),
			StreamState::Invalid => None,
		}
	}

	pub(crate) fn attempt_handshake(self) -> Self {
		match self {
			// We have the bare TLS stream only, attempt to do a web-socket handshake.
			StreamState::Initialized(s) => Self::from_handshake_result(accept(*s)),
			// We already have an on-going handshake, attempt another try.
			StreamState::InHandshake(hs) => Self::from_handshake_result(hs.handshake()),
			_ => self,
		}
	}

	#[allow(clippy::type_complexity)]
	fn from_handshake_result(
		handshake_result: Result<
			WebSocket<MaybeServerTlsStream<S>>,
			HandshakeError<ServerHandshake<MaybeServerTlsStream<S>, NoCallback>>,
		>,
	) -> Self {
		match handshake_result {
			Ok(ws) => Self::Established(Box::new(ws)),
			Err(e) => match e {
				// I/O would block our handshake attempt. Need to re-try.
				HandshakeError::Interrupted(mhs) => {
					info!("Web-socket handshake interrupted");
					Self::InHandshake(mhs)
				},
				HandshakeError::Failure(e) => {
					error!("Web-socket handshake failed: {:?}", e);
					Self::Invalid
				},
			},
		}
	}
}
