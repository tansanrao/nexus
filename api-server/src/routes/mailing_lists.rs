use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{MailingList, MailingListRepository, MailingListWithRepos, DataResponse};
use crate::sync::create_mailing_list_partitions;
use crate::sync::manifest::{fetch_manifest, parse_manifest};
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::{sqlx, Connection};
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use rocket_okapi::okapi::schemars::JsonSchema;

/// Get all mailing lists
#[openapi(tag = "Mailing Lists")]
#[get("/admin/mailing-lists")]
pub async fn list_mailing_lists(
    mut db: Connection<NexusDb>,
) -> Result<Json<DataResponse<Vec<MailingList>>>, ApiError> {
    let lists: Vec<MailingList> = sqlx::query_as(
        r#"SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
           FROM mailing_lists
           ORDER BY sync_priority ASC, name ASC"#
    )
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(DataResponse { data: lists }))
}

/// Get a specific mailing list by slug
#[openapi(tag = "Mailing Lists")]
#[get("/admin/mailing-lists/<slug>")]
pub async fn get_mailing_list(
    slug: String,
    mut db: Connection<NexusDb>,
) -> Result<Json<MailingList>, ApiError> {
    let list: MailingList = sqlx::query_as(
        r#"SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
           FROM mailing_lists
           WHERE slug = $1"#
    )
    .bind(&slug)
    .fetch_one(&mut **db)
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;

    Ok(Json(list))
}

/// Get a mailing list with its repositories
#[openapi(tag = "Mailing Lists")]
#[get("/admin/mailing-lists/<slug>/repositories")]
pub async fn get_mailing_list_with_repos(
    slug: String,
    mut db: Connection<NexusDb>,
) -> Result<Json<MailingListWithRepos>, ApiError> {
    // Get the mailing list
    let list: MailingList = sqlx::query_as(
        r#"SELECT id, name, slug, description, enabled, sync_priority, created_at, last_synced_at
           FROM mailing_lists
           WHERE slug = $1"#
    )
    .bind(&slug)
    .fetch_one(&mut **db)
    .await
    .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;

    // Get repositories for this mailing list
    let repos: Vec<MailingListRepository> = sqlx::query_as(
        r#"SELECT id, mailing_list_id, repo_url, repo_order, last_indexed_commit, created_at
           FROM mailing_list_repositories
           WHERE mailing_list_id = $1
           ORDER BY repo_order ASC"#
    )
    .bind(list.id)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(MailingListWithRepos {
        list,
        repos,
    }))
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ToggleRequest {
    enabled: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ToggleResponse {
    message: String,
    enabled: bool,
}

/// Toggle a mailing list enabled/disabled status
#[openapi(tag = "Mailing Lists")]
#[patch("/admin/mailing-lists/<slug>/toggle", data = "<request>")]
pub async fn toggle_mailing_list(
    slug: String,
    request: Json<ToggleRequest>,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ToggleResponse>, ApiError> {
    // Update the enabled status
    sqlx::query(
        r#"UPDATE mailing_lists
           SET enabled = $1
           WHERE slug = $2"#
    )
    .bind(request.enabled)
    .bind(&slug)
    .execute(pool.inner())
    .await?;

    Ok(Json(ToggleResponse {
        message: format!(
            "Mailing list '{}' has been {}",
            slug,
            if request.enabled { "enabled" } else { "disabled" }
        ),
        enabled: request.enabled,
    }))
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SeedResponse {
    message: String,
    #[serde(rename = "mailingListsCreated")]
    mailing_lists_created: usize,
    #[serde(rename = "repositoriesCreated")]
    repositories_created: usize,
    #[serde(rename = "partitionsCreated")]
    partitions_created: usize,
}

/// Seed all mailing lists from lore.kernel.org manifest
/// This endpoint is idempotent - safe to run multiple times
/// Dynamically fetches and parses the grokmirror manifest
#[openapi(tag = "Mailing Lists")]
#[post("/admin/mailing-lists/seed")]
pub async fn seed_mailing_lists(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<SeedResponse>, ApiError> {
    log::info!("seeding mailing lists from grokmirror manifest");

    // Fetch and parse the manifest dynamically
    let manifest = fetch_manifest()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch manifest: {}", e)))?;

    let seed_data = parse_manifest(&manifest);
    let total_lists = seed_data.len();

    log::info!("parsed {} mailing lists from manifest", total_lists);

    let mut mailing_lists_created = 0;
    let mut repositories_created = 0;
    let mut partitions_created = 0;

    for ml_seed in seed_data {
        // Insert mailing list (idempotent with ON CONFLICT DO NOTHING)
        let result = sqlx::query(
            r#"INSERT INTO mailing_lists (name, slug, description, enabled, sync_priority)
               VALUES ($1, $2, $3, false, 0)
               ON CONFLICT (slug) DO NOTHING
               RETURNING id"#
        )
        .bind(&ml_seed.name)
        .bind(&ml_seed.slug)
        .bind(&ml_seed.description)
        .fetch_optional(pool.inner())
        .await?;

        // Get the mailing list ID (either newly inserted or existing)
        let ml_id: i32 = if let Some(row) = result {
            mailing_lists_created += 1;
            row.try_get("id")?
        } else {
            // Already exists, fetch the ID
            let row: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
                .bind(&ml_seed.slug)
                .fetch_one(pool.inner())
                .await?;
            row.0
        };

        // Insert repositories for this mailing list
        for repo_shard in &ml_seed.repos {
            let result = sqlx::query(
                r#"INSERT INTO mailing_list_repositories
                   (mailing_list_id, repo_url, repo_order, last_indexed_commit)
                   VALUES ($1, $2, $3, NULL)
                   ON CONFLICT (mailing_list_id, repo_order) DO NOTHING"#
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

        // Check if partitions need to be created (only if this is a new mailing list)
        let partition_exists: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM pg_tables WHERE tablename = 'emails_{}')",
            ml_seed.slug.replace('-', "_")
        ))
        .fetch_one(pool.inner())
        .await
        .unwrap_or(false);

        if !partition_exists {
            // Create partitions for this mailing list
            create_mailing_list_partitions(pool.inner(), ml_id, &ml_seed.slug).await?;
            partitions_created += 1;
        }
    }

    log::info!(
        "seed complete: {} lists, {} repos, {} partitions",
        mailing_lists_created, repositories_created, partitions_created
    );

    Ok(Json(SeedResponse {
        message: format!(
            "Seeded {} mailing lists from lore.kernel.org manifest",
            total_lists
        ),
        mailing_lists_created,
        repositories_created,
        partitions_created,
    }))
}
