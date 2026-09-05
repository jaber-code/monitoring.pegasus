//! Prometheus HTTP client (ssr only).
//!
//! Used by the time-series graphs on the job-detail screens. The Current Jobs
//! table does not need it — utilization there comes from the `jobs` table.
//!
//! Auth + TLS mirror the cluster's existing tooling: HTTP Basic auth, and
//! either a pinned private-CA bundle (`verify: true` + `ca_cert_path`) or
//! skipped verification (`verify: false`, dev only).

use std::time::Duration;

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use prometheus_http_query::Client as PromClient;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

use crate::config::PrometheusConfig;

#[derive(Clone)]
pub struct Metrics {
    client: PromClient,
}

impl Metrics {
    pub fn connect(cfg: &PrometheusConfig) -> anyhow::Result<Self> {
        let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(cfg.timeout_secs));

        if !cfg.verify {
            builder = builder.danger_accept_invalid_certs(true);
        } else if let Some(ca_path) = &cfg.ca_cert_path {
            let pem =
                std::fs::read(ca_path).with_context(|| format!("reading CA cert {ca_path}"))?;
            let cert = reqwest::Certificate::from_pem(&pem).context("parsing CA cert PEM")?;
            builder = builder.add_root_certificate(cert);
        }

        if let (Some(user), Some(pass)) = (&cfg.username, &cfg.password) {
            let token = STANDARD.encode(format!("{user}:{pass}"));
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {token}"))
                    .context("building basic-auth header")?,
            );
            builder = builder.default_headers(headers);
        }

        let http = builder.build().context("building reqwest client")?;
        let client = PromClient::from(http, cfg.endpoint.as_str())
            .map_err(|e| anyhow::anyhow!("building prometheus client: {e}"))?;

        Ok(Self { client })
    }

    /// Cheap round-trip that proves endpoint + credentials + TLS all work.
    pub async fn health(&self) -> anyhow::Result<()> {
        self.client
            .query("vector(1)")
            .get()
            .await
            .map_err(|e| anyhow::anyhow!("prometheus health query: {e}"))?;
        Ok(())
    }

    /// The underlying query client, for the graph modules built later.
    pub fn client(&self) -> &PromClient {
        &self.client
    }
}
