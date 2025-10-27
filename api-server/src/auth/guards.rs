use chrono::Utc;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::State;
use rocket_db_pools::sqlx::{self, Row};

use crate::auth::jwt::AccessTokenClaims;
use crate::auth::responses::Role;
use crate::auth::{AuthError, AuthResult, AuthState};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub role: Role,
    pub permissions: Vec<String>,
    pub token_version: i32,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        matches!(self.role, Role::Admin)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match extract_user(request).await {
            Ok(user) => Outcome::Success(user),
            Err(err) => Outcome::Failure((err.status(), err)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequireAdmin(pub AuthUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequireAdmin {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthUser::from_request(request).await {
            Outcome::Success(user) => {
                if user.is_admin() {
                    Outcome::Success(RequireAdmin(user))
                } else {
                    Outcome::Failure((Status::Forbidden, AuthError::Forbidden))
                }
            }
            Outcome::Failure(err) => Outcome::Failure(err),
            Outcome::Forward(_) => Outcome::Failure((Status::Unauthorized, AuthError::Unauthorized)),
        }
    }
}

async fn extract_user(request: &Request<'_>) -> AuthResult<AuthUser> {
    let token = bearer_token_from_request(request)?;

    let auth_state = request
        .guard::<&State<AuthState>>()
        .await
        .succeeded()
        .ok_or_else(|| AuthError::Config("AuthState missing from state".into()))?;

    let pool = request
        .guard::<&State<sqlx::PgPool>>()
        .await
        .succeeded()
        .ok_or_else(|| AuthError::Config("database pool missing from state".into()))?;

    let claims = auth_state.jwt_service.decode_access_token(token)?;
    validate_claims(&claims)?;

    let user_id: i32 = claims.sub.parse().map_err(|_| AuthError::Unauthorized)?;

    let row = sqlx::query(
        "SELECT email, role, token_version, disabled FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool.inner())
    .await?;

    let row = row.ok_or(AuthError::Unauthorized)?;
    let email: String = row.try_get("email")?;
    let role_str: String = row.try_get("role")?;
    let token_version: i32 = row.try_get("token_version")?;
    let disabled: bool = row.try_get("disabled")?;

    if disabled {
        return Err(AuthError::AccountDisabled);
    }

    if token_version != claims.token_version {
        return Err(AuthError::TokenInvalid);
    }

    let role = Role::from_str(&role_str);
    if role.as_str() != claims.role {
        return Err(AuthError::TokenInvalid);
    }

    let permissions = role.permissions();

    Ok(AuthUser {
        id: user_id,
        email,
        role,
        permissions,
        token_version,
    })
}

fn bearer_token_from_request(request: &Request<'_>) -> AuthResult<&str> {
    let header = request
        .headers()
        .get_one("Authorization")
        .ok_or(AuthError::Unauthorized)?;
    let mut parts = header.splitn(2, ' ');
    let scheme = parts.next().unwrap_or_default();
    let token = parts.next().unwrap_or_default();
    if scheme.eq_ignore_ascii_case("Bearer") && !token.is_empty() {
        Ok(token)
    } else {
        Err(AuthError::Unauthorized)
    }
}

fn validate_claims(claims: &AccessTokenClaims) -> AuthResult<()> {
    let now = Utc::now().timestamp();
    if claims.exp < now {
        return Err(AuthError::TokenExpired);
    }
    Ok(())
}
