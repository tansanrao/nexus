use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("nexus_db")]
pub struct NexusDb(sqlx::PgPool);

#[derive(Database)]
#[database("bulk_write_db")]
pub struct BulkWriteDb(sqlx::PgPool);
