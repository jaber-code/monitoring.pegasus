//! Runtime configuration, loaded once at server start from `config.yaml`.
//!
//! Absent file ⇒ [`Config::load`] returns `None` and the server falls back to
//! the in-memory mock data source, so local dev needs no database.
//!
//! Path resolution: `$MONITORING_CONFIG` if set, else `./config.yaml`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DbConfig,
    #[serde(default)]
    pub prometheus: Option<PrometheusConfig>,
    /// Per-screen defaults and guardrails.
    #[serde(default)]
    pub screens: ScreensConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub host: String,
    #[serde(default = "default_pg_port")]
    pub port: u16,
    pub dbname: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrometheusConfig {
    pub endpoint: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    /// Verify the server's TLS certificate. `false` is dev-only.
    #[serde(default = "default_true")]
    pub verify: bool,
    /// PEM CA bundle for a private CA, relative to the config file's directory.
    #[serde(default)]
    pub ca_cert_path: Option<String>,
    #[serde(default = "default_prom_timeout")]
    pub timeout_secs: u64,
}

/// Config for the individual screens. Each screen owns a named sub-section so
/// tuning one never touches another. Grows as screens are added
/// (`job_archive`, `job_statistics`, …).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScreensConfig {
    #[serde(default)]
    pub current_jobs: CurrentJobsConfig,
}

/// The Current Jobs screen: what a fresh page load shows, plus hard limits on
/// query cost. The UI filter bar and URL query params override the per-view
/// choices; the server always clamps to the limits here.
#[derive(Debug, Clone, Deserialize)]
pub struct CurrentJobsConfig {
    /// Slurm states shown on a fresh load, before the user filters.
    #[serde(default = "default_states")]
    pub default_states: Vec<String>,
    /// Include `archived` rows. Off unless a request opts in *and* this is set.
    #[serde(default)]
    pub include_archived: bool,
    /// Finished jobs are only returned if they ended within this many hours.
    #[serde(default = "default_window_hours")]
    pub finished_window_hours: i64,
    /// Hard cap on rows sent to the browser.
    #[serde(default = "default_max_rows")]
    pub max_rows: i64,
}

impl Default for CurrentJobsConfig {
    fn default() -> Self {
        Self {
            default_states: default_states(),
            include_archived: false,
            finished_window_hours: default_window_hours(),
            max_rows: default_max_rows(),
        }
    }
}

impl Config {
    /// Load the config file. `Ok(None)` when the file does not exist;
    /// `Err` when it exists but is malformed.
    pub fn load() -> anyhow::Result<Option<Self>> {
        let path = Self::path();
        if !path.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
        let mut cfg: Config = serde_yaml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("parsing {}: {e}", path.display()))?;
        cfg.resolve_paths(path.parent().unwrap_or_else(|| Path::new(".")));
        Ok(Some(cfg))
    }

    fn path() -> PathBuf {
        std::env::var_os("MONITORING_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config.yaml"))
    }

    /// Make relative file paths in the config absolute w.r.t. the config dir.
    fn resolve_paths(&mut self, base: &Path) {
        if let Some(p) = self.prometheus.as_mut() {
            if let Some(ca) = p.ca_cert_path.as_mut() {
                let path = Path::new(ca.as_str());
                if path.is_relative() {
                    *ca = base.join(path).to_string_lossy().into_owned();
                }
            }
        }
    }
}

impl DbConfig {
    /// `key=value` connection string for `tokio_postgres::connect`.
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.dbname, self.user, self.password
        )
    }
}

fn default_pg_port() -> u16 {
    5432
}
fn default_true() -> bool {
    true
}
fn default_prom_timeout() -> u64 {
    30
}
fn default_states() -> Vec<String> {
    ["RUNNING", "PENDING", "COMPLETED", "FAILED"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}
fn default_window_hours() -> i64 {
    24
}
fn default_max_rows() -> i64 {
    500
}