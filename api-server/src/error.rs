use crate::search::SearchError;
use chrono::Utc;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use rocket_okapi::OpenApiError;
use rocket_okapi::r#gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::response::OpenApiResponderInner;
use serde::Serialize;
use std::io::Cursor;

#[derive(Debug)]
pub enum ApiError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

/// RFC 7807-style problem details payload.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, title, detail, problem_type) = match self {
            ApiError::DatabaseError(e) => {
                log::error!("database error: {}", e);
                (
                    Status::InternalServerError,
                    "Internal Server Error",
                    "An internal database error occurred".to_string(),
                    "https://docs.nexus/errors/internal",
                )
            }
            ApiError::NotFound(msg) => {
                log::debug!("not found: {}", msg);
                (
                    Status::NotFound,
                    "Resource Not Found",
                    msg,
                    "https://docs.nexus/errors/not-found",
                )
            }
            ApiError::BadRequest(msg) => {
                log::debug!("bad request: {}", msg);
                (
                    Status::BadRequest,
                    "Bad Request",
                    msg,
                    "https://docs.nexus/errors/bad-request",
                )
            }
            ApiError::InternalError(msg) => {
                log::error!("internal error: {}", msg);
                (
                    Status::InternalServerError,
                    "Internal Server Error",
                    "An internal server error occurred".to_string(),
                    "https://docs.nexus/errors/internal",
                )
            }
        };

        let body = ProblemDetails {
            problem_type: problem_type.to_string(),
            title: title.to_string(),
            status: status.code,
            detail,
            instance: None,
            timestamp: Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        };

        let json = serde_json::to_string(&body).unwrap_or_else(|_| {
            r#"{"type":"about:blank","title":"Internal Server Error","status":500,"detail":"Failed to serialize error"}"#
                .to_string()
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

impl From<SearchError> for ApiError {
    fn from(err: SearchError) -> Self {
        log::error!("search error: {}", err);
        ApiError::InternalError("Search service error".to_string())
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
