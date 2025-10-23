# Testing Database Guide

## Ephemeral Database Helper
- `src/lib.rs` exposes `api_server::test_support::TestDatabase` for integration tests.
- `TestDatabase::new_from_env()` and `TestDatabase::new()` automatically launch a disposable PostgreSQL container via `testcontainers`, clone a fresh database, run migrations, and return a dedicated `PgPool`.
- Databases are dropped automatically on `drop`, but call `test_db.close().await` when possible to surface errors early.
- Rocket tests can inject the pool via `TestRocketBuilder::manage_pg_pool(test_db.pool_clone())`.

### Example
```rust
let test_db = TestDatabase::new_from_env().await.expect("db");
let pool = test_db.pool_clone();

let client = TestRocketBuilder::new()
    .manage_pg_pool(pool.clone())
    .mount_api_routes(routes![...])
    .async_client()
    .await;

// ... run requests ...

drop(client);
test_db.close().await.expect("cleanup");
```

## Base Template Configuration (`TEST_DATABASE_URL`)
- Point `TEST_DATABASE_URL` at a writable **template** database (e.g. `postgres://postgres:postgres@127.0.0.1:6543/nexus_template`).
- The helper issues `CREATE DATABASE … TEMPLATE template0`, so the template only needs schemas/extensions you want cloned.
- Each test receives a unique database name (UUID suffix). `DROP DATABASE … WITH (FORCE)` ensures cleanup even if connections linger.

## Future Enhancements
- CI optimization: reuse a single container per workflow run and rely on `TestDatabase` to create per-test databases. This keeps setup cost low while isolating state.
