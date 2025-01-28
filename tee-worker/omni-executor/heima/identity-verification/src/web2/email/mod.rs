mod mailer;
pub use mailer::Mailer;

pub fn generate_verification_code() -> String {
	crate::helpers::generate_otp(6)
}

pub async fn send_verification_email(
	mailer: &impl mailer::MailerTrait,
	to: String,
	verification_code: String,
) -> Result<(), mailer::Error> {
	let mail = mailer::Mail {
		to,
		subject: "Verify your email".to_string(),
		body: mailer::template::EMAIL_VERIFICATION_TEMPLATE
			.replace("{{ verification_code }}", &verification_code),
		content_type: "text/html".to_string(),
	};
	mailer.send(mail).await
}
