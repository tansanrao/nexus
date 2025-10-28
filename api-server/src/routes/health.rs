//! Lightweight service health endpoint used for readiness checks and tests.

use rocket::serde::json::Json;
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::ApiResponse;

/// Basic response payload describing API health.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HealthResponse {
    /// Static status string reporting application readiness.
    pub status: String,
}

#[openapi(tag = "Health")]
#[get("/health/live")]
pub fn live_health() -> Json<ApiResponse<HealthResponse>> {
    Json(ApiResponse::new(HealthResponse {
        status: "ok".to_string(),
    }))
}

#[openapi(tag = "Health")]
#[get("/health/ready")]
pub async fn ready_health(
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<HealthResponse>>, ApiError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&mut **db)
        .await
        .map_err(|err| ApiError::InternalError(format!("readiness check failed: {err}")))?;

    Ok(Json(ApiResponse::new(HealthResponse {
        status: "ok".to_string(),
    })))
}
