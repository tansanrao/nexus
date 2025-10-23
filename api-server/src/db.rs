use rocket_db_pools::{Database, sqlx};

#[derive(Database)]
#[database("nexus_db")]
pub struct NexusDb(sqlx::PgPool);
