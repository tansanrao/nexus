use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde::Serialize;
use std::io::Cursor;
use chrono::Utc;
use rocket_okapi::r#gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::OpenApiError;

#[derive(Debug)]
pub enum ApiError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

/// Individual error detail following REST best practices
#[derive(Serialize, JsonSchema)]
struct ErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
}

/// Standard error response format following REST best practices
#[derive(Serialize, JsonSchema)]
struct ErrorResponse {
    status: String,
    code: u16,
    timestamp: String,
    errors: Vec<ErrorDetail>,
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, error_type, message) = match self {
            ApiError::DatabaseError(e) => {
                log::error!("database error: {}", e);
                // Sanitize database errors - don't expose internal details
                (
                    Status::InternalServerError,
                    "DATABASE_ERROR",
                    "An internal database error occurred".to_string()
                )
            }
            ApiError::NotFound(msg) => {
                log::debug!("not found: {}", msg);
                (Status::NotFound, "NOT_FOUND", msg)
            }
            ApiError::BadRequest(msg) => {
                log::debug!("bad request: {}", msg);
                (Status::BadRequest, "BAD_REQUEST", msg)
            }
            ApiError::InternalError(msg) => {
                log::error!("internal error: {}", msg);
                // Sanitize internal errors - don't expose details
                (
                    Status::InternalServerError,
                    "INTERNAL_ERROR",
                    "An internal server error occurred".to_string()
                )
            }
        };

        let status_text = if status == Status::BadRequest {
            "BAD_REQUEST"
        } else if status == Status::NotFound {
            "NOT_FOUND"
        } else if status == Status::InternalServerError {
            "INTERNAL_SERVER_ERROR"
        } else {
            "ERROR"
        };

        let error_response = ErrorResponse {
            status: status_text.to_string(),
            code: status.code,
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            errors: vec![ErrorDetail {
                error_type: error_type.to_string(),
                message,
                field: None,
            }],
        };

        let json = serde_json::to_string(&error_response)
            .unwrap_or_else(|_| {
                r#"{"status":"INTERNAL_SERVER_ERROR","code":500,"timestamp":"","errors":[{"type":"SERIALIZATION_ERROR","message":"Failed to serialize error"}]}"#.to_string()
            });

        Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(json.len(), Cursor::new(json))
            .ok()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".to_string()),
            _ => ApiError::DatabaseError(err),
        }
    }
}

impl OpenApiResponderInner for ApiError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::*;

        Ok(Responses {
            responses: rocket_okapi::okapi::map! {
                "400".to_string() => RefOr::Object(Response {
                    description: "Bad Request - Invalid input parameters".to_string(),
                    ..Default::default()
                }),
                "404".to_string() => RefOr::Object(Response {
                    description: "Not Found - The requested resource was not found".to_string(),
                    ..Default::default()
                }),
                "500".to_string() => RefOr::Object(Response {
                    description: "Internal Server Error - An unexpected error occurred".to_string(),
                    ..Default::default()
                }),
            },
            ..Default::default()
        })
    }
}
