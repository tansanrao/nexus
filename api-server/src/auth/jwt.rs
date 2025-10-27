use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::auth::{AuthConfig, AuthError, AuthResult};

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
    decoding_key: DecodingKey<'static>,
    validation: Validation,
    issuer: String,
    audience: String,
    access_token_ttl: Duration,
    kid: Option<String>,
}

impl JwtService {
    pub fn from_config(config: &AuthConfig) -> AuthResult<Self> {
        let key_bytes = std::fs::read(&config.jwt_private_key_path)?;
        let encoding_key = EncodingKey::from_rsa_pem(&key_bytes)?;
        let decoding_key = DecodingKey::from_rsa_pem(&key_bytes)?;

        let mut validation = Validation::new(Algorithm::RS256);
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
        let mut header = Header::new(Algorithm::RS256);
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
            algorithm: "RS256".to_string(),
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
    use std::fs;
    use tempfile::NamedTempFile;

    const TEST_RSA_PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEArELvhKIbrhh3t/lWS/jGyrv/6zsuOgy3xJZXIxSlIh9KDDYz
GPIYJPf607ylZQxlj9au5J7l7JRIa9sxCSvbMoh6x8/YHBNmFPyzkCq+DTTZH4Wk
EvpZrxnYl3+hskkGacdfD/dbmsaHEttPtPdNITlNISPrrzjxEkvi5vN0CWZnxgZs
WHLrs8qgct4bVX32asEGOcubqpvnDONbJdKp1AzZXewNaw98HoxY/sCATXCWGad4
ukONWZ9sCe0SG9xTPmepcNxR/dhpytRaCvy2xS4dcUJ59lp2rSHIUrFm4TRfxWo/
GdSEJxP2wm2yp5q2ggzA6VMBUuP28CE2ik9n7QIDAQABAoIBACnBovpRamjJ9RFD
T0Qktplzt34/rv2y0gQFFnPCQCI0l/g8VigMnUYu114mmygSuHbEyUnRa7Ysnp6I
eEs7FowaEbsoOoBZwnPBasx+U+nzHtOZi1NvXLiJiRt2PI2xTmzrP3OpGAs9ZwYu
49Qf41Izp+rp4Gpt4N/4xbSKnJzfUE9YwEpHbRj08Ur7dngXuddbLCdZjgNVCn//
qhCpNMSG5iBrvYQ1TDQkDVkVIHK2VWxCsvLhUMfu1SRUbIn7FMnxxh7j8uAqXma8
u7Vv3WvV50cMTnJB0rvhdaIg6O7Y5e8uiSS3tbakyFHrr2ow+TFKI6/CMc4e+r0C
wheZuBkCgYEA2xvOs6JgVg72UuX4w23/DYta+wNX0muuI8cA5W8SUIxTox83nCZI
O86QZGvHVvsmQ1T+VEHkDUPkQnVvukKjFpV7VLNdj/s+7Lt2pSRAFm8tkj+Ber4u
oYS2KGKfOuxH0CwA6BZCJbHt0kWPnWCKAYeUEqfd7yqSeStutY4vnfkCgYEAyUPi
milbUtrbVTnkyL/pRFA8kZuZnP0uMxdgFXsCox0EZ2zrZvXP2IHnKvJOtYhoE6E8
Itp7eP2Pu4LLdet6vQIHE3xUrKYBX770yyxFHWwJn1m1ZxGWrzeGUoSZJXRTEr8R
UzDS5ZayD9VrxehE5E156OkK6ksENk3v4OexppUCgYEA1dpdM8zPFA/EcYLN+wi4
AKM8KHTJ2bGJpJfOEyEGkiF0XGjSoRBoPh9NpQXg6M92OA+Tr+8jw6K4/fibFQOH
JDq/xhrOvgHuF6aclXA9MOhQZUagfIl0/+aE2APx/9Ov/8mDFQLsitgQE8Qa+PLJ
n9aROmgnYBCAJ82xX3iolxkCgYEAuqsr0K/q873pD/LSLx9PyvxgMOyQXPq1js1v
YHzmxUJ0gziSXLxAOh7BuSNjvRr27L3ueKULP/xtAw0ciBIPlJ380iXOoxKU06jY
glhdAhziD9m0VhQKHhjxjDdPk12AbzKnbvEpqadLH0Ri4Pu8acMx/sOmTAensHY4
tfAu5MECgYBESDe8c8mjig+ktC3P5K8FeR+pNGqp7hjCiRP2J+IPOQhQLYCu2RfU
5+f+Rbk7YIByHjrY4MpcaNvMnSQHFI49O/xBiSGzpkdnLfkZ4Q6Xd6St56qfgzhf
OmSlD5OcHBaImD0VICliqmth4eOzV1tsrnkUBA1DHRAM1Z2/Ausa2Q==
-----END RSA PRIVATE KEY-----"#;

    fn make_test_config(path: &std::path::Path) -> AuthConfig {
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
            jwt_private_key_path: path.to_path_buf(),
            jwt_kid: Some("test-kid".into()),
        }
    }

    #[test]
    fn issues_and_decodes_access_tokens() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), TEST_RSA_PRIVATE_KEY).expect("write key");
        let config = make_test_config(file.path());
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
