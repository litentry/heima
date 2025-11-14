pub mod template;

use async_trait::async_trait;
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};
use tracing::{error, info};

#[derive(Debug)]
pub enum Error {
	SendEmailFailed,
}

#[derive(Debug)]
pub struct Mail {
	pub to: String,
	pub subject: String,
	pub body: String,
	pub content_type: String,
}

#[async_trait]
pub trait MailerTrait {
	async fn send(&self, mail: Mail) -> Result<(), Error>;
}

pub struct Mailer {
	// for mocking purpose
	api_host: Option<String>,
	api_key: String,
	from_email: String,
	from_name: String,
}

impl Mailer {
	pub fn new(
		api_host: Option<String>,
		api_key: String,
		from_email: String,
		from_name: String,
	) -> Self {
		Self { api_host, api_key, from_email, from_name }
	}
}

#[async_trait]
impl MailerTrait for Mailer {
	async fn send(&self, mail: Mail) -> Result<(), Error> {
		info!("[EMAIL_LIFECYCLE] Sending email to: {} via SendGrid", mail.to);

		let from_email = Email::new(&self.from_email).set_name(&self.from_name);
		let personalization = Personalization::new(Email::new(&mail.to));
		let content = Content::new().set_content_type(&mail.content_type).set_value(&mail.body);
		let message = Message::new(from_email)
			.set_subject(&mail.subject)
			.add_content(content)
			.add_personalization(personalization);

		let mut sender = Sender::new(self.api_key.clone(), None);
		if let Some(api_host) = &self.api_host {
			sender.set_host(api_host.to_string());
		}

		let start_time = std::time::Instant::now();
		let result = sender.send(&message).await;
		let duration = start_time.elapsed();

		match &result {
			Ok(_) => {
				info!("[EMAIL_LIFECYCLE] SendGrid API success for {} ({:?})", mail.to, duration);
				Ok(())
			},
			Err(e) => {
				error!(
					"[EMAIL_LIFECYCLE] SendGrid API failed for {} ({:?}): {:?}",
					mail.to, duration, e
				);
				Err(Error::SendEmailFailed)
			},
		}
	}
}

/// A mailer implementation that prints verification codes to console/logs instead of sending emails
pub struct ConsoleMailer;

impl Default for ConsoleMailer {
	fn default() -> Self {
		Self::new()
	}
}

impl ConsoleMailer {
	pub fn new() -> Self {
		Self
	}
}

#[async_trait]
impl MailerTrait for ConsoleMailer {
	async fn send(&self, mail: Mail) -> Result<(), Error> {
		// Extract verification code from the email body
		let verification_code = extract_verification_code(&mail.body);

		tracing::info!("==============================================");
		tracing::info!("Email Verification Code (Console Mailer)");
		tracing::info!("==============================================");
		tracing::info!("To: {}", mail.to);
		tracing::info!("Subject: {}", mail.subject);
		if let Some(code) = verification_code {
			tracing::info!("VERIFICATION CODE: {}", code);
		}
		tracing::info!("==============================================");

		Ok(())
	}
}

/// Extract verification code from email body
fn extract_verification_code(body: &str) -> Option<String> {
	// The template uses {{ verification_code }}, so after replacement it will be the actual code
	// Look for a 6-digit code pattern within the HTML structure
	// The code appears inside <p style="font-size: 16px; font-weight: 600;">CODE</p>

	// First try to find it with the specific HTML pattern
	if let Ok(re) = regex::Regex::new(r"<p[^>]*font-weight:\s*600[^>]*>(\d{6})</p>") {
		if let Some(captures) = re.captures(body) {
			if let Some(code) = captures.get(1) {
				return Some(code.as_str().to_string());
			}
		}
	}

	// Fallback: Look for any 6-digit code pattern in the body
	if let Ok(re) = regex::Regex::new(r"\b\d{6}\b") {
		if let Some(m) = re.find(body) {
			return Some(m.as_str().to_string());
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract_verification_code_from_html() {
		// Test with actual HTML structure from the template
		let html_with_code = r#"<div style="background-color: #e5e7eb; padding: 8px 4px">
			<p style="font-size: 16px; font-weight: 600;">123456</p>
		</div>"#;

		assert_eq!(extract_verification_code(html_with_code), Some("123456".to_string()));

		// Test with different formatting
		let html_with_code2 = r#"<p style="font-weight: 600; font-size: 16px;">987654</p>"#;
		assert_eq!(extract_verification_code(html_with_code2), Some("987654".to_string()));

		// Test fallback with plain text
		let plain_text = "Your verification code is 555555";
		assert_eq!(extract_verification_code(plain_text), Some("555555".to_string()));

		// Test with no code
		let no_code = "This text has no verification code";
		assert_eq!(extract_verification_code(no_code), None);
	}

	#[test]
	fn test_extract_verification_code_from_full_template() {
		// Test with the actual template after replacement
		let full_email =
			template::EMAIL_VERIFICATION_TEMPLATE.replace("{{ verification_code }}", "246810");
		assert_eq!(extract_verification_code(&full_email), Some("246810".to_string()));
	}
}
