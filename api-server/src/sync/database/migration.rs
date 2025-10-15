//! Database migration management for the sync system.
//!
//! This module handles running SQLx migrations to set up and update the database schema.
//! Migrations are idempotent - running them multiple times is safe.

use rocket_db_pools::sqlx::PgPool;

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
    log::info!("running database migrations");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    log::info!("database migrations completed");
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

    // Drop all existing tables in reverse order of dependencies
    // PostgreSQL CASCADE will handle partitions
    sqlx::query("DROP TABLE IF EXISTS thread_memberships CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS threads CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS email_recipients CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS email_references CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS emails CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS author_mailing_list_activity CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS author_name_aliases CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS authors CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS mailing_list_repositories CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS mailing_lists CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS sync_jobs CASCADE")
        .execute(pool)
        .await?;

    // Drop the sqlx migrations tracking table to allow re-running migrations
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations CASCADE")
        .execute(pool)
        .await?;

    log::info!("all tables dropped, running migrations");

    // Run all migrations from scratch
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    log::info!("database schema created via migrations");
    log::info!("call /api/admin/mailing-lists/seed to populate lists");
    Ok(())
}
