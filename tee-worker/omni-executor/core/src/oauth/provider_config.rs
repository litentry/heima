pub trait OAuth2ProviderConfig {
	fn token_endpoint(&self) -> &'static str;
	fn authorize_endpoint(&self) -> &'static str;
	fn scope(&self) -> &'static str;
}

pub struct GoogleProviderConfig;

impl OAuth2ProviderConfig for GoogleProviderConfig {
	fn token_endpoint(&self) -> &'static str {
		"https://oauth2.googleapis.com/token"
	}

	fn authorize_endpoint(&self) -> &'static str {
		"https://accounts.google.com/o/oauth2/v2/auth"
	}

	fn scope(&self) -> &'static str {
		"openid email"
	}
}

pub struct AppleProviderConfig;

impl OAuth2ProviderConfig for AppleProviderConfig {
	fn token_endpoint(&self) -> &'static str {
		"https://appleid.apple.com/auth/token"
	}

	fn authorize_endpoint(&self) -> &'static str {
		"https://appleid.apple.com/auth/authorize"
	}

	fn scope(&self) -> &'static str {
		"email"
	}
}
