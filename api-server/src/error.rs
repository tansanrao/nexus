use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::{Request, Response};
use serde::Serialize;
use std::io::Cursor;

#[derive(Debug)]
pub enum ApiError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, error_type, message) = match self {
            ApiError::DatabaseError(e) => {
                log::error!("database error: {}", e);
                (Status::InternalServerError, "DatabaseError", e.to_string())
            }
            ApiError::NotFound(msg) => {
                log::debug!("not found: {}", msg);
                (Status::NotFound, "NotFound", msg)
            }
            ApiError::BadRequest(msg) => {
                log::debug!("bad request: {}", msg);
                (Status::BadRequest, "BadRequest", msg)
            }
            ApiError::InternalError(msg) => {
                log::error!("internal error: {}", msg);
                (Status::InternalServerError, "InternalError", msg)
            }
        };

        let error_response = ErrorResponse {
            error: error_type.to_string(),
            message,
        };

        let json = serde_json::to_string(&error_response)
            .unwrap_or_else(|_| r#"{"error":"SerializationError","message":"Failed to serialize error"}"#.to_string());

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
