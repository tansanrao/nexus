//! Partition management for mailing list tables.
//!
//! PostgreSQL partitioning is used to separate data for each mailing list.
//! Each mailing list gets its own set of partitions for emails, threads, and related tables.
//!
//! # Index Management
//! Indexes are NOT created here - they are automatically inherited from the parent
//! partitioned tables. This follows PostgreSQL best practices for partitioned tables.

use rocket_db_pools::sqlx::PgPool;

/// Create all partitions for a specific mailing list.
///
/// Creates partitions for:
/// - emails
/// - threads
/// - email_recipients
/// - email_references
/// - thread_memberships
///
/// # Naming Convention
/// Table names are formatted as `{table}_{slug}` where hyphens in slug are
/// replaced with underscores for PostgreSQL compatibility.
///
/// # Index Inheritance
/// Indexes are NOT created here - they are automatically created by PostgreSQL
/// when you create indexes on the parent partitioned table. This follows PostgreSQL
/// best practices for partitioned tables.
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
/// * `list_id` - Mailing list ID (partition key value)
/// * `slug` - Mailing list slug (used in partition table names)
///
/// # Returns
/// `Ok(())` if all partitions are created successfully, error otherwise
pub async fn create_mailing_list_partitions(pool: &PgPool, list_id: i32, slug: &str) -> Result<(), sqlx::Error> {
    log::debug!("creating partitions: {} (id={})", slug, list_id);

    // Sanitize slug for use in table names (replace hyphens with underscores)
    let safe_slug = slug.replace('-', "_");

    // Authors table is now global (not partitioned) - skip partition creation

    // Create emails partition
    sqlx::query(&format!(
        r#"CREATE TABLE emails_{} PARTITION OF emails
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create threads partition
    sqlx::query(&format!(
        r#"CREATE TABLE threads_{} PARTITION OF threads
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create email_recipients partition
    sqlx::query(&format!(
        r#"CREATE TABLE email_recipients_{} PARTITION OF email_recipients
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create email_references partition
    sqlx::query(&format!(
        r#"CREATE TABLE email_references_{} PARTITION OF email_references
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create thread_memberships partition
    sqlx::query(&format!(
        r#"CREATE TABLE thread_memberships_{} PARTITION OF thread_memberships
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    log::debug!("partitions created: {}", slug);
    Ok(())
}

/// Drop all partitions for a specific mailing list.
///
/// Drops partition tables in reverse order of dependencies to avoid
/// foreign key constraint violations.
///
/// **WARNING**: This permanently deletes all data for the mailing list.
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
/// * `slug` - Mailing list slug (used in partition table names)
///
/// # Returns
/// `Ok(())` if all partitions are dropped successfully, error otherwise
#[allow(dead_code)]
pub async fn drop_mailing_list_partitions(pool: &PgPool, slug: &str) -> Result<(), sqlx::Error> {
    log::debug!("dropping partitions: {}", slug);
    let safe_slug = slug.replace('-', "_");

    // Drop in reverse order of dependencies
    sqlx::query(&format!("DROP TABLE IF EXISTS thread_memberships_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS email_references_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS email_recipients_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS threads_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS emails_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS authors_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;

    log::debug!("partitions dropped: {}", slug);
    Ok(())
}
