use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn permissions(&self) -> Vec<String> {
        match self {
            Role::Admin => vec!["admin".into(), "user".into()],
            Role::User => vec!["user".into()],
        }
    }

    pub fn from_str(role: &str) -> Self {
        match role {
            "admin" => Role::Admin,
            _ => Role::User,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogoutRequest {
    #[serde(default)]
    pub all_devices: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub csrf_token: String,
    pub user: UserSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub csrf_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionResponse {
    pub session_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserSummary {
    pub id: i32,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SigningKeyMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    pub algorithm: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
}
