use api_server::models::PatchType;
use api_server::routes::admin::refresh_search_index;
use api_server::test_support::{TestDatabase, TestDatabaseError, TestRocketBuilder};
use chrono::Utc;
use rocket::http::{ContentType, Status};
use rocket::routes;
use rocket::serde::json::json;

#[tokio::test]
async fn refresh_search_index_backfills_fts_columns() {
    let test_db = match TestDatabase::new_from_env().await {
        Ok(db) => db,
        Err(TestDatabaseError::MissingUrl) => {
            eprintln!("skipping search index test: TEST_DATABASE_URL not set");
            return;
        }
        Err(err) => panic!("failed to provision test database: {err:?}"),
    };

    let pool = test_db.pool_clone();

    let mailing_list_id: i32 = sqlx::query_scalar(
        "INSERT INTO mailing_lists (name, slug, description, enabled, sync_priority) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind("Linux Kernel Mailing List")
    .bind("lkml")
    .bind::<Option<String>>(Some("Linux kernel development".to_string()))
    .bind(true)
    .bind(0)
    .fetch_one(&pool)
    .await
    .expect("failed to insert mailing list");

    let author_id: i32 = sqlx::query_scalar(
        "INSERT INTO authors (email, canonical_name) VALUES ($1, $2) RETURNING id",
    )
    .bind("author@example.com")
    .bind::<Option<String>>(Some("Author".to_string()))
    .fetch_one(&pool)
    .await
    .expect("failed to insert author");

    let email_id: i32 = sqlx::query_scalar(
        "INSERT INTO emails (
            mailing_list_id, message_id, git_commit_hash, author_id,
            subject, normalized_subject, date, in_reply_to, body,
            series_id, series_number, series_total, epoch,
            patch_type, is_patch_only, patch_metadata, lex_ts, body_ts
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8, $9,
            $10, $11, $12, $13,
            $14, $15, $16, NULL, NULL
        ) RETURNING id",
    )
    .bind(mailing_list_id)
    .bind("<message@id>")
    .bind("abc123")
    .bind(author_id)
    .bind("[PATCH v1] Test email")
    .bind("patch v1 test email")
    .bind(Utc::now())
    .bind::<Option<String>>(None)
    .bind("This is the email body for testing.")
    .bind::<Option<String>>(None)
    .bind::<Option<i32>>(None)
    .bind::<Option<i32>>(None)
    .bind(0)
    .bind(PatchType::None)
    .bind(false)
    .bind::<Option<serde_json::Value>>(None)
    .fetch_one(&pool)
    .await
    .expect("failed to insert email");

    let client = TestRocketBuilder::new()
        .manage_pg_pool(pool.clone())
        .mount_api_routes(routes![refresh_search_index])
        .async_client()
        .await;

    let response = client
        .post("/api/v1/admin/search/index/refresh")
        .header(ContentType::JSON)
        .body(json!({ "mailingListSlug": "lkml" }).to_string())
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let (lex_present, body_present): (bool, bool) = sqlx::query_as(
        "SELECT lex_ts IS NOT NULL, body_ts IS NOT NULL FROM emails WHERE mailing_list_id = $1 AND id = $2",
    )
    .bind(mailing_list_id)
    .bind(email_id)
    .fetch_one(&pool)
    .await
    .expect("failed to fetch updated email");

    assert!(lex_present, "lexical tsvector should be populated");
    assert!(body_present, "body tsvector should be populated");

    test_db.close().await.expect("failed to drop test database");
}
