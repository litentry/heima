#![allow(clippy::result_large_err)]

mod auth_token_key_store;
pub use auth_token_key_store::AuthTokenKeyStore;

mod config;
mod detailed_error;
mod error_code;
mod mailer_factory;

mod methods;
pub use methods::RpcResult;

mod middlewares;
mod oauth2_factory;

mod server;
pub use server::{start_server, RpcContext};

pub mod utils;
mod verify_auth;

pub use executor_crypto::shielding_key::ShieldingKey;

// Removed unused hex imports - they may be used in other modules
use jsonrpsee::types::ErrorCode;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
