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

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::primitives::Intent;

/// Used to perform intent on destination chain
#[async_trait]
pub trait IntentExecutor: Send {
	async fn execute(&self, intent: Intent) -> Result<(), ()>;
}

pub struct MockedIntentExecutor {
	sender: mpsc::UnboundedSender<()>,
}

impl MockedIntentExecutor {
	pub fn new() -> (Self, mpsc::UnboundedReceiver<()>) {
		let (sender, receiver) = mpsc::unbounded_channel();
		(Self { sender }, receiver)
	}
}

#[async_trait]
impl IntentExecutor for MockedIntentExecutor {
	async fn execute(&self, _intent: Intent) -> Result<(), ()> {
		self.sender.send(()).map_err(|_| ())
	}
}
