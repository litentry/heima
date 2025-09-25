use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// SendGrid webhook event types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SendGridEventType {
	Processed,
	Dropped,
	Delivered,
	Deferred,
	Bounce,
	Open,
	Click,
	#[serde(rename = "spamreport")]
	SpamReport,
	Unsubscribe,
	#[serde(rename = "group_unsubscribe")]
	GroupUnsubscribe,
	#[serde(rename = "group_resubscribe")]
	GroupResubscribe,
	#[serde(other)]
	Unknown,
}

/// SendGrid webhook event payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendGridEvent {
	pub email: String,
	pub timestamp: u64,
	pub event: SendGridEventType,
	#[serde(rename = "sg_event_id")]
	pub sg_event_id: Option<String>,
	#[serde(rename = "sg_message_id")]
	pub sg_message_id: Option<String>,
	pub ip: Option<String>,
	pub useragent: Option<String>,
	pub reason: Option<String>,
	pub status: Option<String>,
	pub response: Option<String>,
	pub url: Option<String>,
	pub attempt: Option<u32>,
	#[serde(flatten)]
	pub additional_fields: HashMap<String, serde_json::Value>,
}

/// Processes a single SendGrid webhook event
pub fn process_sendgrid_event(event: &SendGridEvent) {
	match event.event {
		SendGridEventType::Delivered => {
			info!("[SENDGRID_WEBHOOK] Email delivered to {}", event.email);
		},
		SendGridEventType::Dropped => {
			warn!(
				"[SENDGRID_WEBHOOK] Email dropped for {}: {}",
				event.email,
				event.reason.as_deref().unwrap_or("unknown reason")
			);
		},
		SendGridEventType::Deferred => {
			warn!(
				"[SENDGRID_WEBHOOK] Email deferred for {}: {}",
				event.email,
				event.response.as_deref().unwrap_or("unknown response")
			);
		},
		SendGridEventType::Bounce => {
			error!(
				"[SENDGRID_WEBHOOK] Email bounced for {}: {} (status: {})",
				event.email,
				event.reason.as_deref().unwrap_or("unknown reason"),
				event.status.as_deref().unwrap_or("unknown")
			);
		},
		SendGridEventType::SpamReport => {
			warn!("[SENDGRID_WEBHOOK] Spam report from {}", event.email);
		},
		// Don't log other events (processed, open, click, etc.) as they're not critical
		_ => {},
	}
}

/// Processes a batch of SendGrid webhook events
pub fn process_sendgrid_webhook_batch(events: Vec<SendGridEvent>) {
	if events.is_empty() {
		return;
	}

	info!("[SENDGRID_WEBHOOK] Processing {} webhook events", events.len());

	for event in &events {
		process_sendgrid_event(event);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_deserialize_delivered_event() {
		let json_data = json!({
			"email": "test@example.com",
			"timestamp": 1609459200,
			"event": "delivered",
			"sg_event_id": "evt_123",
			"sg_message_id": "msg_456",
			"response": "250 OK",
			"ip": "192.168.1.1"
		});

		let event: SendGridEvent = serde_json::from_value(json_data).unwrap();
		assert_eq!(event.email, "test@example.com");
		assert!(matches!(event.event, SendGridEventType::Delivered));
		assert_eq!(event.sg_event_id, Some("evt_123".to_string()));
		assert_eq!(event.response, Some("250 OK".to_string()));
	}

	#[test]
	fn test_deserialize_bounce_event() {
		let json_data = json!({
			"email": "bounce@example.com",
			"timestamp": 1609459300,
			"event": "bounce",
			"reason": "Invalid email address",
			"status": "5.1.1"
		});

		let event: SendGridEvent = serde_json::from_value(json_data).unwrap();
		assert_eq!(event.email, "bounce@example.com");
		assert!(matches!(event.event, SendGridEventType::Bounce));
		assert_eq!(event.reason, Some("Invalid email address".to_string()));
		assert_eq!(event.status, Some("5.1.1".to_string()));
	}

	#[test]
	fn test_process_webhook_batch() {
		let events = vec![
			SendGridEvent {
				email: "user1@example.com".to_string(),
				timestamp: 1609459200,
				event: SendGridEventType::Delivered,
				sg_event_id: Some("evt_1".to_string()),
				sg_message_id: Some("msg_1".to_string()),
				ip: None,
				useragent: None,
				reason: None,
				status: None,
				response: Some("250 OK".to_string()),
				url: None,
				attempt: None,
				additional_fields: HashMap::new(),
			},
			SendGridEvent {
				email: "user2@example.com".to_string(),
				timestamp: 1609459300,
				event: SendGridEventType::Open,
				sg_event_id: Some("evt_2".to_string()),
				sg_message_id: Some("msg_2".to_string()),
				ip: Some("192.168.1.1".to_string()),
				useragent: Some("Mozilla/5.0".to_string()),
				reason: None,
				status: None,
				response: None,
				url: None,
				attempt: None,
				additional_fields: HashMap::new(),
			},
		];

		// This test mainly checks that the function doesn't panic
		process_sendgrid_webhook_batch(events);
	}
}
