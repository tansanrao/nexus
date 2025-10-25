//! Database migration management for the sync system.
//!
//! This module validates and applies SQLx migrations before the API starts serving
//! requests. We always ensure the database is on the latest schema and abort startup
//! when drift is detected.

use rocket_db_pools::sqlx::{self, PgPool, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Run database migrations.
///
/// This is idempotent - migrations that have already been applied will be skipped.
/// Uses SQLx's built-in migration system to track which migrations have been run.
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
/// `Ok(())` if migrations succeed, error otherwise
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    log::info!("checking database migration state");

    // `run` ensures the migrations table exists, verifies checksums, and applies
    // any pending migrations before we start serving traffic.
    MIGRATOR.run(pool).await?;

    log::info!("database migrations up to date");
    Ok(())
}

/// Reset database by dropping and recreating all tables.
///
/// **WARNING**: This will drop ALL data including all mailing list partitions.
/// Use with extreme caution - typically only for development/testing.
///
/// # Process
/// 1. Drops all tables in reverse dependency order
/// 2. Drops the SQLx migrations tracking table
/// 3. Re-runs all migrations from scratch
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
/// `Ok(())` if reset succeeds, error otherwise
pub async fn reset_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    log::info!("resetting database schema");

    // Drop the public schema entirely to ensure we remove every artifact from
    // prior migrations (tables, types, sequences, etc.). Recreate it with the
    // default grants so subsequent migrations run against a clean slate.
    sqlx::query("DROP SCHEMA IF EXISTS public CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("CREATE SCHEMA public").execute(pool).await?;

    sqlx::query("GRANT ALL ON SCHEMA public TO postgres")
        .execute(pool)
        .await?;

    sqlx::query("GRANT ALL ON SCHEMA public TO public")
        .execute(pool)
        .await?;

    sqlx::query("COMMENT ON SCHEMA public IS 'standard public schema'")
        .execute(pool)
        .await?;

    // Drop extension metadata so migrations can recreate them with the expected
    // options. Extensions live outside the schema, so we explicitly remove any
    // that the migrator manages.
    sqlx::query("DROP EXTENSION IF EXISTS vchord CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP EXTENSION IF EXISTS pg_trgm CASCADE")
        .execute(pool)
        .await?;

    log::info!("all schemas dropped, running migrations");

    // Run all migrations from scratch
    sqlx::migrate!("./migrations").run(pool).await?;

    log::info!("database schema created via migrations");
    log::info!("call /api/admin/mailing-lists/seed to populate lists");
    Ok(())
}
