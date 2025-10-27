use api_server::auth::{AuthConfig, AuthState, JwtService, PasswordService, RefreshTokenStore};
use api_server::models::PatchType;
use api_server::routes::admin::refresh_search_index;
use api_server::test_support::{TestDatabase, TestDatabaseError, TestRocketBuilder};
use chrono::Utc;
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use rocket::routes;
use rocket::serde::json::json;
use std::fs;
use tempfile::NamedTempFile;

const TEST_RSA_PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEArELvhKIbrhh3t/lWS/jGyrv/6zsuOgy3xJZXIxSlIh9KDDYz
GPIYJPf607ylZQxlj9au5J7l7JRIa9sxCSvbMoh6x8/YHBNmFPyzkCq+DTTZH4Wk
EvpZrxnYl3+hskkGacdfD/dbmsaHEttPtPdNITlNISPrrzjxEkvi5vN0CWZnxgZs
WHLrs8qgct4bVX32asEGOcubqpvnDONbJdKp1AzZXewNaw98HoxY/sCATXCWGad4
ukONWZ9sCe0SG9xTPmepcNxR/dhpytRaCvy2xS4dcUJ59lp2rSHIUrFm4TRfxWo/
GdSEJxP2wm2yp5q2ggzA6VMBUuP28CE2ik9n7QIDAQABAoIBACnBovpRamjJ9RFD
T0Qktplzt34/rv2y0gQFFnPCQCI0l/g8VigMnUYu114mmygSuHbEyUnRa7Ysnp6I
eEs7FowaEbsoOoBZwnPBasx+U+nzHtOZi1NvXLiJiRt2PI2xTmzrP3OpGAs9ZwYu
49Qf41Izp+rp4Gpt4N/4xbSKnJzfUE9YwEpHbRj08Ur7dngXuddbLCdZjgNVCn//
qhCpNMSG5iBrvYQ1TDQkDVkVIHK2VWxCsvLhUMfu1SRUbIn7FMnxxh7j8uAqXma8
u7Vv3WvV50cMTnJB0rvhdaIg6O7Y5e8uiSS3tbakyFHrr2ow+TFKI6/CMc4e+r0C
wheZuBkCgYEA2xvOs6JgVg72UuX4w23/DYta+wNX0muuI8cA5W8SUIxTox83nCZI
O86QZGvHVvsmQ1T+VEHkDUPkQnVvukKjFpV7VLNdj/s+7Lt2pSRAFm8tkj+Ber4u
oYS2KGKfOuxH0CwA6BZCJbHt0kWPnWCKAYeUEqfd7yqSeStutY4vnfkCgYEAyUPi
milbUtrbVTnkyL/pRFA8kZuZnP0uMxdgFXsCox0EZ2zrZvXP2IHnKvJOtYhoE6E8
Itp7eP2Pu4LLdet6vQIHE3xUrKYBX770yyxFHWwJn1m1ZxGWrzeGUoSZJXRTEr8R
UzDS5ZayD9VrxehE5E156OkK6ksENk3v4OexppUCgYEA1dpdM8zPFA/EcYLN+wi4
AKM8KHTJ2bGJpJfOEyEGkiF0XGjSoRBoPh9NpQXg6M92OA+Tr+8jw6K4/fibFQOH
JDq/xhrOvgHuF6aclXA9MOhQZUagfIl0/+aE2APx/9Ov/8mDFQLsitgQE8Qa+PLJ
n9aROmgnYBCAJ82xX3iolxkCgYEAuqsr0K/q873pD/LSLx9PyvxgMOyQXPq1js1v
YHzmxUJ0gziSXLxAOh7BuSNjvRr27L3ueKULP/xtAw0ciBIPlJ380iXOoxKU06jY
glhdAhziD9m0VhQKHhjxjDdPk12AbzKnbvEpqadLH0Ri4Pu8acMx/sOmTAensHY4
tfAu5MECgYBESDe8c8mjig+ktC3P5K8FeR+pNGqp7hjCiRP2J+IPOQhQLYCu2RfU
5+f+Rbk7YIByHjrY4MpcaNvMnSQHFI49O/xBiSGzpkdnLfkZ4Q6Xd6St56qfgzhf
OmSlD5OcHBaImD0VICliqmth4eOzV1tsrnkUBA1DHRAM1Z2/Ausa2Q==
-----END RSA PRIVATE KEY-----"#;

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

    let _email_id: i32 = sqlx::query_scalar(
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

    let key_file = NamedTempFile::new().expect("failed to create temp key file");
    fs::write(key_file.path(), TEST_RSA_PRIVATE_KEY).expect("failed to write key");

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
        jwt_private_key_path: key_file.path().to_path_buf(),
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
        .mount_api_routes(routes![refresh_search_index])
        .build()
        .manage(auth_state);

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance with auth");

    let response = client
        .post("/api/v1/admin/search/index/refresh")
        .header(ContentType::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", admin_token.token),
        ))
        .body(json!({ "mailingListSlug": "lkml" }).to_string())
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    test_db.close().await.expect("failed to drop test database");
}
