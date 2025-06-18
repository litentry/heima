pub mod template;

use async_trait::async_trait;
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};
use tracing::error;

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
		let from_email = Email::new(&self.from_email).set_name(&self.from_name);
		let personalization = Personalization::new(Email::new(&mail.to));
		let content = Content::new().set_content_type(&mail.content_type).set_value(&mail.body);
		let message = Message::new(from_email)
			.set_subject(&mail.subject)
			.add_content(content)
			.add_personalization(personalization);
		let mut sender = Sender::new(self.api_key.clone(), None);
		// for mocking purpose
		if let Some(api_host) = &self.api_host {
			sender.set_host(api_host.to_string());
		}
		sender.send(&message).await.map_err(|e| {
			error!("Failed to send email: {:?}", e);
			Error::SendEmailFailed
		})?;

		Ok(())
	}
}
