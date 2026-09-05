//! Plain domain types shared between the server and the browser.
//!
//! Everything here is `Serialize + Deserialize` so it can cross the
//! `#[server]` boundary, and free of Leptos / Axum dependencies.

mod catalog;
mod jobs;

pub use catalog::{Catalog, Dashboard, Folder};
pub use jobs::{
    format_duration, format_mem_mib, JobPage, JobPhase, JobQuery, JobState, JobSummary, SortDir,
    SortKey,
};
