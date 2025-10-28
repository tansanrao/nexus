//! Administrative endpoints for job orchestration and database management.

use crate::auth::RequireAdmin;
use crate::error::ApiError;
use crate::models::{ApiResponse, ResponseMeta};
use crate::sync::pg_config::PgConfig;
use crate::sync::queue::{JobQueue, JobRecord, JobStatus, JobType};
use crate::sync::reset_database;
use rocket::serde::json::Json;
use rocket::{State, delete, get, patch, post};
use rocket_db_pools::sqlx;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, rocket::form::FromForm)]
#[serde(rename_all = "camelCase")]
pub struct JobListParams {
    #[field(default = 1)]
    #[serde(default = "default_page")]
    page: i64,
    #[field(name = "pageSize", default = 25)]
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i64,
    #[field(name = "status")]
    #[serde(default)]
    status: Vec<String>,
    #[field(name = "type")]
    #[serde(default)]
    job_type: Vec<String>,
}

impl Default for JobListParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
            status: Vec::new(),
            job_type: Vec::new(),
        }
    }
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    25
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobRequest {
    #[serde(rename = "jobType")]
    pub job_type: JobType,
    #[serde(default)]
    pub payload: JsonValue,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(rename = "mailingListSlug")]
    pub mailing_list_slug: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJobRequest {
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DatabaseStatusResponse {
    #[serde(rename = "totalAuthors")]
    pub total_authors: i64,
    #[serde(rename = "totalEmails")]
    pub total_emails: i64,
    #[serde(rename = "totalThreads")]
    pub total_threads: i64,
    #[serde(rename = "totalRecipients")]
    pub total_recipients: i64,
    #[serde(rename = "totalReferences")]
    pub total_references: i64,
    #[serde(rename = "totalThreadMemberships")]
    pub total_thread_memberships: i64,
    #[serde(rename = "dateRangeStart")]
    pub date_range_start: Option<chrono::NaiveDateTime>,
    #[serde(rename = "dateRangeEnd")]
    pub date_range_end: Option<chrono::NaiveDateTime>,
}

#[openapi(tag = "Admin - Jobs")]
#[get("/jobs?<params..>")]
pub async fn list_jobs(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
    params: Option<JobListParams>,
) -> Result<Json<ApiResponse<Vec<JobRecord>>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page;
    let page_size = params.page_size;

    let statuses = parse_status_filters(&params.status)?;
    let types = parse_type_filters(&params.job_type)?;

    let queue = JobQueue::new(pool.inner().clone());
    let (jobs, total) = queue.list_jobs(&statuses, &types, page, page_size).await?;

    let mut meta = ResponseMeta::default()
        .with_pagination(crate::models::PaginationMeta::new(page, page_size, total));

    if !params.status.is_empty() || !params.job_type.is_empty() {
        let mut filters = JsonMap::new();
        if !params.status.is_empty() {
            filters.insert(
                "status".to_string(),
                JsonValue::Array(
                    params
                        .status
                        .iter()
                        .map(|s| JsonValue::String(s.clone()))
                        .collect(),
                ),
            );
        }
        if !params.job_type.is_empty() {
            filters.insert(
                "type".to_string(),
                JsonValue::Array(
                    params
                        .job_type
                        .iter()
                        .map(|s| JsonValue::String(s.clone()))
                        .collect(),
                ),
            );
        }
        meta = meta.with_filters(filters);
    }

    Ok(Json(ApiResponse::with_meta(jobs, meta)))
}

#[openapi(tag = "Admin - Jobs")]
#[post("/jobs", data = "<request>")]
pub async fn create_job(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
    request: Json<CreateJobRequest>,
) -> Result<Json<ApiResponse<JobRecord>>, ApiError> {
    let data = request.into_inner();
    let queue = JobQueue::new(pool.inner().clone());

    let mailing_list_id = if let Some(slug) = data.mailing_list_slug.as_ref() {
        Some(resolve_mailing_list_id_by_slug(pool.inner(), slug).await?)
    } else {
        None
    };

    let payload = if data.payload.is_null() {
        JsonValue::Object(JsonMap::new())
    } else {
        data.payload
    };

    let priority = data.priority.unwrap_or(0);
    let job_id = queue
        .enqueue_job(data.job_type, mailing_list_id, payload, priority)
        .await?;

    let job = queue
        .get_job(job_id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch newly created job".to_string()))?;

    Ok(Json(ApiResponse::new(job)))
}

#[openapi(tag = "Admin - Jobs")]
#[get("/jobs/<job_id>")]
pub async fn get_job(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
    job_id: i32,
) -> Result<Json<ApiResponse<JobRecord>>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let job = queue
        .get_job(job_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {job_id} not found")))?;

    Ok(Json(ApiResponse::new(job)))
}

#[openapi(tag = "Admin - Jobs")]
#[patch("/jobs/<job_id>", data = "<request>")]
pub async fn patch_job(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
    job_id: i32,
    request: Json<UpdateJobRequest>,
) -> Result<Json<ApiResponse<JobRecord>>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let data = request.into_inner();

    if let Some(action) = data.action.as_ref() {
        if action == "cancel" {
            let cancelled = queue.cancel_job(job_id).await?;
            if !cancelled {
                return Err(ApiError::BadRequest(
                    "Job cannot be cancelled in its current state".to_string(),
                ));
            }
        } else {
            return Err(ApiError::BadRequest(format!(
                "Unsupported action '{action}'"
            )));
        }
    }

    if let Some(priority) = data.priority {
        let updated = queue.update_priority(job_id, priority).await?;
        if !updated {
            return Err(ApiError::BadRequest(
                "Job priority update failed".to_string(),
            ));
        }
    }

    let job = queue
        .get_job(job_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {job_id} not found")))?;

    Ok(Json(ApiResponse::new(job)))
}

#[openapi(tag = "Admin - Jobs")]
#[delete("/jobs/<job_id>")]
pub async fn delete_job(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
    job_id: i32,
) -> Result<Json<ApiResponse<JsonValue>>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let removed = queue.delete_job(job_id).await?;
    if !removed {
        return Err(ApiError::BadRequest(
            "Job must be completed before deletion".to_string(),
        ));
    }

    Ok(Json(ApiResponse::new(JsonValue::Null)))
}

#[openapi(tag = "Admin - Database")]
#[post("/database/reset")]
pub async fn reset_database_endpoint(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<MessageResponse>>, ApiError> {
    reset_database(pool.inner())
        .await
        .map_err(|e| ApiError::InternalError(format!("Database reset failed: {e}")))?;

    Ok(Json(ApiResponse::new(MessageResponse {
        message: "Database reset successfully".to_string(),
    })))
}

#[openapi(tag = "Admin - Database")]
#[get("/database/status")]
pub async fn database_status(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<DatabaseStatusResponse>>, ApiError> {
    let (
        total_authors,
        total_emails,
        total_threads,
        total_recipients,
        total_references,
        total_thread_memberships,
        date_range,
    ) = tokio::try_join!(
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM authors")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM emails")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM threads")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM email_recipients")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM email_references")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM thread_memberships")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
                "SELECT MIN(date), MAX(date) FROM emails",
            )
            .fetch_one(pool.inner())
            .await
            .or_else(|_| Ok::<_, sqlx::Error>((None, None)))
        }
    )?;

    let response = DatabaseStatusResponse {
        total_authors: total_authors.0,
        total_emails: total_emails.0,
        total_threads: total_threads.0,
        total_recipients: total_recipients.0,
        total_references: total_references.0,
        total_thread_memberships: total_thread_memberships.0,
        date_range_start: date_range.0,
        date_range_end: date_range.1,
    };

    Ok(Json(ApiResponse::new(response)))
}

#[openapi(tag = "Admin - Database")]
#[get("/database/config")]
pub async fn database_config(
    _admin: RequireAdmin,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<JsonValue>>, ApiError> {
    let config = PgConfig::check_config(pool.inner())
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get config: {e}")))?;

    let json = serde_json::json!({
        "maxConnections": config.max_connections,
        "sharedBuffers": config.shared_buffers,
        "workMem": config.work_mem,
        "maintenanceWorkMem": config.maintenance_work_mem,
        "maxParallelWorkers": config.max_parallel_workers,
        "maxWorkerProcesses": config.max_worker_processes,
    });

    Ok(Json(ApiResponse::new(json)))
}

async fn resolve_mailing_list_id_by_slug(pool: &sqlx::PgPool, slug: &str) -> Result<i32, ApiError> {
    let row: Option<(i32,)> = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            ApiError::InternalError(format!("Failed to lookup mailing list '{slug}': {e}"))
        })?;

    row.map(|(id,)| id)
        .ok_or_else(|| ApiError::BadRequest(format!("Mailing list '{slug}' not found")))
}

fn parse_status_filters(values: &[String]) -> Result<Vec<JobStatus>, ApiError> {
    values
        .iter()
        .map(|value| match value.as_str() {
            "queued" => Ok(JobStatus::Queued),
            "running" => Ok(JobStatus::Running),
            "succeeded" => Ok(JobStatus::Succeeded),
            "failed" => Ok(JobStatus::Failed),
            "cancelled" => Ok(JobStatus::Cancelled),
            other => Err(ApiError::BadRequest(format!("Unknown status '{other}'"))),
        })
        .collect()
}

fn parse_type_filters(values: &[String]) -> Result<Vec<JobType>, ApiError> {
    values
        .iter()
        .map(|value| match value.as_str() {
            "import" => Ok(JobType::Import),
            "index_maintenance" => Ok(JobType::IndexMaintenance),
            other => Err(ApiError::BadRequest(format!("Unknown job type '{other}'"))),
        })
        .collect()
}
