#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use super::{Mail, Mailer};
use crate::alloc::{string::String, vec, vec::Vec};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http_req::response::Headers;
use itc_rest_client::{
	error::Error as HttpError,
	http_client::{HttpClient, SendWithCertificateVerification},
	rest_client::RestClient,
	RestPath, RestPost,
};
use serde::Serialize;
use url::Url;

#[derive(Serialize)]
struct Personalization {
	to: Vec<Email>,
}

#[derive(Serialize, Clone)]
pub(crate) struct Email {
	pub email: String,
	pub name: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct Content {
	#[serde(rename(serialize = "type"))]
	content_type: String,
	value: String,
}

#[derive(Serialize)]
pub(crate) struct SendGridMail {
	personalizations: Vec<Personalization>,
	from: Email,
	subject: String,
	content: Vec<Content>,
}

impl SendGridMail {
	pub fn new(from_email: String, mail: Mail) -> Self {
		let content = vec![Content { content_type: String::from("text/html"), value: mail.body }];
		let to = Email { email: mail.to, name: None };
		let from = Email { email: from_email, name: Some(String::from("Litentry")) };
		Self {
			personalizations: vec![Personalization { to: vec![to] }],
			from,
			subject: mail.subject,
			content,
		}
	}
}

impl RestPath<String> for SendGridMail {
	fn get_path(path: String) -> Result<String, HttpError> {
		Ok(path)
	}
}

pub struct SendGridMailer {
	client: RestClient<HttpClient<SendWithCertificateVerification>>,
	from: String,
}

impl SendGridMailer {
	pub fn new(api_key: String, from_email: String) -> Self {
		let base_url = Url::parse("https://api.sendgrid.com/v3/mail/send").unwrap();
		let authorization = std::format!("Bearer {}", api_key);

		let mut headers = Headers::new();
		headers.insert(AUTHORIZATION.as_str(), &authorization);
		headers.insert(CONTENT_TYPE.as_str(), "application/json");

		let http_client = HttpClient::new(
			SendWithCertificateVerification::new(vec![]),
			true,
			None,
			Some(headers),
			None,
		);

		Self { client: RestClient::new(http_client, base_url), from: from_email }
	}
}

impl Mailer for SendGridMailer {
	fn send(&mut self, mail: Mail) -> Result<(), String> {
		let sendgrid_mail = SendGridMail::new(self.from.clone(), mail);
		self.client
			.post(String::default(), &sendgrid_mail)
			.map_err(|e| std::format!("Failed to send verification email: {:?}", e))?;

		Ok(())
	}
}
