use crate::server::RpcContext;
use heima_identity_verification::web2::email::{
	generate_verification_code, send_verification_email,
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use primitives::{Identity, Web2IdentityType};
use storage::{Storage, VerificationCodeStorage};

pub fn register_request_email_verification_code(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_requestEmailVerificationCode", |params, ctx, _| async move {
			let Ok(email) = params.one::<String>() else {
				return Err(ErrorCode::ParseError.into());
			};
			let email_identity = Identity::from_web2_account(&email, Web2IdentityType::Email);
			let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
			let verification_code = generate_verification_code();

			verification_code_storage
				.insert(email_identity.hash(), verification_code.clone())
				.map_err(|_| ErrorCode::InternalError)?;

			send_verification_email(&ctx.mailer, email, verification_code)
				.await
				.map_err(|_| {
					log::error!("Failed to send verification email");
					ErrorCode::InternalError
				})?;

			Ok::<(), ErrorObject>(())
		})
		.expect("Failed to register omni_requestEmailVerificationCode method");
}
