//! Authentication module: configuration, credential handling, token minting,
//! Rocket request guards, and HTTP route handlers.

use std::sync::Arc;

pub mod config;
pub mod error;
pub mod guards;
pub mod jwt;
pub mod passwords;
pub mod refresh_store;
pub mod responses;
pub mod routes;

pub use config::AuthConfig;
pub use error::{AuthError, AuthResult};
pub use guards::{AuthUser, RequireAdmin};
pub use jwt::JwtService;
pub use passwords::PasswordService;
pub use refresh_store::RefreshTokenStore;

#[derive(Clone)]
pub struct AuthState {
    pub config: AuthConfig,
    pub password_service: Arc<PasswordService>,
    pub jwt_service: Arc<JwtService>,
    pub refresh_store: RefreshTokenStore,
}

impl AuthState {
    pub fn new(
        config: AuthConfig,
        password_service: PasswordService,
        jwt_service: JwtService,
        refresh_store: RefreshTokenStore,
    ) -> Self {
        Self {
            config,
            password_service: Arc::new(password_service),
            jwt_service: Arc::new(jwt_service),
            refresh_store,
        }
    }
}
