//! The real [`ClusterData`] implementation: composes whatever live backends a
//! deployment has configured — today the Slurm mirror in PostgreSQL, plus
//! Prometheus for (later) time-series graphs — behind the one trait the rest
//! of the app talks to. See [`crate::db`] for the Postgres client itself and
//! [`crate::metrics`] for the Prometheus client; this file only wires them
//! together to answer each screen's data need.
//!
//! Built once at server start from [`crate::config::Config`]. The catalog is
//! still static seed data — it becomes a real query when dashboards move into
//! the database.

use async_trait::async_trait;

use crate::config::{Config, CurrentJobsConfig};
use crate::data::{mock::seed_catalog, ClusterData, DataError};
use crate::db::Db;
use crate::metrics::Metrics;
use crate::models::{Catalog, JobPage, JobQuery};

pub struct LiveClusterData {
    db: Db,
    current_jobs_cfg: CurrentJobsConfig,
    catalog: Catalog,
    /// Present when a `[prometheus]` section was configured and reachable.
    /// Unused until the job-detail graphs are built.
    metrics: Option<Metrics>,
}

impl LiveClusterData {
    pub async fn connect(cfg: &Config) -> anyhow::Result<Self> {
        let db = Db::connect(&cfg.database).await?;

        let metrics = match &cfg.prometheus {
            Some(prom) => match Metrics::connect(prom) {
                Ok(m) => match m.health().await {
                    Ok(()) => {
                        leptos::logging::log!("metrics: prometheus reachable at {}", prom.endpoint);
                        Some(m)
                    }
                    Err(e) => {
                        leptos::logging::warn!("metrics: prometheus health check failed: {e}");
                        Some(m)
                    }
                },
                Err(e) => {
                    leptos::logging::warn!("metrics: prometheus client disabled: {e}");
                    None
                }
            },
            None => None,
        };

        Ok(Self {
            db,
            current_jobs_cfg: cfg.screens.current_jobs.clone(),
            catalog: seed_catalog(),
            metrics,
        })
    }

    pub fn metrics(&self) -> Option<&Metrics> {
        self.metrics.as_ref()
    }
}

#[async_trait]
impl ClusterData for LiveClusterData {
    async fn catalog(&self) -> Result<Catalog, DataError> {
        Ok(self.catalog.clone())
    }

    async fn current_jobs(&self, query: JobQuery) -> Result<JobPage, DataError> {
        self.db
            .current_jobs(&query, &self.current_jobs_cfg)
            .await
            .map_err(|e| {
                leptos::logging::error!("current_jobs query failed: {e:#}");
                DataError::backend(format!("{e:#}"))
            })
    }
}
