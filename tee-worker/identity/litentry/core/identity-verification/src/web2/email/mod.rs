mod verification_code_store;
pub use verification_code_store::*;
mod mailer;
pub use mailer::*;

use crate::alloc::string::String;

pub fn send_verification_email(
	mailer: &mut impl Mailer,
	to_email: String,
	verification_code: String,
) -> Result<(), String> {
	let mail = mailer::Mail {
		to: to_email,
		subject: String::from("Verify your email address"),
		body: template::EMAIL_VERIFICATION_TEMPLATE
			.replace("{{ verification_code }}", &verification_code),
	};

	mailer.send(mail)
}
