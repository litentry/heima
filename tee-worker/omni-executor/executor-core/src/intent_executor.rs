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

use executor_primitives::AccountId;
use executor_primitives::Intent;
use executor_primitives::IntentId;
use metrics::describe_gauge;

pub type IntentExecutionResult = (Option<Vec<u8>>, bool);

/// Used to perform intent on destination chain
#[async_trait]
pub trait IntentExecutor: Send {
	async fn execute(
		&self,
		account_id: &AccountId,
		intent_id: IntentId,
		intent: Intent,
	) -> Result<IntentExecutionResult, ()>;

	async fn name(&self) -> &'static str;

	async fn on_execution_error(&self) {
		let name = self.name().await;
		describe_gauge!(executor_gauge_name(name), executor_gauge_desc(name));
	}
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
	async fn execute(
		&self,
		_account_id: &AccountId,
		_intent_id: IntentId,
		_intent: Intent,
	) -> Result<IntentExecutionResult, ()> {
		self.sender.send(()).map(|_| (None, false)).map_err(|_| ())
	}

	async fn name(&self) -> &'static str {
		"mocked"
	}
}

fn executor_gauge_name(name: &str) -> String {
	format!("{}_intent_execution_failures", name)
}

fn executor_gauge_desc(name: &str) -> String {
	format!("Number of {} intent executor failures", name)
}
