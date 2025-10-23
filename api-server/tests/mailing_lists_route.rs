use api_server::models::{DataResponse, MailingList};
use api_server::routes::mailing_lists::test_routes;
use api_server::test_support::{TestDatabase, TestDatabaseError, TestRocketBuilder};
use rocket::http::Status;
use rocket::routes;

#[tokio::test]
async fn list_mailing_lists_returns_seed_data() {
    let test_db = match TestDatabase::new_from_env().await {
        Ok(db) => db,
        Err(TestDatabaseError::MissingUrl) => {
            eprintln!("skipping mailing list integration test: TEST_DATABASE_URL not set");
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
    .bind(Some("Linux kernel development".to_string()))
    .bind(true)
    .bind(0)
    .fetch_one(&pool)
    .await
    .expect("failed to insert mailing list");

    sqlx::query(
        "INSERT INTO mailing_list_repositories (mailing_list_id, repo_url, repo_order, last_indexed_commit) VALUES ($1, $2, $3, $4)",
    )
    .bind(mailing_list_id)
    .bind("https://lore.kernel.org/lkml/git/0.git")
    .bind(0)
    .bind::<Option<String>>(None)
    .execute(&pool)
    .await
    .expect("failed to insert mailing list repository");

    let client = TestRocketBuilder::new()
        .manage_pg_pool(pool.clone())
        .mount_api_routes(routes![test_routes::list_mailing_lists_test])
        .async_client()
        .await;

    let response = client.get("/api/v1/admin/mailing-lists").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let payload: DataResponse<Vec<MailingList>> = response
        .into_json()
        .await
        .expect("payload should deserialize");

    assert_eq!(payload.data.len(), 1);
    let list = &payload.data[0];
    assert_eq!(list.slug, "lkml");
    assert_eq!(list.name, "Linux Kernel Mailing List");
    assert_eq!(
        list.description.as_deref(),
        Some("Linux kernel development")
    );

    drop(client);

    test_db.close().await.expect("failed to drop test database");
}
