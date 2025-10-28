use api_server::auth::{AuthConfig, AuthState, JwtService, PasswordService, RefreshTokenStore};
use api_server::models::ApiResponse;
use api_server::routes::admin::{create_job, list_jobs};
use api_server::sync::queue::{JobRecord, JobType};
use api_server::test_support::{TestDatabase, TestDatabaseError, TestRocketBuilder};
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use rocket::routes;
use rocket::serde::json::json;

const TEST_JWT_SECRET: &str = "test-admin-secret";

#[tokio::test]
async fn create_index_maintenance_job_enqueues() {
    let test_db = match TestDatabase::new_from_env().await {
        Ok(db) => db,
        Err(TestDatabaseError::MissingUrl) => {
            eprintln!("skipping job test: TEST_DATABASE_URL not set");
            return;
        }
        Err(err) => panic!("failed to provision test database: {err:?}"),
    };

    let pool = test_db.pool_clone();

    let _mailing_list_id: i32 = sqlx::query_scalar(
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

    let config = AuthConfig {
        issuer: "https://nexus.test".into(),
        audience: "nexus-api".into(),
        access_token_ttl_secs: 900,
        refresh_token_ttl_secs: 7 * 24 * 60 * 60,
        session_cookie_ttl_secs: 30 * 60,
        refresh_cookie_name: "test_refresh_token".into(),
        csrf_cookie_name: "test_csrf".into(),
        csrf_header_name: "X-CSRF-Token".into(),
        session_cookie_name: "test_session".into(),
        cookie_domain: None,
        cookie_secure: false,
        jwt_secret: TEST_JWT_SECRET.into(),
        jwt_kid: Some("test-kid".into()),
    };
    let password_service = PasswordService::new().expect("password service");
    let jwt_service = JwtService::from_config(&config).expect("jwt service");
    let refresh_store = RefreshTokenStore::new(pool.clone());
    let auth_state = AuthState::new(config, password_service, jwt_service, refresh_store);

    let admin_id: i32 = sqlx::query_scalar(
        "INSERT INTO users (auth_provider, email, display_name, role) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind("local")
    .bind("admin@example.com")
    .bind::<Option<String>>(Some("Test Admin".to_string()))
    .bind("admin")
    .fetch_one(&pool)
    .await
    .expect("failed to insert admin user");

    let permissions = vec!["admin".to_string(), "user".to_string()];
    let admin_token = auth_state
        .jwt_service
        .clone()
        .issue_access_token(admin_id, "admin@example.com", "admin", &permissions, 0)
        .expect("issue admin token");

    let rocket = TestRocketBuilder::new()
        .manage_pg_pool(pool.clone())
        .mount_admin_routes(routes![create_job, list_jobs])
        .build()
        .manage(auth_state);

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance with auth");

    let payload = json!({
        "jobType": "index_maintenance",
        "payload": {"action": "refresh", "mailingListSlug": "lkml"},
        "priority": 5,
        "mailingListSlug": "lkml"
    });

    let response = client
        .post("/admin/v1/jobs")
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", admin_token.token),
        ))
        .body(payload.to_string())
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body: ApiResponse<JobRecord> = response.into_json().await.expect("JSON response");
    assert_eq!(body.data.job_type, JobType::IndexMaintenance);
    assert_eq!(body.data.mailing_list_slug.as_deref(), Some("lkml"));

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE job_type = 'index_maintenance'")
            .fetch_one(&pool)
            .await
            .expect("count jobs");
    assert_eq!(count.0, 1);
}
