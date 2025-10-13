use rocket_db_pools::sqlx::{PgPool, Postgres, Transaction};

/// PostgreSQL configuration management for performance tuning
pub struct PgConfig;

impl PgConfig {
    /// Apply session-level optimizations for parallel queries
    pub async fn enable_parallel_queries(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::Error> {
        log::debug!("enabling parallel query optimizations");

        // Lower parallel setup cost to encourage parallelization
        sqlx::query("SET LOCAL parallel_setup_cost = 100")
            .execute(&mut **tx)
            .await?;

        sqlx::query("SET LOCAL parallel_tuple_cost = 0.01")
            .execute(&mut **tx)
            .await?;

        // Enable parallel operations
        sqlx::query("SET LOCAL enable_parallel_hash = on")
            .execute(&mut **tx)
            .await?;

        sqlx::query("SET LOCAL enable_partitionwise_aggregate = on")
            .execute(&mut **tx)
            .await?;

        // Increase work_mem for better sort/hash operations
        sqlx::query("SET LOCAL work_mem = '256MB'")
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    /// Apply all bulk sync optimizations at transaction level
    pub async fn apply_bulk_sync_optimizations(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::Error> {
        log::info!("applying bulk sync database optimizations");

        // Memory settings
        sqlx::query("SET LOCAL maintenance_work_mem = '2GB'")
            .execute(&mut **tx)
            .await?;

        sqlx::query("SET LOCAL work_mem = '256MB'")
            .execute(&mut **tx)
            .await?;

        // Disable synchronous commit for better performance
        // Data is still durable via WAL, just committed asynchronously
        sqlx::query("SET LOCAL synchronous_commit = 'off'")
            .execute(&mut **tx)
            .await?;

        // Enable parallel queries
        sqlx::query("SET LOCAL parallel_setup_cost = 100")
            .execute(&mut **tx)
            .await?;

        sqlx::query("SET LOCAL parallel_tuple_cost = 0.01")
            .execute(&mut **tx)
            .await?;

        sqlx::query("SET LOCAL enable_parallel_hash = on")
            .execute(&mut **tx)
            .await?;

        sqlx::query("SET LOCAL enable_partitionwise_aggregate = on")
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    /// Run VACUUM ANALYZE after bulk operations
    pub async fn vacuum_analyze(pool: &PgPool) -> Result<(), sqlx::Error> {
        log::info!("running VACUUM ANALYZE to update statistics");

        // VACUUM ANALYZE must run outside a transaction
        sqlx::query("VACUUM ANALYZE")
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Run VACUUM ANALYZE on specific tables
    pub async fn vacuum_analyze_tables(
        pool: &PgPool,
        tables: &[&str],
    ) -> Result<(), sqlx::Error> {
        for table in tables {
            log::debug!("running VACUUM ANALYZE on table: {}", table);
            let query = format!("VACUUM ANALYZE {}", table);
            sqlx::query(&query)
                .execute(pool)
                .await?;
        }

        Ok(())
    }

    /// Check current configuration settings
    pub async fn check_config(pool: &PgPool) -> Result<ConfigSnapshot, sqlx::Error> {
        let max_connections: (String,) = sqlx::query_as("SHOW max_connections")
            .fetch_one(pool)
            .await?;

        let shared_buffers: (String,) = sqlx::query_as("SHOW shared_buffers")
            .fetch_one(pool)
            .await?;

        let work_mem: (String,) = sqlx::query_as("SHOW work_mem")
            .fetch_one(pool)
            .await?;

        let maintenance_work_mem: (String,) = sqlx::query_as("SHOW maintenance_work_mem")
            .fetch_one(pool)
            .await?;

        let max_parallel_workers: (String,) = sqlx::query_as("SHOW max_parallel_workers")
            .fetch_one(pool)
            .await?;

        let max_worker_processes: (String,) = sqlx::query_as("SHOW max_worker_processes")
            .fetch_one(pool)
            .await?;

        Ok(ConfigSnapshot {
            max_connections: max_connections.0,
            shared_buffers: shared_buffers.0,
            work_mem: work_mem.0,
            maintenance_work_mem: maintenance_work_mem.0,
            max_parallel_workers: max_parallel_workers.0,
            max_worker_processes: max_worker_processes.0,
        })
    }
}

/// Snapshot of current PostgreSQL configuration
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub max_connections: String,
    pub shared_buffers: String,
    pub work_mem: String,
    pub maintenance_work_mem: String,
    pub max_parallel_workers: String,
    pub max_worker_processes: String,
}

impl std::fmt::Display for ConfigSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "PostgreSQL Configuration:")?;
        writeln!(f, "  max_connections: {}", self.max_connections)?;
        writeln!(f, "  shared_buffers: {}", self.shared_buffers)?;
        writeln!(f, "  work_mem: {}", self.work_mem)?;
        writeln!(f, "  maintenance_work_mem: {}", self.maintenance_work_mem)?;
        writeln!(f, "  max_parallel_workers: {}", self.max_parallel_workers)?;
        writeln!(f, "  max_worker_processes: {}", self.max_worker_processes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_snapshot_display() {
        let snapshot = ConfigSnapshot {
            max_connections: "100".to_string(),
            shared_buffers: "128MB".to_string(),
            work_mem: "4MB".to_string(),
            maintenance_work_mem: "64MB".to_string(),
            max_parallel_workers: "8".to_string(),
            max_worker_processes: "16".to_string(),
        };

        let display = format!("{}", snapshot);
        assert!(display.contains("max_connections: 100"));
        assert!(display.contains("shared_buffers: 128MB"));
    }
}
