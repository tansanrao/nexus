use std::ops::DerefMut;

use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine as _;
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use rocket_db_pools::sqlx::{self, PgPool, Postgres, Row, Transaction};
use sha2::{Digest, Sha512};
use uuid::Uuid;

use crate::auth::{AuthError, AuthResult};

const SECRET_LEN: usize = 32;
const SALT_LEN: usize = 16;

#[derive(Debug, Clone)]
pub struct RefreshTokenIssued {
    pub token_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct RefreshTokenRotation {
    pub user_id: i32,
    pub old_token_id: Uuid,
    pub new_token: RefreshTokenIssued,
}

#[derive(Debug, Clone)]
pub struct RefreshTokenStore {
    pool: PgPool,
}

impl RefreshTokenStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn issue_token_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: i32,
        fingerprint: Option<&str>,
        now: DateTime<Utc>,
        ttl: Duration,
    ) -> AuthResult<RefreshTokenIssued> {
        let token_id = Uuid::new_v4();
        let secret = generate_secret();
        let salt = generate_salt();
        let hashed_token = hash_secret(&secret, &salt);
        let expires_at = now + ttl;
        let stored = encode_hash(&salt, &hashed_token);

        sqlx::query(
            "INSERT INTO user_refresh_tokens (token_id, user_id, hashed_token, expires_at, device_fingerprint) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(token_id)
        .bind(user_id)
        .bind(stored)
        .bind(expires_at)
        .bind(fingerprint)
        .execute(tx.deref_mut())
        .await?;

        let token = format!("{}.{}", token_id, secret);

        Ok(RefreshTokenIssued {
            token_id,
            token,
            expires_at,
        })
    }

    pub async fn rotate_token_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        plain_token: &str,
        fingerprint: Option<&str>,
        now: DateTime<Utc>,
        ttl: Duration,
    ) -> AuthResult<RefreshTokenRotation> {
        let parsed = ParsedRefreshToken::parse(plain_token)?;

        let row = sqlx::query(
            "SELECT user_id, hashed_token, expires_at, revoked_at FROM user_refresh_tokens WHERE token_id = $1 FOR UPDATE",
        )
        .bind(parsed.token_id)
        .fetch_optional(tx.deref_mut())
        .await?;

        let row = match row {
            Some(row) => row,
            None => return Err(AuthError::TokenInvalid),
        };

        let user_id: i32 = row.try_get("user_id")?;
        let hashed: String = row.try_get("hashed_token")?;
        let expires_at: DateTime<Utc> = row.try_get("expires_at")?;
        let revoked_at: Option<DateTime<Utc>> = row.try_get("revoked_at")?;

        if let Some(_) = revoked_at {
            return Err(AuthError::TokenReuseDetected { user_id });
        }

        if expires_at <= now {
            return Err(AuthError::TokenExpired);
        }

        if !verify_secret(&parsed.secret, &hashed)? {
            return Err(AuthError::TokenInvalid);
        }

        sqlx::query(
            "UPDATE user_refresh_tokens SET revoked_at = $1, last_used_at = $1 WHERE token_id = $2",
        )
        .bind(now)
        .bind(parsed.token_id)
        .execute(tx.deref_mut())
        .await?;

        let new_token = self
            .issue_token_tx(tx, user_id, fingerprint, now, ttl)
            .await?;

        Ok(RefreshTokenRotation {
            user_id,
            old_token_id: parsed.token_id,
            new_token,
        })
    }

    pub async fn revoke_token_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        plain_token: &str,
        now: DateTime<Utc>,
    ) -> AuthResult<Option<i32>> {
        let parsed = ParsedRefreshToken::parse(plain_token)?;

        let row = sqlx::query(
            "SELECT user_id FROM user_refresh_tokens WHERE token_id = $1",
        )
        .bind(parsed.token_id)
        .fetch_optional(tx.deref_mut())
        .await?;

        if let Some(row) = row {
            let user_id: i32 = row.try_get("user_id")?;
            sqlx::query(
                "UPDATE user_refresh_tokens SET revoked_at = $1 WHERE token_id = $2",
            )
            .bind(now)
            .bind(parsed.token_id)
            .execute(tx.deref_mut())
            .await?;

            Ok(Some(user_id))
        } else {
            Ok(None)
        }
    }

    pub async fn revoke_all_for_user_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: i32,
        now: DateTime<Utc>,
    ) -> AuthResult<u64> {
        let result = sqlx::query(
            "UPDATE user_refresh_tokens SET revoked_at = $1 WHERE user_id = $2 AND revoked_at IS NULL",
        )
        .bind(now)
        .bind(user_id)
        .execute(tx.deref_mut())
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn purge_expired(&self, now: DateTime<Utc>) -> AuthResult<u64> {
        let mut conn = self.pool.acquire().await?;
        let result = sqlx::query(
            "DELETE FROM user_refresh_tokens WHERE expires_at <= $1 OR (revoked_at IS NOT NULL AND revoked_at <= $2)",
        )
        .bind(now)
        .bind(now - Duration::days(30))
        .execute(&mut *conn)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug)]
struct ParsedRefreshToken {
    token_id: Uuid,
    secret: String,
}

impl ParsedRefreshToken {
    fn parse(token: &str) -> AuthResult<Self> {
        let mut parts = token.splitn(2, '.');
        let token_id = parts
            .next()
            .ok_or_else(|| AuthError::TokenInvalid)?
            .parse::<Uuid>()
            .map_err(|_| AuthError::TokenInvalid)?;
        let secret = parts
            .next()
            .ok_or_else(|| AuthError::TokenInvalid)?
            .to_string();

        Ok(Self { token_id, secret })
    }
}

fn generate_secret() -> String {
    let mut bytes = [0u8; SECRET_LEN];
    rand::thread_rng().fill_bytes(&mut bytes);
    STANDARD_NO_PAD.encode(bytes)
}

fn generate_salt() -> [u8; SALT_LEN] {
    let mut bytes = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

fn hash_secret(secret: &str, salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(salt);
    hasher.update(secret.as_bytes());
    hasher.finalize().to_vec()
}

fn encode_hash(salt: &[u8], hash: &[u8]) -> String {
    let salt_b64 = STANDARD_NO_PAD.encode(salt);
    let hash_b64 = STANDARD_NO_PAD.encode(hash);
    format!("{}${}", salt_b64, hash_b64)
}

fn verify_secret(secret: &str, stored: &str) -> AuthResult<bool> {
    let (salt_b64, hash_b64) = stored
        .split_once('$')
        .ok_or_else(|| AuthError::TokenInvalid)?;
    let salt = STANDARD_NO_PAD
        .decode(salt_b64)
        .map_err(|_| AuthError::TokenInvalid)?;
    let expected = STANDARD_NO_PAD
        .decode(hash_b64)
        .map_err(|_| AuthError::TokenInvalid)?;
    let candidate = hash_secret(secret, &salt);
    Ok(constant_time_eq::constant_time_eq(&candidate, &expected))
}

mod constant_time_eq {
    /// Constant-time comparison to avoid timing side-channels.
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result: u8 = 0;
        for (&x, &y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }

        result == 0
    }
}
