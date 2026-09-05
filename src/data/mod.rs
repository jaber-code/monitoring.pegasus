//! Server-side data access.
//!
//! The browser never touches this module directly — it calls the `#[server]`
//! functions in [`crate::api`], which call [`provider()`] to get the active
//! [`ClusterData`] implementation.
//!
//! Backends:
//!   * [`mock::MockClusterData`] — in-memory, always available.
//!   * `live::LiveClusterData` — the real Slurm mirror in Postgres + (later) Prometheus.
//!
//! [`init_provider`] is called once at server start from `main`. If it is never
//! called (no `config.yaml`, or the DB probe failed) [`provider()`] falls back
//! to the mock, so the UI still renders.

mod error;

pub use error::DataError;

#[cfg(feature = "ssr")]
mod mock;

#[cfg(feature = "ssr")]
mod live;

#[cfg(feature = "ssr")]
pub use live::LiveClusterData;

#[cfg(feature = "ssr")]
mod imp {
    use std::sync::{Arc, OnceLock};

    use async_trait::async_trait;

    use super::{mock::MockClusterData, DataError};
    use crate::models::{Catalog, JobPage, JobQuery};

    /// Everything the UI can ask the cluster for.
    ///
    /// One method per screen's worth of data. Keep methods coarse-grained so a
    /// screen is one round-trip.
    #[async_trait]
    pub trait ClusterData: Send + Sync {
        /// Folders + dashboards for the home screen.
        async fn catalog(&self) -> Result<Catalog, DataError>;

        /// Rows for the Current Jobs screen, already filtered and capped.
        async fn current_jobs(&self, query: JobQuery) -> Result<JobPage, DataError>;
    }

    static PROVIDER: OnceLock<Arc<dyn ClusterData>> = OnceLock::new();

    /// Install the process-wide data source. First call wins; later calls are
    /// ignored (there is only ever one server).
    pub fn init_provider(provider: Arc<dyn ClusterData>) {
        let _ = PROVIDER.set(provider);
    }

    /// The active data source, defaulting to the mock if none was installed.
    pub fn provider() -> Arc<dyn ClusterData> {
        PROVIDER
            .get_or_init(|| Arc::new(MockClusterData::new()))
            .clone()
    }
}

#[cfg(feature = "ssr")]
pub use imp::{init_provider, provider, ClusterData};
