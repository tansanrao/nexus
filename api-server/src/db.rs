use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("linux_kb_db")]
pub struct LinuxKbDb(sqlx::PgPool);
