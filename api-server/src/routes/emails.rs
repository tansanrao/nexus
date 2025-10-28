//! Email endpoints scoped to mailing lists.

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{
    ApiResponse, EmailWithAuthor, PaginationMeta, ResponseMeta, SortDescriptor, SortDirection,
};
use crate::routes::helpers::resolve_mailing_list_id;
use rocket::get;
use rocket::serde::json::Json;
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};

const MAX_PAGE_SIZE: i64 = 100;

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, rocket::form::FromForm)]
#[serde(rename_all = "camelCase")]
pub struct EmailListParams {
    #[field(default = 1)]
    #[serde(default = "default_page")]
    page: i64,
    #[field(name = "pageSize", default = 50)]
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i64,
    #[field(name = "sort")]
    #[serde(default)]
    sort: Vec<String>,
}

impl Default for EmailListParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
            sort: vec!["date:desc".to_string()],
        }
    }
}

impl EmailListParams {
    fn page(&self) -> i64 {
        self.page.max(1)
    }

    fn page_size(&self) -> i64 {
        self.page_size.clamp(1, MAX_PAGE_SIZE)
    }

    fn normalized_sort(&self) -> Vec<String> {
        if self.sort.is_empty() {
            vec!["date:desc".to_string()]
        } else {
            self.sort.clone()
        }
    }
}

fn parse_email_sorts(values: &[String]) -> (Vec<String>, Vec<SortDescriptor>) {
    let mut clauses = Vec::new();
    let mut descriptors = Vec::new();

    for value in values {
        let mut parts = value.splitn(2, ':');
        let field = parts.next().unwrap_or_default().trim();
        if field.is_empty() {
            continue;
        }
        let direction = parts.next().unwrap_or("desc").trim();
        let (column, api_field) = match field {
            "date" => ("date", "date"),
            "subject" => ("subject", "subject"),
            "createdAt" => ("created_at", "createdAt"),
            _ => continue,
        };

        let dir = if direction.eq_ignore_ascii_case("asc") {
            SortDirection::Asc
        } else {
            SortDirection::Desc
        };

        let sql_dir = match dir {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        clauses.push(format!("{column} {sql_dir}"));
        descriptors.push(SortDescriptor {
            field: api_field.to_string(),
            direction: dir,
        });
    }

    if clauses.is_empty() {
        clauses.push("date DESC".to_string());
        descriptors.push(SortDescriptor {
            field: "date".to_string(),
            direction: SortDirection::Desc,
        });
    }

    (clauses, descriptors)
}

#[openapi(tag = "Emails")]
#[get("/lists/<slug>/emails?<params..>")]
pub async fn list_emails(
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<EmailListParams>,
) -> Result<Json<ApiResponse<Vec<EmailWithAuthor>>>, ApiError> {
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;
    let params = params.unwrap_or_default();
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let sort_values = params.normalized_sort();
    let (order_clauses, sort_meta) = parse_email_sorts(&sort_values);
    let order_sql = order_clauses.join(", ");

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE mailing_list_id = $1")
        .bind(mailing_list_id)
        .fetch_one(&mut **db)
        .await?;

    let query = format!(
        r#"
        SELECT
            e.id, e.mailing_list_id, e.message_id, e.git_commit_hash, e.author_id,
            e.subject, e.date, e.in_reply_to, e.body, e.created_at,
            a.canonical_name AS author_name, a.email AS author_email,
            e.patch_type, e.is_patch_only, e.patch_metadata
        FROM emails e
        JOIN authors a ON e.author_id = a.id
        WHERE e.mailing_list_id = $1
        ORDER BY {order_sql}
        LIMIT $2 OFFSET $3
        "#
    );

    let emails = sqlx::query_as::<_, EmailWithAuthor>(&query)
        .bind(mailing_list_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&mut **db)
        .await?;

    let meta = ResponseMeta::default()
        .with_list_id(slug)
        .with_sort(sort_meta)
        .with_pagination(PaginationMeta::new(page, page_size, total.0));

    Ok(Json(ApiResponse::with_meta(emails, meta)))
}

#[openapi(tag = "Emails")]
#[get("/lists/<slug>/emails/<email_id>")]
pub async fn get_email(
    slug: String,
    mut db: Connection<NexusDb>,
    email_id: i32,
) -> Result<Json<ApiResponse<EmailWithAuthor>>, ApiError> {
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let email = sqlx::query_as::<_, EmailWithAuthor>(
        r#"
        SELECT
            e.id, e.mailing_list_id, e.message_id, e.git_commit_hash, e.author_id,
            e.subject, e.date, e.in_reply_to, e.body, e.created_at,
            a.canonical_name AS author_name, a.email AS author_email,
            e.patch_type, e.is_patch_only, e.patch_metadata
        FROM emails e
        JOIN authors a ON e.author_id = a.id
        WHERE e.mailing_list_id = $1 AND e.id = $2
        "#,
    )
    .bind(mailing_list_id)
    .bind(email_id)
    .fetch_one(&mut **db)
    .await?;

    let meta = ResponseMeta::default().with_list_id(slug);
    Ok(Json(ApiResponse::with_meta(email, meta)))
}
