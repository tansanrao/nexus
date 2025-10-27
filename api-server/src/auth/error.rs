use rocket::http::Status;
use thiserror::Error;

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account locked")]
    AccountLocked,
    #[error("account disabled")]
    AccountDisabled,
    #[error("token expired")]
    TokenExpired,
    #[error("token invalid")]
    TokenInvalid,
    #[error("token reuse detected")]
    TokenReuseDetected { user_id: i32 },
    #[error("csrf token missing")]
    CsrfMissing,
    #[error("csrf token mismatch")]
    CsrfMismatch,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("configuration error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Sqlx(#[from] rocket_db_pools::sqlx::Error),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("argon2 parameter error: {0}")]
    Argon2(String),
    #[error("password hashing error: {0}")]
    PasswordHash(String),
    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("time error: {0}")]
    Time(#[from] time::error::Error),
    #[error("unexpected error: {0}")]
    Other(String),
}

impl AuthError {
    pub fn status(&self) -> Status {
        match self {
            AuthError::InvalidCredentials => Status::Unauthorized,
            AuthError::AccountLocked => Status::Locked,
            AuthError::AccountDisabled => Status::Forbidden,
            AuthError::TokenExpired
            | AuthError::TokenInvalid
            | AuthError::TokenReuseDetected { .. } => Status::Unauthorized,
            AuthError::Unauthorized => Status::Unauthorized,
            AuthError::Forbidden => Status::Forbidden,
            AuthError::CsrfMissing => Status::BadRequest,
            AuthError::CsrfMismatch => Status::Unauthorized,
            AuthError::Config(_) => Status::InternalServerError,
            AuthError::Io(_)
            | AuthError::Sqlx(_)
            | AuthError::Jwt(_)
            | AuthError::Argon2(_)
            | AuthError::PasswordHash(_) => Status::InternalServerError,
            AuthError::Base64(_) | AuthError::Time(_) | AuthError::Other(_) => {
                Status::InternalServerError
            }
        }
    }
}

impl From<argon2::Error> for AuthError {
    fn from(err: argon2::Error) -> Self {
        AuthError::Argon2(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AuthError::PasswordHash(err.to_string())
    }
}
