use std::path::PathBuf;

use crate::auth::{AuthError, AuthResult};

/// Authentication configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub issuer: String,
    pub audience: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
    pub session_cookie_ttl_secs: i64,
    pub refresh_cookie_name: String,
    pub csrf_cookie_name: String,
    pub csrf_header_name: String,
    pub session_cookie_name: String,
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
    pub jwt_private_key_path: PathBuf,
    pub jwt_kid: Option<String>,
}

impl AuthConfig {
    pub fn from_env() -> AuthResult<Self> {
        let issuer = std::env::var("NEXUS_JWT_ISSUER").unwrap_or_else(|_| "http://localhost".into());
        let audience = std::env::var("NEXUS_JWT_AUDIENCE").unwrap_or_else(|_| "nexus-api".into());
        let access_token_ttl_secs = std::env::var("NEXUS_ACCESS_TOKEN_TTL_SECS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(900);
        let refresh_token_ttl_secs = std::env::var("NEXUS_REFRESH_TOKEN_TTL_SECS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(7 * 24 * 60 * 60);
        let session_cookie_ttl_secs = std::env::var("NEXUS_SESSION_COOKIE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(30 * 60);
        let refresh_cookie_name = std::env::var("NEXUS_REFRESH_COOKIE_NAME")
            .unwrap_or_else(|_| "nexus_refresh_token".into());
        let csrf_cookie_name = std::env::var("NEXUS_CSRF_COOKIE_NAME")
            .unwrap_or_else(|_| "nexus_csrf".into());
        let csrf_header_name = std::env::var("NEXUS_CSRF_HEADER_NAME")
            .unwrap_or_else(|_| "X-CSRF-Token".into());
        let session_cookie_name = std::env::var("NEXUS_SESSION_COOKIE_NAME")
            .unwrap_or_else(|_| "nexus_session".into());
        let cookie_domain = std::env::var("NEXUS_COOKIE_DOMAIN").ok();
        let cookie_secure = std::env::var("NEXUS_COOKIE_SECURE")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(true);
        let jwt_private_key_path = std::env::var("NEXUS_JWT_PRIVATE_KEY_PATH")
            .map(PathBuf::from)
            .map_err(|_| AuthError::Config("NEXUS_JWT_PRIVATE_KEY_PATH is required".into()))?;
        let jwt_kid = std::env::var("NEXUS_JWT_KID").ok();

        Ok(Self {
            issuer,
            audience,
            access_token_ttl_secs,
            refresh_token_ttl_secs,
            session_cookie_ttl_secs,
            refresh_cookie_name,
            csrf_cookie_name,
            csrf_header_name,
            session_cookie_name,
            cookie_domain,
            cookie_secure,
            jwt_private_key_path,
            jwt_kid,
        })
    }
}
