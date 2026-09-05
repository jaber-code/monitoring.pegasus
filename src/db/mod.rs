//! PostgreSQL access for the Slurm mirror database (ssr only).
//!
//! One [`Db`] wraps a `deadpool` connection pool. Query logic lives in the
//! submodules ([`jobs`]); this module owns connection setup and a one-time
//! introspection of the `jobs` table so queries adapt to the columns that
//! actually exist (the live schema lags the collector's migrations).

mod jobs;

use std::collections::HashSet;

use deadpool_postgres::{Config as PoolConfig, PoolConfig as PoolSize, Runtime};
use tokio_postgres::NoTls;

use crate::config::DbConfig;

pub struct Db {
    pool: deadpool_postgres::Pool,
    /// The Current Jobs SQL, built once from the real column set.
    jobs_sql: String,
}

impl Db {
    /// Build the pool, verify the database is reachable, and compile the
    /// column-adaptive queries. Returns `Err` if the probe fails or the `jobs`
    /// table is missing.
    pub async fn connect(cfg: &DbConfig) -> anyhow::Result<Self> {
        let mut pool_cfg = PoolConfig::new();
        pool_cfg.host = Some(cfg.host.clone());
        pool_cfg.port = Some(cfg.port);
        pool_cfg.dbname = Some(cfg.dbname.clone());
        pool_cfg.user = Some(cfg.user.clone());
        pool_cfg.password = Some(cfg.password.clone());
        // Read-only, low-QPS UI: a handful of connections is plenty.
        pool_cfg.pool = Some(PoolSize::new(8));

        let pool = pool_cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

        let client = pool.get().await?;
        client.query_one("SELECT 1", &[]).await?;

        let columns: HashSet<String> = client
            .query(
                "SELECT lower(column_name) AS c
                   FROM information_schema.columns
                  WHERE table_schema = 'public' AND table_name = 'jobs'",
                &[],
            )
            .await?
            .iter()
            .map(|row| row.get::<_, String>("c"))
            .collect();

        anyhow::ensure!(
            columns.contains("jobid") && columns.contains("jobstate"),
            "table `jobs` not found or missing core columns (saw {} columns)",
            columns.len()
        );
        drop(client);

        let jobs_sql = jobs::build_current_jobs_sql(&columns);
        Ok(Self { pool, jobs_sql })
    }

    /// Check out a pooled connection.
    async fn client(&self) -> anyhow::Result<deadpool_postgres::Client> {
        Ok(self.pool.get().await?)
    }
}
