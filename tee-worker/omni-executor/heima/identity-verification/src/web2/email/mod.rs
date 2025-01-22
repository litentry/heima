mod mailer;

use crate::helpers;
use mailer::{template, Error, Mail, MailerTrait};

pub fn generate_verification_code() -> String {
	helpers::generate_otp(6)
}

pub async fn send_verification_email(
	mailer: &impl MailerTrait,
	to: String,
	verification_code: String,
) -> Result<(), Error> {
	let mail = Mail {
		to,
		subject: "Verify your email".to_string(),
		body: template::EMAIL_VERIFICATION_TEMPLATE
			.replace("{{ verification_code }}", &verification_code),
		content_type: "text/html".to_string(),
	};
	mailer.send(mail).await
}
