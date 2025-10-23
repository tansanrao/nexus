//! Lightweight service health endpoint used for readiness checks and tests.

use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};

/// Basic response payload describing API health.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HealthResponse {
    /// Static status string reporting application readiness.
    pub status: String,
}

/// Health check endpoint returning a trivial JSON payload.
#[openapi(tag = "Health")]
#[get("/health")]
pub fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}
