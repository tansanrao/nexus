use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::auth::{AuthConfig, AuthResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub token_version: i32,
}

#[derive(Debug, Clone)]
pub struct SignedAccessToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct JwtMetadata {
    pub kid: Option<String>,
    pub algorithm: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_ttl_secs: i64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    issuer: String,
    audience: String,
    access_token_ttl: Duration,
    kid: Option<String>,
}

impl JwtService {
    pub fn from_config(config: &AuthConfig) -> AuthResult<Self> {
        let secret_bytes = config.jwt_secret.as_bytes();
        let encoding_key = EncodingKey::from_secret(secret_bytes);
        let decoding_key = DecodingKey::from_secret(secret_bytes);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[config.audience.clone()]);
        validation.set_issuer(&[config.issuer.clone()]);
        validation.leeway = 30;

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            issuer: config.issuer.clone(),
            audience: config.audience.clone(),
            access_token_ttl: Duration::seconds(config.access_token_ttl_secs),
            kid: config.jwt_kid.clone(),
        })
    }

    pub fn issue_access_token(
        &self,
        user_id: i32,
        email: &str,
        role: &str,
        permissions: &[String],
        token_version: i32,
    ) -> AuthResult<SignedAccessToken> {
        let now = Utc::now();
        let expires_at = now + self.access_token_ttl;
        let jti = Uuid::new_v4().to_string();

        let mut header = Header::new(Algorithm::HS256);
        header.kid = self.kid.clone();

        let claims = AccessTokenClaims {
            sub: user_id.to_string(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            jti,
            email: email.to_string(),
            role: role.to_string(),
            permissions: permissions.to_vec(),
            token_version,
        };

        let token = encode(&header, &claims, &self.encoding_key)?;

        Ok(SignedAccessToken { token, expires_at })
    }

    pub fn decode_access_token(&self, token: &str) -> AuthResult<AccessTokenClaims> {
        let token_data = decode::<AccessTokenClaims>(token, &self.decoding_key, &self.validation)?;
        Ok(token_data.claims)
    }

    pub fn metadata(&self) -> JwtMetadata {
        JwtMetadata {
            kid: self.kid.clone(),
            algorithm: "HS256".to_string(),
            issuer: self.issuer.clone(),
            audience: self.audience.clone(),
            access_token_ttl_secs: self.access_token_ttl.num_seconds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthConfig;

    const TEST_JWT_SECRET: &str = "super-secret-test-key";

    fn make_test_config() -> AuthConfig {
        AuthConfig {
            issuer: "https://nexus.test".into(),
            audience: "nexus-api".into(),
            access_token_ttl_secs: 900,
            refresh_token_ttl_secs: 604800,
            session_cookie_ttl_secs: 1800,
            refresh_cookie_name: "nexus_refresh_token".into(),
            csrf_cookie_name: "nexus_csrf".into(),
            csrf_header_name: "X-CSRF-Token".into(),
            session_cookie_name: "nexus_session".into(),
            cookie_domain: None,
            cookie_secure: false,
            jwt_secret: TEST_JWT_SECRET.into(),
            jwt_kid: Some("test-kid".into()),
        }
    }

    #[test]
    fn issues_and_decodes_access_tokens() {
        let config = make_test_config();
        let service = JwtService::from_config(&config).expect("jwt service");

        let permissions = vec!["user".to_string()];
        let token = service
            .issue_access_token(42, "user@example.com", "user", &permissions, 0)
            .expect("issue token");

        let claims = service
            .decode_access_token(&token.token)
            .expect("decode token");

        assert_eq!(claims.sub, "42");
        assert_eq!(claims.email, "user@example.com");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.token_version, 0);
        assert_eq!(claims.permissions, permissions);
        assert!(claims.exp > claims.iat);
    }
}
