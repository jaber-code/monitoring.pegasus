//! `#[server]` functions — the client's only entry point to server data.
//!
//! Each function is a thin adapter: call [`crate::data::provider()`], map the
//! [`crate::data::DataError`] onto `ServerFnError`, return a [`crate::models`]
//! type. Business logic lives in `data`, not here.
//!
//! Functions that take a struct argument use JSON input encoding — the default
//! URL encoding drops an all-default nested struct entirely, which the server
//! then rejects as a missing field.

use leptos::prelude::*;
use leptos::server_fn::codec::Json;

use crate::models::{Catalog, JobPage, JobQuery};

/// Fetch the dashboard catalog for the home screen.
#[server]
pub async fn get_catalog() -> Result<Catalog, ServerFnError> {
    let result = crate::data::provider().catalog().await;
    #[cfg(feature = "ssr")]
    if let Err(e) = &result {
        tracing::error!("get_catalog failed: {e}");
    }
    result.map_err(|e| ServerFnError::new(e.to_string()))
}

/// Fetch the Current Jobs table, filtered and capped server-side per `query`.
#[server(input = Json)]
pub async fn get_current_jobs(query: JobQuery) -> Result<JobPage, ServerFnError> {
    #[cfg(feature = "ssr")]
    tracing::debug!(?query, "get_current_jobs");

    let result = crate::data::provider().current_jobs(query).await;
    #[cfg(feature = "ssr")]
    if let Err(e) = &result {
        tracing::error!("get_current_jobs failed: {e}");
    }
    result.map_err(|e| ServerFnError::new(e.to_string()))
}
