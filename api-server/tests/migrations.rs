use api_server::test_support::{TestDatabase, TestDatabaseError};
use sqlx::migrate::Migrator;

static TEST_MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::test]
async fn migrations_apply_and_revert_cleanly() {
    let test_db = match TestDatabase::new_from_env().await {
        Ok(db) => db,
        Err(TestDatabaseError::MissingUrl) => {
            eprintln!("skipping migration revert test: TEST_DATABASE_URL not set");
            return;
        }
        Err(err) => panic!("failed to provision test database: {err:?}"),
    };

    let pool = test_db.pool_clone();

    TEST_MIGRATOR.run(&pool).await.expect("migrations run");

    TEST_MIGRATOR
        .undo(&pool, 0)
        .await
        .expect("migrations revert");

    let mailing_list_tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'mailing_lists'",
    )
    .fetch_one(&pool)
    .await
    .expect("lookup succeeded");

    assert_eq!(
        mailing_list_tables, 0,
        "mailing_lists should be dropped after revert"
    );

    TEST_MIGRATOR.run(&pool).await.expect("migrations rerun");

    let mailing_list_tables_after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'mailing_lists'",
    )
    .fetch_one(&pool)
    .await
    .expect("lookup succeeded");

    assert_eq!(mailing_list_tables_after, 1);

    test_db.close().await.expect("failed to drop test database");
}
