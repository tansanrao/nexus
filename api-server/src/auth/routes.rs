use std::ops::DerefMut;

use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{State, get, post};
use rocket_db_pools::sqlx::{self, Row};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use rocket_okapi::request::OpenApiFromRequest;
use time::Duration as TimeDuration;

use crate::auth::guards::{AuthUser, RequireAdmin};
use crate::auth::refresh_store::RefreshTokenIssued;
use crate::auth::responses::{
    LoginRequest, LoginResponse, LogoutRequest, RefreshResponse, Role, SessionResponse,
    SigningKeyMetadata, UserSummary,
};
use crate::auth::{AuthError, AuthResult, AuthState};

type AuthRouteResult<T> = Result<Json<T>, status::Custom<Json<AuthErrorResponse>>>;

#[derive(Debug, serde::Serialize, JsonSchema)]
pub struct AuthErrorResponse {
    pub status: u16,
    pub message: String,
}

#[openapi(tag = "Auth")]
#[post("/auth/signup")]
pub async fn signup_blocked() -> status::Custom<Json<AuthErrorResponse>> {
    respond_message(
        Status::Forbidden,
        "Signup is disabled; contact an administrator to provision an account.",
    )
}

#[openapi(tag = "Auth")]
#[post("/auth/login", data = "<payload>")]
pub async fn login(
    state: &State<AuthState>,
    pool: &State<sqlx::PgPool>,
    cookies: &CookieJar<'_>,
    payload: Json<LoginRequest>,
) -> AuthRouteResult<LoginResponse> {
    let email = payload.email.trim().to_lowercase();
    let password = payload.password.trim();

    if email.is_empty() || password.is_empty() {
        return Err(respond_message(
            Status::BadRequest,
            "Email and password are required",
        ));
    }

    let now = Utc::now();
    let mut tx = pool
        .begin()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    let row = sqlx::query(
        r#"
        SELECT u.id, u.email, u.display_name, u.role, u.token_version, u.disabled,
               cred.password_hash, cred.failed_attempts, cred.locked_until
        FROM users u
        LEFT JOIN local_user_credentials cred ON cred.user_id = u.id
        WHERE lower(u.email) = $1
        FOR UPDATE
        "#,
    )
    .bind(&email)
    .fetch_optional(tx.deref_mut())
    .await
    .map_err(|err| respond_error(AuthError::from(err)))?;

    let row = match row {
        Some(row) => row,
        None => return Err(invalid_credentials()),
    };

    let user_id: i32 = row
        .try_get("id")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let db_email: String = row
        .try_get("email")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let display_name: Option<String> = row
        .try_get("display_name")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let role_str: String = row
        .try_get("role")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let token_version: i32 = row
        .try_get("token_version")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let disabled: bool = row
        .try_get("disabled")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let password_hash: Option<String> = row
        .try_get("password_hash")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let failed_attempts: i32 = row.try_get("failed_attempts").unwrap_or(0);
    let locked_until: Option<DateTime<Utc>> = row
        .try_get("locked_until")
        .map_err(|err| respond_error(AuthError::from(err)))?;

    if password_hash.is_none() {
        return Err(invalid_credentials());
    }

    if disabled {
        return Err(respond_error(AuthError::AccountDisabled));
    }

    if let Some(lock_time) = locked_until {
        if lock_time > now {
            return Err(respond_error(AuthError::AccountLocked));
        }
    }

    let verified = state
        .password_service
        .verify_password(password, password_hash.as_ref().unwrap())
        .map_err(|err| respond_error(err))?;

    if !verified {
        handle_failed_attempt(&mut tx, user_id, failed_attempts, now)
            .await
            .map_err(respond_error)?;
        return Err(invalid_credentials());
    }

    reset_failed_attempts(&mut tx, user_id)
        .await
        .map_err(respond_error)?;

    sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
        .bind(now)
        .bind(user_id)
        .execute(tx.deref_mut())
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    let role = Role::from_str(&role_str);
    let permissions = role.permissions();
    let access_token = state
        .jwt_service
        .issue_access_token(
            user_id,
            &db_email,
            role.as_str(),
            &permissions,
            token_version,
        )
        .map_err(respond_error)?;

    let refresh_token = state
        .refresh_store
        .issue_token_tx(
            &mut tx,
            user_id,
            payload.device_fingerprint.as_deref(),
            now,
            Duration::seconds(state.config.refresh_token_ttl_secs),
        )
        .await
        .map_err(respond_error)?;

    tx.commit()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    set_refresh_cookie(cookies, state, &refresh_token);
    let csrf_token = generate_random_token();
    set_csrf_cookie(cookies, state, &csrf_token, refresh_token.expires_at);

    let response = LoginResponse {
        access_token: access_token.token.clone(),
        access_token_expires_at: access_token.expires_at,
        refresh_token_expires_at: refresh_token.expires_at,
        csrf_token,
        user: UserSummary {
            id: user_id,
            email: db_email,
            display_name,
            role,
        },
    };

    Ok(Json(response))
}

#[openapi(tag = "Auth")]
#[post("/auth/refresh")]
pub async fn refresh(
    state: &State<AuthState>,
    pool: &State<sqlx::PgPool>,
    cookies: &CookieJar<'_>,
    csrf: CsrfToken,
) -> AuthRouteResult<RefreshResponse> {
    let refresh_cookie = match cookies.get(&state.config.refresh_cookie_name) {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(respond_error(AuthError::Unauthorized)),
    };

    let csrf_cookie = match cookies.get(&state.config.csrf_cookie_name) {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(respond_error(AuthError::CsrfMissing)),
    };

    if csrf_cookie != csrf.0 {
        return Err(respond_error(AuthError::CsrfMismatch));
    }

    let now = Utc::now();
    let mut tx = pool
        .begin()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    let rotation = match state
        .refresh_store
        .rotate_token_tx(
            &mut tx,
            &refresh_cookie,
            None,
            now,
            Duration::seconds(state.config.refresh_token_ttl_secs),
        )
        .await
    {
        Ok(rotation) => rotation,
        Err(AuthError::TokenReuseDetected { user_id }) => {
            handle_token_reuse(state, pool, user_id, now).await?;
            return Err(respond_error(AuthError::Unauthorized));
        }
        Err(err) => return Err(respond_error(err)),
    };

    let user_row = sqlx::query("SELECT email, role, token_version FROM users WHERE id = $1")
        .bind(rotation.user_id)
        .fetch_one(tx.deref_mut())
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    let email: String = user_row
        .try_get("email")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let role_str: String = user_row
        .try_get("role")
        .map_err(|err| respond_error(AuthError::from(err)))?;
    let token_version: i32 = user_row
        .try_get("token_version")
        .map_err(|err| respond_error(AuthError::from(err)))?;

    let role = Role::from_str(&role_str);
    let permissions = role.permissions();

    let access_token = state
        .jwt_service
        .issue_access_token(
            rotation.user_id,
            &email,
            role.as_str(),
            &permissions,
            token_version,
        )
        .map_err(respond_error)?;

    tx.commit()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    set_refresh_cookie(cookies, state, &rotation.new_token);
    let csrf_token = generate_random_token();
    set_csrf_cookie(cookies, state, &csrf_token, rotation.new_token.expires_at);

    let response = RefreshResponse {
        access_token: access_token.token,
        access_token_expires_at: access_token.expires_at,
        refresh_token_expires_at: rotation.new_token.expires_at,
        csrf_token,
    };

    Ok(Json(response))
}

#[openapi(tag = "Auth")]
#[post("/auth/logout", data = "<payload>")]
pub async fn logout(
    state: &State<AuthState>,
    pool: &State<sqlx::PgPool>,
    cookies: &CookieJar<'_>,
    payload: Json<LogoutRequest>,
) -> Result<Status, status::Custom<Json<AuthErrorResponse>>> {
    let refresh_cookie = match cookies.get(&state.config.refresh_cookie_name) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            clear_auth_cookies(cookies, state);
            return Ok(Status::NoContent);
        }
    };

    let now = Utc::now();
    let mut tx = pool
        .begin()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    let revoke_result = state
        .refresh_store
        .revoke_token_tx(&mut tx, &refresh_cookie, now)
        .await
        .map_err(respond_error)?;

    if payload.all_devices.unwrap_or(false) {
        if let Some(user_id) = revoke_result {
            state
                .refresh_store
                .revoke_all_for_user_tx(&mut tx, user_id, now)
                .await
                .map_err(respond_error)?;
            increment_token_version(&mut tx, user_id)
                .await
                .map_err(respond_error)?;
            // TODO: integrate with SSE/WebSocket hub to drop sessions for user_id.
        }
    }

    tx.commit()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    clear_auth_cookies(cookies, state);

    Ok(Status::NoContent)
}

#[openapi(tag = "Auth")]
#[post("/auth/session")]
pub async fn session_cookie(
    state: &State<AuthState>,
    cookies: &CookieJar<'_>,
    user: AuthUser,
) -> AuthRouteResult<SessionResponse> {
    let permissions = user.permissions.clone();
    let token = state
        .jwt_service
        .issue_access_token(
            user.id,
            &user.email,
            user.role.as_str(),
            &permissions,
            user.token_version,
        )
        .map_err(respond_error)?;

    let mut cookie = Cookie::build((
        state.config.session_cookie_name.clone(),
        token.token.clone(),
    ))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Lax)
    .secure(state.config.cookie_secure)
    .max_age(TimeDuration::seconds(state.config.session_cookie_ttl_secs))
    .build();

    if let Some(domain) = &state.config.cookie_domain {
        cookie.set_domain(domain.clone());
    }

    cookies.add(cookie);

    Ok(Json(SessionResponse {
        session_expires_at: token.expires_at,
    }))
}

#[openapi(tag = "Auth")]
#[get("/auth/keys")]
pub async fn signing_keys(
    state: &State<AuthState>,
    _admin: RequireAdmin,
) -> AuthRouteResult<SigningKeyMetadata> {
    let jwt_meta = state.jwt_service.metadata();
    let response = SigningKeyMetadata {
        kid: jwt_meta.kid,
        algorithm: jwt_meta.algorithm,
        issuer: jwt_meta.issuer,
        audience: jwt_meta.audience,
        access_token_ttl_secs: jwt_meta.access_token_ttl_secs,
        refresh_token_ttl_secs: state.config.refresh_token_ttl_secs,
    };

    Ok(Json(response))
}

#[derive(Debug, OpenApiFromRequest)]
pub struct CsrfToken(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CsrfToken {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = match request.guard::<&State<AuthState>>().await {
            Outcome::Success(state) => state,
            _ => {
                let err = AuthError::Config("AuthState not available".into());
                return Outcome::Error((err.status(), err));
            }
        };

        if let Some(value) = request.headers().get_one(&state.config.csrf_header_name) {
            if !value.is_empty() {
                return Outcome::Success(CsrfToken(value.to_string()));
            }
        }

        let err = AuthError::CsrfMissing;
        let status = err.status();
        Outcome::Error((status, err))
    }
}

fn respond_error(err: AuthError) -> status::Custom<Json<AuthErrorResponse>> {
    let status = err.status();
    status::Custom(
        status,
        Json(AuthErrorResponse {
            status: status.code,
            message: err.to_string(),
        }),
    )
}

fn respond_message(
    status: Status,
    message: impl Into<String>,
) -> status::Custom<Json<AuthErrorResponse>> {
    status::Custom(
        status,
        Json(AuthErrorResponse {
            status: status.code,
            message: message.into(),
        }),
    )
}

fn invalid_credentials() -> status::Custom<Json<AuthErrorResponse>> {
    respond_error(AuthError::InvalidCredentials)
}

async fn handle_failed_attempt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: i32,
    failed_attempts: i32,
    now: DateTime<Utc>,
) -> AuthResult<()> {
    let new_attempts = failed_attempts + 1;
    let lock_until = if new_attempts >= 5 {
        Some(now + Duration::minutes(5))
    } else {
        None
    };

    sqlx::query(
        "UPDATE local_user_credentials SET failed_attempts = $1, locked_until = $2 WHERE user_id = $3",
    )
    .bind(new_attempts)
    .bind(lock_until)
    .bind(user_id)
    .execute(tx.deref_mut())
    .await?;

    Ok(())
}

async fn reset_failed_attempts(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: i32,
) -> AuthResult<()> {
    sqlx::query(
        "UPDATE local_user_credentials SET failed_attempts = 0, locked_until = NULL WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(tx.deref_mut())
    .await?;
    Ok(())
}

async fn increment_token_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: i32,
) -> AuthResult<()> {
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE id = $1")
        .bind(user_id)
        .execute(tx.deref_mut())
        .await?;
    Ok(())
}

async fn handle_token_reuse(
    state: &State<AuthState>,
    pool: &State<sqlx::PgPool>,
    user_id: i32,
    now: DateTime<Utc>,
) -> Result<(), status::Custom<Json<AuthErrorResponse>>> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    state
        .refresh_store
        .revoke_all_for_user_tx(&mut tx, user_id, now)
        .await
        .map_err(respond_error)?;
    increment_token_version(&mut tx, user_id)
        .await
        .map_err(respond_error)?;

    tx.commit()
        .await
        .map_err(|err| respond_error(AuthError::from(err)))?;

    // TODO: integrate SSE/WebSocket session teardown when hub is available.

    Ok(())
}

fn set_refresh_cookie(
    cookies: &CookieJar<'_>,
    state: &State<AuthState>,
    token: &RefreshTokenIssued,
) {
    let mut cookie = Cookie::build((
        state.config.refresh_cookie_name.clone(),
        token.token.clone(),
    ))
    .path("/api/v1/auth/refresh")
    .http_only(true)
    .same_site(SameSite::Lax)
    .secure(state.config.cookie_secure)
    .max_age(TimeDuration::seconds(state.config.refresh_token_ttl_secs))
    .build();

    if let Some(domain) = &state.config.cookie_domain {
        cookie.set_domain(domain.clone());
    }

    cookies.add(cookie);
}

fn set_csrf_cookie(
    cookies: &CookieJar<'_>,
    state: &State<AuthState>,
    token: &str,
    expires_at: DateTime<Utc>,
) {
    let max_age_secs = (expires_at - Utc::now()).num_seconds().max(0);
    let mut cookie = Cookie::build((state.config.csrf_cookie_name.clone(), token.to_string()))
        .path("/api/v1/auth/refresh")
        .http_only(false)
        .same_site(SameSite::Lax)
        .secure(state.config.cookie_secure)
        .max_age(TimeDuration::seconds(max_age_secs))
        .build();

    if let Some(domain) = &state.config.cookie_domain {
        cookie.set_domain(domain.clone());
    }

    cookies.add(cookie);
}

fn clear_auth_cookies(cookies: &CookieJar<'_>, state: &State<AuthState>) {
    for (name, path) in [
        (&state.config.refresh_cookie_name, "/api/v1/auth/refresh"),
        (&state.config.csrf_cookie_name, "/api/v1/auth/refresh"),
        (&state.config.session_cookie_name, "/"),
    ] {
        let mut cookie = Cookie::build((name.clone(), String::new()))
            .path(path)
            .removal()
            .build();

        if let Some(domain) = &state.config.cookie_domain {
            cookie.set_domain(domain.clone());
        }
        cookies.add(cookie);
    }
}

fn generate_random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
