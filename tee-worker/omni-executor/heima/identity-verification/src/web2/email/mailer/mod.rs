pub mod template;

use async_trait::async_trait;
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};

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
	api_key: String,
	from_email: String,
	from_name: String,
}

impl Mailer {
	pub fn new(api_key: String, from_email: String, from_name: String) -> Self {
		Self { api_key, from_email, from_name }
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
		let sender = Sender::new(self.api_key.clone(), None);
		sender.send(&message).await.map_err(|_| {
			log::error!("Failed to send email");
			Error::SendEmailFailed
		})?;

		Ok(())
	}
}
