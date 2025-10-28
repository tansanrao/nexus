//! Mailing list endpoints exposed under `/api/v1` and `/admin/v1`.

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{
    ApiResponse, MailingList, MailingListRepository, MailingListWithRepos, PaginationMeta,
    ResponseMeta, SortDescriptor, SortDirection,
};
use crate::sync::create_mailing_list_partitions;
use crate::sync::manifest::{fetch_manifest, parse_manifest};
use rocket::serde::json::Json;
use rocket::{State, get, patch, post};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};
use sqlx::Row;

const MAX_PAGE_SIZE: i64 = 100;

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    25
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, rocket::form::FromForm)]
#[serde(rename_all = "camelCase")]
pub struct ListQueryParams {
    #[field(default = 1)]
    #[serde(default = "default_page")]
    page: i64,
    #[field(name = "pageSize", default = 25)]
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i64,
    #[field(name = "sort")]
    #[serde(default)]
    sort: Vec<String>,
}

impl Default for ListQueryParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
            sort: Vec::new(),
        }
    }
}

impl ListQueryParams {
    fn page(&self) -> i64 {
        self.page.max(1)
    }

    fn page_size(&self) -> i64 {
        self.page_size.clamp(1, MAX_PAGE_SIZE)
    }

    fn sort(&self) -> &[String] {
        &self.sort
    }
}

fn parse_sort_fields(values: &[String]) -> (Vec<String>, Vec<SortDescriptor>) {
    let mut clauses = Vec::new();
    let mut descriptors = Vec::new();

    for value in values {
        let mut raw = value.splitn(2, ':');
        let field = raw.next().unwrap_or_default().trim();
        if field.is_empty() {
            continue;
        }
        let direction = raw.next().unwrap_or("asc").trim();
        let (column, api_field) = match field {
            "name" => ("name", "name"),
            "createdAt" => ("created_at", "createdAt"),
            "syncPriority" => ("sync_priority", "syncPriority"),
            "lastSyncedAt" => ("last_synced_at", "lastSyncedAt"),
            _ => continue,
        };

        let sort_dir = match direction.to_ascii_lowercase().as_str() {
            "desc" => SortDirection::Desc,
            _ => SortDirection::Asc,
        };

        let sql_dir = match sort_dir {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        clauses.push(format!("{column} {sql_dir}"));
        descriptors.push(SortDescriptor {
            field: api_field.to_string(),
            direction: sort_dir,
        });
    }

    if clauses.is_empty() {
        clauses.push("name ASC".to_string());
        descriptors.push(SortDescriptor {
            field: "name".to_string(),
            direction: SortDirection::Asc,
        });
    }

    (clauses, descriptors)
}

#[openapi(tag = "Lists")]
#[get("/lists?<params..>")]
pub async fn list_lists(
    mut db: Connection<NexusDb>,
    params: Option<ListQueryParams>,
) -> Result<Json<ApiResponse<Vec<MailingList>>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let (order_clauses, sort_meta) = parse_sort_fields(params.sort());
    let order_sql = order_clauses.join(", ");

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mailing_lists")
        .fetch_one(&mut **db)
        .await?;

    let query = format!(
        r#"
        SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
        FROM mailing_lists
        ORDER BY {order_sql}
        LIMIT $1 OFFSET $2
        "#
    );

    let lists = sqlx::query_as::<_, MailingList>(&query)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&mut **db)
        .await?;

    let meta = ResponseMeta::default()
        .with_pagination(PaginationMeta::new(page, page_size, total.0))
        .with_sort(sort_meta);

    Ok(Json(ApiResponse::with_meta(lists, meta)))
}

#[openapi(tag = "Lists")]
#[get("/lists/<slug>")]
pub async fn get_list(
    slug: String,
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<MailingList>>, ApiError> {
    let list: MailingList = sqlx::query_as(
        r#"
        SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
        FROM mailing_lists
        WHERE slug = $1
        "#,
    )
    .bind(&slug)
    .fetch_one(&mut **db)
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{slug}' not found")))?;

    let meta = ResponseMeta::default().with_list_id(slug);
    Ok(Json(ApiResponse::with_meta(list, meta)))
}

#[openapi(tag = "Admin - Lists")]
#[get("/lists?<params..>")]
pub async fn admin_list_lists(
    pool: &State<sqlx::PgPool>,
    params: Option<ListQueryParams>,
) -> Result<Json<ApiResponse<Vec<MailingList>>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let (order_clauses, sort_meta) = parse_sort_fields(params.sort());
    let order_sql = order_clauses.join(", ");

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mailing_lists")
        .fetch_one(pool.inner())
        .await?;

    let query = format!(
        r#"
        SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
        FROM mailing_lists
        ORDER BY {order_sql}
        LIMIT $1 OFFSET $2
        "#
    );

    let lists = sqlx::query_as::<_, MailingList>(&query)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool.inner())
        .await?;

    let meta = ResponseMeta::default()
        .with_pagination(PaginationMeta::new(page, page_size, total.0))
        .with_sort(sort_meta);

    Ok(Json(ApiResponse::with_meta(lists, meta)))
}

#[openapi(tag = "Admin - Lists")]
#[get("/lists/<slug>")]
pub async fn admin_get_list(
    slug: String,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<MailingList>>, ApiError> {
    let list: MailingList = sqlx::query_as(
        r#"
        SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
        FROM mailing_lists
        WHERE slug = $1
        "#,
    )
    .bind(&slug)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{slug}' not found")))?;

    let meta = ResponseMeta::default().with_list_id(slug);
    Ok(Json(ApiResponse::with_meta(list, meta)))
}

#[openapi(tag = "Admin - Lists")]
#[get("/lists/<slug>/repositories")]
pub async fn admin_get_list_with_repos(
    slug: String,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<MailingListWithRepos>>, ApiError> {
    let list: MailingList = sqlx::query_as(
        r#"
        SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
        FROM mailing_lists
        WHERE slug = $1
        "#,
    )
    .bind(&slug)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{slug}' not found")))?;

    let repos: Vec<MailingListRepository> = sqlx::query_as(
        r#"
        SELECT id, mailing_list_id, repo_url, repo_order, last_indexed_commit, created_at
        FROM mailing_list_repositories
        WHERE mailing_list_id = $1
        ORDER BY repo_order ASC
        "#,
    )
    .bind(list.id)
    .fetch_all(pool.inner())
    .await?;

    let meta = ResponseMeta::default().with_list_id(slug);
    Ok(Json(ApiResponse::with_meta(
        MailingListWithRepos { list, repos },
        meta,
    )))
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ToggleRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ToggleResponse {
    pub message: String,
    pub enabled: bool,
}

#[openapi(tag = "Admin - Lists")]
#[patch("/lists/<slug>/toggle", data = "<request>")]
pub async fn admin_toggle_list(
    slug: String,
    request: Json<ToggleRequest>,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<ToggleResponse>>, ApiError> {
    let enabled = request.enabled;
    let rows = sqlx::query(
        r#"
        UPDATE mailing_lists
        SET enabled = $1
        WHERE slug = $2
        "#,
    )
    .bind(enabled)
    .bind(&slug)
    .execute(pool.inner())
    .await?;

    if rows.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!(
            "Mailing list '{slug}' not found"
        )));
    }

    let response = ToggleResponse {
        message: format!(
            "Mailing list '{}' has been {}",
            slug,
            if enabled { "enabled" } else { "disabled" }
        ),
        enabled,
    };

    let meta = ResponseMeta::default().with_list_id(slug);
    Ok(Json(ApiResponse::with_meta(response, meta)))
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SeedResponse {
    pub message: String,
    #[serde(rename = "mailingListsCreated")]
    pub mailing_lists_created: usize,
    #[serde(rename = "repositoriesCreated")]
    pub repositories_created: usize,
    #[serde(rename = "partitionsCreated")]
    pub partitions_created: usize,
}

#[openapi(tag = "Admin - Lists")]
#[post("/lists/seed")]
pub async fn admin_seed_lists(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<SeedResponse>>, ApiError> {
    log::info!("seeding mailing lists from grokmirror manifest");

    let manifest = fetch_manifest()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch manifest: {e}")))?;

    let seed_data = parse_manifest(&manifest);
    let total_lists = seed_data.len();

    log::info!("parsed {} mailing lists from manifest", total_lists);

    let mut mailing_lists_created = 0;
    let mut repositories_created = 0;
    let mut partitions_created = 0;

    for ml_seed in seed_data {
        let result = sqlx::query(
            r#"
            INSERT INTO mailing_lists (name, slug, description, enabled, sync_priority)
            VALUES ($1, $2, $3, false, 0)
            ON CONFLICT (slug) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&ml_seed.name)
        .bind(&ml_seed.slug)
        .bind(&ml_seed.description)
        .fetch_optional(pool.inner())
        .await?;

        let ml_id: i32 = if let Some(row) = result {
            mailing_lists_created += 1;
            row.try_get("id")?
        } else {
            let row: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
                .bind(&ml_seed.slug)
                .fetch_one(pool.inner())
                .await?;
            row.0
        };

        for repo_shard in &ml_seed.repos {
            let result = sqlx::query(
                r#"
                INSERT INTO mailing_list_repositories
                (mailing_list_id, repo_url, repo_order, last_indexed_commit)
                VALUES ($1, $2, $3, NULL)
                ON CONFLICT (mailing_list_id, repo_order) DO NOTHING
                "#,
            )
            .bind(ml_id)
            .bind(&repo_shard.url)
            .bind(repo_shard.order)
            .execute(pool.inner())
            .await?;

            if result.rows_affected() > 0 {
                repositories_created += 1;
            }
        }

        let partition_exists: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM pg_tables WHERE tablename = 'emails_{}')",
            ml_seed.slug.replace('-', "_")
        ))
        .fetch_one(pool.inner())
        .await
        .unwrap_or(false);

        if !partition_exists {
            create_mailing_list_partitions(pool.inner(), ml_id, &ml_seed.slug).await?;
            partitions_created += 1;
        }
    }

    log::info!(
        "seed complete: {} lists, {} repos, {} partitions",
        mailing_lists_created,
        repositories_created,
        partitions_created
    );

    let response = SeedResponse {
        message: format!(
            "Seed completed: {} lists, {} repos, {} partitions",
            mailing_lists_created, repositories_created, partitions_created
        ),
        mailing_lists_created,
        repositories_created,
        partitions_created,
    };

    Ok(Json(ApiResponse::new(response)))
}

/// Test-only variant used by integration tests.
#[cfg(test)]
pub mod test_routes {
    use super::*;
    use rocket::State;

    #[get("/lists")]
    pub async fn list_lists_test(
        pool: &State<sqlx::PgPool>,
    ) -> Result<Json<ApiResponse<Vec<MailingList>>>, ApiError> {
        let params = ListQueryParams::default();
        let (order_clauses, sort_meta) = parse_sort_fields(params.sort());
        let order_sql = order_clauses.join(", ");

        let lists = sqlx::query_as::<_, MailingList>(&format!(
            r#"
            SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
            FROM mailing_lists
            ORDER BY {order_sql}
            "#
        ))
        .fetch_all(pool.inner())
        .await?;

        let meta = ResponseMeta::default().with_sort(sort_meta);
        Ok(Json(ApiResponse::with_meta(lists, meta)))
    }
}
